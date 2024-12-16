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
    params: Vec<ParamKeyValuePair>,
}

pub struct ParamKeyValuePair {
    key: String,
    value: ParamValue,
}

pub enum ParamValue {
    Int(i32),
    Str(String),
}
