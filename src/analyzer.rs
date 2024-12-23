use graphql_parser::{parse_schema, schema::Document};

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

    fn find_pos_in_root(root: &Root, pos: usize, schema: &Document<'_, String>) {
        match root {
            Root::Query(query) => Analyzer::find_pos_in_query(query, pos),
        }
    }

    fn find_pos_in_query(query: &Query, pos: usize) {
        Analyzer::find_pos_in_field_list(&query.field_list, pos);
    }

    fn find_pos_in_field_list(field_list: &FieldList, pos: usize) -> bool {
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
                let has_match = Analyzer::find_pos_in_field_list(field_list, pos);
                if has_match {
                    return true;
                }
            }
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
}
