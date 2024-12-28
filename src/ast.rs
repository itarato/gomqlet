use crate::tokenizer::Token;

pub enum Root {
    Query(Query),
    Mutation(Mutation),
}

pub struct Query {
    pub start_pos: usize,
    pub end_pos: usize,
    pub field_list: FieldList,
}

pub struct Mutation {
    pub start_pos: usize,
    pub end_pos: usize,
    pub field_list: FieldList,
}

pub struct FieldList {
    pub start_pos: usize,
    pub end_pos: usize,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub start_pos: usize,
    pub end_pos: usize,
    pub name: Token,
    pub arglist: Option<ArgList>,
    pub field_list: Option<FieldList>,
}

pub struct ArgList {
    pub start_pos: usize,
    pub end_pos: usize,
    pub params: Vec<ParamKeyValuePair>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParamKeyValuePair {
    pub start_pos: usize,
    pub end_pos: usize,
    pub key: Token,
    pub value: ParamValue,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParamValue {
    Simple(Token),
    List(ListParamValue),
    // For error correction reasons a placeholder type.
    Missing(usize), // pos (representing start and end).
                    // TODO: object, list
}

impl ParamValue {
    pub fn start_pos(&self) -> usize {
        match self {
            ParamValue::Simple(token) => token.pos,
            ParamValue::Missing(pos) => *pos,
            ParamValue::List(list) => list.start_pos,
        }
    }

    pub fn end_pos(&self) -> usize {
        match self {
            ParamValue::Simple(token) => token.end_pos(),
            ParamValue::Missing(pos) => *pos,
            ParamValue::List(list) => list.end_pos,
        }
    }

    pub fn as_list(&self) -> &ListParamValue {
        match &self {
            ParamValue::List(list) => list,
            _ => panic!("Param value expected to be a list"),
        }
    }

    pub fn as_simple(&self) -> &Token {
        match &self {
            ParamValue::Simple(token) => token,
            _ => panic!("Param value expected to be a simple type"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ListParamValue {
    pub start_pos: usize,
    pub end_pos: usize,
    pub elems: Vec<ParamValue>,
}
