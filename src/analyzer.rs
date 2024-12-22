use crate::{
    ast::{ArgList, Query, Root},
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
    }

    fn find_pos_in_root(root: &Root, pos: usize) {
        match root {
            Root::Query(query) => Analyzer::find_pos_in_query(query, pos),
        }
    }

    fn find_pos_in_query(query: &Query, pos: usize) {
        if pos < query.start_pos || pos >= query.end_pos {
            // Outside of the whole query.
            return;
        }

        for field in &query.field_list.fields {
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
                return;
            }

            if let Some(arglist) = &field.arglist {
                let has_match = Analyzer::find_pos_in_arglist(arglist, pos);
                if has_match {
                    return;
                }
            }

            if let Some(field_list) = &field.field_list {
                // CONTINUE HERE
            }
        }

        // In query but not on fields. -> AC can offer fields.
        debug!("On query.");
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
