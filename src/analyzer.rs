use crate::{
    ast::{self},
    schema::{self, Type},
    tokenizer::Token,
};

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub elems: Vec<String>,
    pub token: Option<Token>,
}

type AnalyzerResult = Result<Option<Suggestion>, String>;

pub struct Analyzer {
    schema: schema::Schema,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            schema: schema::Schema::new(),
        }
    }

    pub fn analyze(&self, root: ast::Root, pos: usize) -> AnalyzerResult {
        self.find_pos_in_root(&root, pos)
    }

    fn find_pos_in_root(&self, root: &ast::Root, pos: usize) -> AnalyzerResult {
        match root {
            ast::Root::Query(query) => self.find_pos_in_query(query, pos),
            ast::Root::Mutation(mutation) => self.find_pos_in_mutation(mutation, pos),
        }
    }

    // TODO: Make sure the CALLER IS RESPONSIBLE of knowing if the called scope is ON POS!!!!

    fn find_pos_in_query(&self, query: &ast::Query, pos: usize) -> AnalyzerResult {
        let query_scope = self
            .schema
            .type_definition(self.schema.query_root_name.clone())
            .ok_or("Query is not found in the schema".to_string())?;

        self.find_pos_in_field_list(&query.field_list, pos, query_scope)
    }

    fn find_pos_in_mutation(&self, query: &ast::Mutation, pos: usize) -> AnalyzerResult {
        let mutation_scope = self
            .schema
            .type_definition(self.schema.mutation_root_name.clone())
            .ok_or("Mutation is not found in the schema".to_string())?;

        self.find_pos_in_field_list(&query.field_list, pos, mutation_scope)
    }

    fn find_pos_in_field_list(
        &self,
        field_list: &ast::FieldList,
        pos: usize,
        scope: &schema::Type,
    ) -> AnalyzerResult {
        if pos < field_list.start_pos || pos >= field_list.end_pos {
            // Outside of the whole query.
            return Ok(None);
        }

        for field in &field_list.fields {
            if pos > field.end_pos {
                continue;
            }

            if pos < field.start_pos {
                break;
            }

            // On field.
            if pos >= field.name.pos && pos <= field.name.end_pos() {
                // On the field name.
                return Ok(Some(Suggestion {
                    elems: scope.field_names(field.name.original.clone()),
                    token: Some(field.name.clone()),
                }));
            }

            if let Some(arglist) = &field.arglist {
                if pos >= arglist.start_pos && pos <= arglist.end_pos {
                    return scope
                        .field(field.name.original.clone())
                        .ok_or(format!("Invalid field {}", field.name.original))
                        .and_then(|field_def| {
                            self.find_pos_in_arglist(arglist, pos, &field_def.args)
                        });
                }
            }

            if let Some(field_list) = &field.field_list {
                return self
                    .schema
                    .field_type(scope, field.name.original.clone())
                    .and_then(|subfield_type_definition| {
                        self.find_pos_in_field_list(field_list, pos, subfield_type_definition)
                    });
            }

            return Ok(None);
        }

        // In query but not on fields. -> AC can offer fields.
        Ok(Some(Suggestion {
            elems: scope.field_names(String::new()),
            token: None,
        }))
    }

    fn find_pos_in_arglist(
        &self,
        arglist: &ast::ArgList,
        pos: usize,
        scope: &schema::ArgList,
    ) -> AnalyzerResult {
        if pos < arglist.start_pos || pos > arglist.end_pos {
            return Ok(None);
        }

        // Inside arglist.
        for arg in &arglist.params {
            if pos > arg.end_pos {
                continue;
            }

            if pos < arg.start_pos {
                break;
            }

            if pos >= arg.key.pos && pos <= arg.key.end_pos() {
                // On arg key.
                debug!("On key: {}", arg.key.original);
                return Ok(Some(Suggestion {
                    elems: scope.arg_names(&arg.key.original),
                    token: Some(arg.key.clone()),
                }));
            } else if pos >= arg.value.start_pos() && pos <= arg.value.end_pos() {
                // On arg value.
                debug!("On arg value: {:?}", arg.value);

                match &arg.value {
                    crate::ast::ParamValue::Simple(token) => {
                        // TODO!!!
                    }
                    crate::ast::ParamValue::Object(object) => {
                        // In cart field (scope)
                        // On arglist arg key: `input`
                        // -> read type => INPUT_OBJECT (object) of CartInput
                        let current_arg = scope
                            .arg(&arg.key.original)
                            .ok_or(format!("Invalid arg name {}", &arg.key.original))?;
                        let value_type_name = match &current_arg.arg_type {
                            schema::TypeClass::Input(name) => name,
                            _ => {
                                return Err(format!(
                                    "Arg value of key {} exected to be input type",
                                    arg.key.original
                                ))
                            }
                        };

                        // -> lookup CartInput
                        let value_type = self
                            .schema
                            .type_definition(value_type_name.clone())
                            .ok_or(format!("Type {} not found.", &value_type_name))?;

                        // -> get `inputFields`: attributes/lines/discountCodes/...
                        let value_args = match value_type {
                            Type::InputObject(input_object) => &input_object.args,
                            _ => {
                                return Err(format!(
                                    "Type {} is expected to be an input object",
                                    &value_type_name
                                ))
                            }
                        };

                        return self.find_pos_in_arglist(object, pos, value_args);
                    }
                    crate::ast::ParamValue::List(list) => {
                        // TODO!!!
                    }
                    crate::ast::ParamValue::Missing(pos) => {
                        // TODO!!!
                    }
                }

                // TODO!!!
                // todo!("On-value autocomplete (simple,list,object)");

                // TODO: when the cursor is after the last keyword, the replacement `Missing` type has no length,
                //       so this function cannot identify it and offer values options.
                //       Maybe make the missing value own a length (between colon and next token)?

                return Ok(None);
            } else {
                return Ok(None);
            }
        }

        // In arglist -> offer key.
        debug!("On arglist.");
        Ok(Some(Suggestion {
            elems: scope.arg_names(&String::new()),
            token: None,
        }))
    }
}
