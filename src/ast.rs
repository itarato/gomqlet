pub enum Root {
    Query(Query),
}

pub struct Query {
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: String,
    pub arglist: Option<ArgList>,
    pub fields: Vec<Field>,
}

pub struct ArgList {
    pub params: Vec<ParamKeyValuePair>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParamKeyValuePair {
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
