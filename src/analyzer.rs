use std::path::PathBuf;

use crate::{
    ast::{self},
    net_ops::NetOps,
    schema::{self, Type},
    tokenizer::Token,
    util::Error,
};

#[derive(Debug, Clone)]
pub struct SuggestionElem {
    pub name: String,
    pub kind: String,
    pub fuzzy_match_positions: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub elems: Vec<SuggestionElem>,
    pub token: Option<Token>,
}

type AnalyzerResult = Result<Option<Suggestion>, Error>;

pub struct Analyzer {
    schema: schema::Schema,
}

impl Analyzer {
    pub fn new(
        net_ops: &NetOps,
        schema_cache_file_path: &PathBuf,
        reload_schema: bool,
    ) -> Analyzer {
        Analyzer {
            schema: schema::Schema::new(&net_ops, schema_cache_file_path, reload_schema),
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

    fn find_pos_in_query(&self, query: &ast::Query, pos: usize) -> AnalyzerResult {
        let query_scope = self
            .schema
            .type_definition(&self.schema.query_root_name)
            .ok_or("Query is not found in the schema".to_string())?;

        if !query.field_list.range_exclusive().contains(&pos) {
            return Ok(None);
        }

        self.find_pos_in_field_list(&query.field_list, pos, query_scope)
    }

    fn find_pos_in_mutation(&self, mutation: &ast::Mutation, pos: usize) -> AnalyzerResult {
        let mutation_scope = self
            .schema
            .type_definition(&self.schema.mutation_root_name)
            .ok_or("Mutation is not found in the schema".to_string())?;

        if !mutation.field_list.range_exclusive().contains(&pos) {
            return Ok(None);
        }

        self.find_pos_in_field_list(&mutation.field_list, pos, mutation_scope)
    }

    fn find_pos_in_field_list(
        &self,
        field_list: &ast::FieldList,
        pos: usize,
        scope: &schema::Type,
    ) -> AnalyzerResult {
        assert!(field_list.range_exclusive().contains(&pos));

        for field in &field_list.fields {
            if field.range_inclusive().contains(&pos) {
                return self.find_pos_in_field(field, pos, scope);
            }
        }

        // In query but not on fields. -> can offer fields.
        match scope {
            &schema::Type::Object(ref inner_type) | &schema::Type::Interface(ref inner_type) => {
                trace!("Suggestion on all fields of a field list");
                Ok(Some(Suggestion {
                    elems: inner_type.field_names(""),
                    token: None,
                }))
            }
            _ => Ok(None),
        }
    }

    fn find_pos_in_field(
        &self,
        field: &ast::Field,
        pos: usize,
        scope: &schema::Type,
    ) -> AnalyzerResult {
        match &field {
            &ast::Field::Concrete(field) => self.find_pos_in_concrete_field(field, pos, scope),
            &ast::Field::Union(field) => self.find_pos_in_union_field(field, pos, scope),
        }
    }

    fn find_pos_in_union_field(
        &self,
        field: &ast::UnionField,
        pos: usize,
        scope: &schema::Type,
    ) -> AnalyzerResult {
        if field.type_name.range_inclusive().contains(&pos) {
            // Autocomplete for the union type.
            let elems = match &scope {
                &schema::Type::Union(union_type) => {
                    union_type.possible_type_names(&field.type_name.original)
                }
                &schema::Type::Object(inner_type) | &schema::Type::Interface(inner_type) => {
                    inner_type.possible_type_names(&field.type_name.original)
                }
                _ => return Err("Expected union type. Likely not a valid union scope.".into()),
            };
            trace!("Suggestion on a union type");
            return Ok(Some(Suggestion {
                elems,
                token: Some(field.type_name.clone()),
            }));
        }

        if field.field_list.range_exclusive().contains(&pos) {
            let union_type_name = &field.type_name.original;
            let inner_scope = self
                .schema
                .type_definition(union_type_name)
                .ok_or(format!("Missing union type: {}", union_type_name))?;

            return self.find_pos_in_field_list(&field.field_list, pos, inner_scope);
        }

        Ok(None)
    }

    fn find_pos_in_concrete_field(
        &self,
        field: &ast::ConcreteField,
        pos: usize,
        scope: &schema::Type,
    ) -> AnalyzerResult {
        // On field.
        if field.name.range_inclusive().contains(&pos) {
            // On the field name.
            trace!("Suggestion on a concrete field name");
            match scope {
                &schema::Type::Object(ref inner_type)
                | &schema::Type::Interface(ref inner_type) => {
                    return Ok(Some(Suggestion {
                        elems: inner_type.field_names(&field.name.original),
                        token: Some(field.name.clone()),
                    }));
                }
                _ => return Err("Keyword found in a non object/interface scope".into()),
            }
        }

        if let Some(arglist) = &field.arglist {
            if arglist.range_exclusive().contains(&pos) {
                return scope
                    .field(&field.name.original)
                    .ok_or(format!("Invalid field {}", field.name.original).into())
                    .and_then(|field_def| self.find_pos_in_arglist(arglist, pos, &field_def.args));
            }
        }

        if let Some(field_list) = &field.field_list {
            if field_list.range_exclusive().contains(&pos) {
                return self
                    .schema
                    .field_type(scope, &field.name.original)
                    .and_then(|subfield_type_definition| {
                        self.find_pos_in_field_list(field_list, pos, subfield_type_definition)
                    });
            }
        }

        // Between the gaps (not on key [or key's fieldset]) but on the key+fieldset frame.
        return Ok(None);
    }

    fn find_pos_in_arglist(
        &self,
        arglist: &ast::ArgList,
        pos: usize,
        scope: &schema::ArgList,
    ) -> AnalyzerResult {
        assert!(arglist.range_exclusive().contains(&pos));

        // Inside arglist.
        for arg in &arglist.params {
            if pos > arg.end_pos {
                continue;
            }

            if pos < arg.start_pos {
                break;
            }

            if arg.key.range_inclusive().contains(&pos) {
                // On arg key.
                trace!("Suggestion on arglist key: {}", arg.key.original);
                return Ok(Some(Suggestion {
                    elems: scope.arg_names(&arg.key.original),
                    token: Some(arg.key.clone()),
                }));
            } else if arg.value.range_inclusive().contains(&pos) {
                // We are in the <arg-name>: ______ scope.
                //                           ^^^^^^
                // Get the current arg definition.
                let current_arg = scope
                    .arg(&arg.key.original)
                    .ok_or(format!("Invalid arg name {}", &arg.key.original))?;
                let current_arg_type = &current_arg.arg_type;

                return self.find_pos_in_arglist_value(&arg.value, current_arg_type, pos);
            }

            return Ok(None);
        }

        // In arglist -> offer key.
        trace!("Suggestion on all arglist fields");
        Ok(Some(Suggestion {
            elems: scope.arg_names(&String::new()),
            token: None,
        }))
    }

    fn find_pos_in_arglist_value(
        &self,
        value: &ast::ParamValue,
        scope: &schema::TypeClass,
        pos: usize,
    ) -> AnalyzerResult {
        match &value {
            crate::ast::ParamValue::Simple(token) => match scope {
                schema::TypeClass::Enum(enum_type_name) => {
                    return self
                        .schema
                        .type_definition(enum_type_name)
                        .ok_or(format!("Enum type {} not found", enum_type_name).into())
                        .and_then(|ref enum_type| match enum_type {
                            &schema::Type::Enum(inner_type) => {
                                trace!("Suggestion on enum arglist value");
                                Ok(Some(Suggestion {
                                    elems: inner_type.field_names(&token.original),
                                    token: Some(token.clone()),
                                }))
                            }
                            _ => Err("Expected enum type".into()),
                        })
                }
                _ => {}
            },
            crate::ast::ParamValue::Object(object_arglist) => {
                // Get the type name of current arg value.
                let value_type_name = match scope.skip_non_null() {
                    schema::TypeClass::Input(name) => name,
                    _ => {
                        return Err(
                            format!("Exected input type for arg value. Got: {:?}", scope).into(),
                        )
                    }
                };
                // Get the schema type definition of the arg value's type.
                let value_type = self
                    .schema
                    .type_definition(value_type_name)
                    .ok_or(format!("Type {} not found.", &value_type_name))?;

                // Get the inner args of that type.
                let value_args = match value_type {
                    Type::InputObject(input_object) => &input_object.args,
                    _ => {
                        return Err(format!(
                            "Type {} is expected to be an input object",
                            &value_type_name
                        )
                        .into())
                    }
                };

                return if object_arglist.range_exclusive().contains(&pos) {
                    self.find_pos_in_arglist(object_arglist, pos, value_args)
                } else {
                    Ok(None)
                };
            }
            crate::ast::ParamValue::List(list) => {
                for list_param_value in &list.elems {
                    if list_param_value.range_inclusive().contains(&pos) {
                        let inner_scope = match &scope.skip_non_null() {
                            schema::TypeClass::List(inner_type_class) => inner_type_class,
                            _ => {
                                return Err(format!(
                                    "Exected list type for arg value. Got: {:?}",
                                    scope
                                )
                                .into())
                            }
                        };

                        return self.find_pos_in_arglist_value(list_param_value, inner_scope, pos);
                    }
                }
            }
            crate::ast::ParamValue::Missing(_pos) => match &scope {
                schema::TypeClass::Enum(enum_type_name) => {
                    return self
                        .schema
                        .type_definition(enum_type_name)
                        .ok_or(format!("Enum type {} not found", enum_type_name).into())
                        .and_then(|ref enum_type| match enum_type {
                            &schema::Type::Enum(inner_type) => {
                                trace!("Suggestion on all enum options of an arglist value");
                                Ok(Some(Suggestion {
                                    elems: inner_type.field_names(""),
                                    token: None,
                                }))
                            }
                            _ => Err("Expected enum type".into()),
                        })
                }
                _ => {}
            },
        }

        Ok(None)
    }
}
