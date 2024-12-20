pub enum Root {
    Query(Query),
}

pub struct Query {
    pub pos: usize,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub pos: usize,
    pub name: String,
    pub arglist: Option<ArgList>,
    pub fields: Vec<Field>,
}

pub struct ArgList {
    pub pos: usize,
    pub params: Vec<ParamKeyValuePair>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParamKeyValuePair {
    pub pos: usize,
    pub key: String,
    pub value: ParamValue,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParamValue {
    Int(i32),
    Str(String),
    Keyword(String),
    // TODO: object, list
}
