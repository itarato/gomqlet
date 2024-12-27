use std::fs::File;

use serde_json::Value;

use crate::{
    ast::{ArgList, FieldList, Query, Root},
    parser::{ParseError, Parser},
    schema::{Schema, Type},
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

pub struct Analyzer {
    schema: Schema,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            schema: Schema::new(),
        }
    }

    pub fn analyze(&self, tokens: Vec<Token>, pos: usize) -> AnalyzerResult {
        let ast = Parser::new(tokens).parse();

        match ast {
            Err(err) => AnalyzerResult::ParseError(err),
            Ok(ref root) => self.find_pos_in_root(root, pos),
        }
    }

    fn find_pos_in_root(&self, root: &Root, pos: usize) -> AnalyzerResult {
        match root {
            Root::Query(query) => self.find_pos_in_query(query, pos),
        }
    }

    fn find_pos_in_query(&self, query: &Query, pos: usize) -> AnalyzerResult {
        let query_scope = match self
            .schema
            .type_definition(self.schema.query_root_name.clone())
        {
            Some(scope) => scope,
            None => {
                return AnalyzerResult::DefinitionError("Query is not found in the schema".into())
            }
        };

        self.find_pos_in_field_list(&query.field_list, pos, query_scope)
            .unwrap_or(AnalyzerResult::Empty)
    }

    fn find_pos_in_field_list(
        &self,
        field_list: &FieldList,
        pos: usize,
        scope: &Type,
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
                    scope.field_names(field.name.original.clone()),
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
                    match self.schema.field_type(scope, field.name.original.clone()) {
                        Ok(subfield_type_definition) => subfield_type_definition,
                        Err(error) => return Some(AnalyzerResult::DefinitionError(error)),
                    };

                let result = self.find_pos_in_field_list(field_list, pos, subfield_type_definition);
                if result.is_some() {
                    return result;
                }
            }

            return Some(AnalyzerResult::Empty);
        }

        // In query but not on fields. -> AC can offer fields.
        Some(AnalyzerResult::Autocomplete(
            scope.field_names(String::new()),
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
}
