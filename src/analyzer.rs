use graphql_parser::{
    parse_schema,
    query::Type,
    schema::{Document, Field, TypeDefinition},
};

use crate::{
    ast::{ArgList, FieldList, Query, Root},
    parser::{ParseError, Parser},
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

#[derive(Debug)]
pub enum AnalyzerResult {
    Autocomplete(Vec<String>),
    ParseError(ParseError),
    DefinitionError(String),
    Empty,
}

pub struct Analyzer;

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer
    }

    pub fn analyze(&self, tokens: Vec<Token>, pos: usize) -> AnalyzerResult {
        let ast = Parser::new(tokens).parse();

        let schema = parse_schema::<String>(
            r"
            type Query {
                books: [Book]
                authors: [Author]
            }

            type Book {
                title: String
                author: Author
            }

            type Author {
                name: String
                nameDefault: String
                books: [Book]
                address: Address
            }

            type Address {
                city: String
                zip: String
            }
            ",
        )
        .unwrap();

        match ast {
            Err(err) => AnalyzerResult::ParseError(err),
            Ok(ref root) => Analyzer::find_pos_in_root(root, pos, &schema),
        }
    }

    fn find_pos_in_root<'a>(
        root: &Root,
        pos: usize,
        schema: &'a Document<'a, String>,
    ) -> AnalyzerResult {
        match root {
            Root::Query(query) => Analyzer::find_pos_in_query(query, pos, schema),
        }
    }

    fn find_pos_in_query<'a>(
        query: &Query,
        pos: usize,
        schema: &'a Document<'a, String>,
    ) -> AnalyzerResult {
        let query_scope = match Analyzer::lookup_graphql_type_definition(schema, "Query".into()) {
            Some(scope) => scope,
            None => {
                return AnalyzerResult::DefinitionError("Query is not found in the schema".into())
            }
        };

        Analyzer::find_pos_in_field_list(&query.field_list, pos, schema, query_scope)
            .unwrap_or(AnalyzerResult::Empty)
    }

    fn find_pos_in_field_list<'a>(
        field_list: &FieldList,
        pos: usize,
        schema: &'a Document<'a, String>,
        scope: &'a TypeDefinition<'a, String>,
    ) -> Option<AnalyzerResult> {
        if pos < field_list.start_pos || pos > field_list.end_pos {
            // Outside of the whole query.
            return None;
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
                return Some(AnalyzerResult::Autocomplete(
                    Analyzer::lookup_field_name_strings_of_type_definition(
                        scope,
                        field.name.original.clone(),
                    ),
                ));
            }

            if let Some(arglist) = &field.arglist {
                let result = Analyzer::find_pos_in_arglist(arglist, pos);
                if result.is_some() {
                    return result;
                }
            }

            if let Some(field_list) = &field.field_list {
                let subfield_type_definition =
                    match Analyzer::lookup_field_type_defition_of_parent_type_definition(
                        schema,
                        scope,
                        field.name.original.clone(),
                    ) {
                        Ok(subfield_type_definition) => subfield_type_definition,
                        Err(error) => return Some(error),
                    };

                let result = Analyzer::find_pos_in_field_list(
                    field_list,
                    pos,
                    schema,
                    subfield_type_definition,
                );
                if result.is_some() {
                    return result;
                }
            }

            return Some(AnalyzerResult::Empty);
        }

        // In query but not on fields. -> AC can offer fields.
        Some(AnalyzerResult::Autocomplete(
            Analyzer::lookup_field_name_strings_of_type_definition(scope, String::new()),
        ))
    }

    fn find_pos_in_arglist(arglist: &ArgList, pos: usize) -> Option<AnalyzerResult> {
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
                    todo!("On-key autocomplete");
                    return Some(AnalyzerResult::Empty);
                }

                if pos >= arg.value.start_pos() && pos <= arg.value.end_pos() {
                    // On arg value.
                    debug!("On arg value: {:?}", arg.value);
                    todo!("On-value autocomplete");
                    return Some(AnalyzerResult::Empty);
                }
            }

            // In arglist -> offer key.
            debug!("On arglist.");
            todo!("Arglist autocomplete");
            return Some(AnalyzerResult::Empty);
        }

        None
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

    fn lookup_field_name_strings_of_type_definition<'a>(
        type_definition: &'a TypeDefinition<'a, String>,
        prefix: String,
    ) -> Vec<String> {
        let mut name_strings = vec![];

        match type_definition {
            TypeDefinition::Object(object) => {
                for field in &object.fields {
                    if field.name.starts_with(&prefix) {
                        name_strings.push(field.name.clone());
                    }
                }
            }
            _ => {}
        }

        name_strings
    }

    fn lookup_field_type_defition_of_parent_type_definition<'a>(
        schema: &'a Document<'a, String>,
        type_definition: &'a TypeDefinition<'a, String>,
        field_name: String,
    ) -> Result<&'a TypeDefinition<'a, String>, AnalyzerResult> {
        let field_definition =
            Analyzer::lookup_field_in_object_type_definition(type_definition, field_name.clone());
        if field_definition.is_none() {
            return Err(AnalyzerResult::DefinitionError(format!(
                "Field {} not found",
                field_name
            )));
        }
        let field_definition = field_definition.unwrap();

        let field_type_name = Analyzer::lookup_type_name_from_field_definition(field_definition);
        if field_type_name.is_none() {
            return Err(AnalyzerResult::DefinitionError(format!(
                "Type of field {} not found",
                field_name
            )));
        }
        let field_type_name = field_type_name.unwrap();

        let subfield_type_definition =
            Analyzer::lookup_graphql_type_definition(schema, field_type_name.clone());
        if subfield_type_definition.is_none() {
            return Err(AnalyzerResult::DefinitionError(format!(
                "Definition of type {} of field {} not found",
                field_type_name, field_name
            )));
        }
        Ok(subfield_type_definition.unwrap())
    }
}
