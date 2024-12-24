use graphql_parser::{
    parse_schema,
    query::Type,
    schema::{Document, Field, TypeDefinition},
};

use crate::{
    ast::{ArgList, FieldList, Query, Root},
    parser::Parser,
    tokenizer::Token,
};

/*

query {
    users(first: 10) {
        id
        name
        address
    }
}

*/

pub struct Analyzer;

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer
    }

    pub fn analyze(&self, tokens: Vec<Token>, pos: usize) {
        let ast = Parser::new(tokens).parse();

        let schema = parse_schema::<String>("./misc/example.schema").unwrap();

        match &ast {
            Err(err) => debug!("AST error: {:?}", err),
            Ok(root) => Analyzer::find_pos_in_root(root, pos, &schema),
        }
    }

    fn find_pos_in_root<'a>(root: &Root, pos: usize, schema: &'a Document<'a, String>) {
        match root {
            Root::Query(query) => Analyzer::find_pos_in_query(query, pos, schema),
        }
    }

    fn find_pos_in_query<'a>(query: &Query, pos: usize, schema: &'a Document<'a, String>) {
        let query_scope = Analyzer::lookup_graphql_type_definition(schema, "Query".into()).unwrap();
        Analyzer::find_pos_in_field_list(&query.field_list, pos, schema, query_scope);
    }

    fn find_pos_in_field_list<'a>(
        field_list: &FieldList,
        pos: usize,
        schema: &'a Document<'a, String>,
        scope: &'a TypeDefinition<'a, String>,
    ) -> bool {
        if pos < field_list.start_pos || pos > field_list.end_pos {
            // Outside of the whole query.
            return false;
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
                debug!("On field name: {}", field.name.original);
                return true;
            }

            if let Some(arglist) = &field.arglist {
                let has_match = Analyzer::find_pos_in_arglist(arglist, pos);
                if has_match {
                    return true;
                }
            }

            if let Some(field_list) = &field.field_list {
                if let Some(field_definition) = Analyzer::lookup_field_in_object_type_definition(
                    scope,
                    field.name.original.clone(),
                ) {
                    if let Some(field_type_name) =
                        Analyzer::lookup_type_name_from_field_definition(field_definition)
                    {
                        if let Some(subfield_type_definition) =
                            Analyzer::lookup_graphql_type_definition(schema, field_type_name)
                        {
                            let has_match = Analyzer::find_pos_in_field_list(
                                field_list,
                                pos,
                                schema,
                                subfield_type_definition,
                            );
                            if has_match {
                                return true;
                            }
                        }
                    }
                }
            }

            return false;
        }

        // In query but not on fields. -> AC can offer fields.
        debug!("On field list.");
        true
    }

    fn find_pos_in_arglist(arglist: &ArgList, pos: usize) -> bool {
        if pos >= arglist.start_pos && pos < arglist.end_pos {
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
                    return true;
                }

                if pos >= arg.value.start_pos() && pos <= arg.value.end_pos() {
                    // On arg value.
                    debug!("On arg value: {:?}", arg.value);
                    return true;
                }
            }

            // In arglist -> offer key.
            debug!("On arglist.");
            return true;
        }

        false
    }

    fn lookup_graphql_type_definition<'a>(
        schema: &'a Document<'a, String>,
        name: String,
    ) -> Option<&'a TypeDefinition<'a, String>> {
        for definition in &schema.definitions {
            match definition {
                graphql_parser::schema::Definition::TypeDefinition(type_definition) => {
                    match type_definition {
                        TypeDefinition::Object(object) => {
                            if object.name == name {
                                return Some(type_definition);
                            }
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }

        None
    }

    fn lookup_field_in_object_type_definition<'a>(
        type_definition: &'a TypeDefinition<'a, String>,
        name: String,
    ) -> Option<&'a Field<'a, String>> {
        match type_definition {
            TypeDefinition::Object(object) => {
                for field in &object.fields {
                    if field.name == name {
                        return Some(field);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn lookup_type_name_from_field_definition<'a>(
        field_definition: &'a Field<'a, String>,
    ) -> Option<String> {
        Analyzer::lookup_type_name_from_type(&field_definition.field_type)
    }

    fn lookup_type_name_from_type<'a>(ty: &'a Type<'a, String>) -> Option<String> {
        match &ty {
            Type::NamedType(name) => Some(name.clone()),
            Type::ListType(list) => Analyzer::lookup_type_name_from_type(list),
            Type::NonNullType(non_null_ty) => Analyzer::lookup_type_name_from_type(non_null_ty),
        }
    }
}
