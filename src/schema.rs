use std::{
    fmt,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use serde_json::Value;

use crate::{analyzer::SuggestionElem, net_ops::NetOps};

#[derive(Debug)]
pub enum TypeClass {
    NonNull(Box<TypeClass>),
    List(Box<TypeClass>),
    Object(String),
    Enum(String),
    Interface(String),
    Scalar(String),
    Input(String),
    Union(String),
}

impl fmt::Display for TypeClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeClass::Object(name) => write!(f, "Obj<{}>", name),
            TypeClass::Enum(name) => write!(f, "Enum<{}>", name),
            TypeClass::Interface(name) => write!(f, "Int<{}>", name),
            TypeClass::Scalar(name) => write!(f, "Scalar<{}>", name),
            TypeClass::Input(name) => write!(f, "Input<{}>", name),
            TypeClass::Union(name) => write!(f, "Union<{}>", name),
            TypeClass::NonNull(inner) => write!(f, "!{}", inner),
            TypeClass::List(inner) => write!(f, "[{}]", inner),
        }
    }
}

impl TypeClass {
    fn underlying_type_name(&self) -> Option<String> {
        match self {
            TypeClass::Object(name) => Some(name.clone()),
            TypeClass::Enum(name) => Some(name.clone()),
            TypeClass::Interface(name) => Some(name.clone()),
            TypeClass::Scalar(name) => Some(name.clone()),
            TypeClass::Input(name) => Some(name.clone()),
            TypeClass::Union(name) => Some(name.clone()),
            TypeClass::NonNull(inner) => inner.underlying_type_name(),
            TypeClass::List(inner) => inner.underlying_type_name(),
        }
    }

    pub fn skip_non_null(&self) -> &TypeClass {
        match self {
            TypeClass::Object(_) => self,
            TypeClass::Enum(_) => self,
            TypeClass::Interface(_) => self,
            TypeClass::Scalar(_) => self,
            TypeClass::Input(_) => self,
            TypeClass::Union(_) => self,
            TypeClass::NonNull(inner) => inner,
            TypeClass::List(_) => self,
        }
    }

    fn from_json_value(node: &Value) -> TypeClass {
        let kind = node.as_object().unwrap()["kind"].as_str().unwrap();
        match kind {
            "NON_NULL" => TypeClass::NonNull(Box::new(TypeClass::from_json_value(
                &node.as_object().unwrap()["ofType"],
            ))),
            "LIST" => TypeClass::List(Box::new(TypeClass::from_json_value(
                &node.as_object().unwrap()["ofType"],
            ))),
            "OBJECT" => TypeClass::Object(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "INTERFACE" => TypeClass::Interface(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "SCALAR" => TypeClass::Scalar(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "INPUT_OBJECT" => TypeClass::Input(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "ENUM" => TypeClass::Enum(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "UNION" => TypeClass::Union(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            _ => unimplemented!("Unmapped field type: {}", kind),
        }
    }
}

pub struct Arg {
    pub name: String,
    pub arg_type: TypeClass,
}

impl Arg {
    fn from_json_value(node: &Value) -> Arg {
        let object = node.as_object().unwrap();
        let name = object["name"].as_str().unwrap().to_string();
        let arg_type = TypeClass::from_json_value(&object["type"]);

        Arg { name, arg_type }
    }
}

pub struct ArgList {
    elems: Vec<Arg>,
}

impl ArgList {
    pub fn arg_names(&self, prefix: &String) -> Vec<SuggestionElem> {
        self.elems
            .iter()
            .filter_map(|arg| {
                if arg.name.starts_with(prefix) {
                    Some(SuggestionElem {
                        name: arg.name.clone(),
                        kind: format!("{}", arg.arg_type),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn arg(&self, name: &String) -> Option<&Arg> {
        for arg in &self.elems {
            if &arg.name == name {
                return Some(arg);
            }
        }

        None
    }
}

pub struct Field {
    pub name: String,
    pub field_type: TypeClass,
    pub args: ArgList,
}

impl Field {
    fn from_json_value(node: &Value) -> Field {
        let name = node.as_object().unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();

        Field {
            name,
            field_type: TypeClass::from_json_value(&node.as_object().unwrap()["type"]),
            args: ArgList {
                elems: Field::resolve_args(node.as_object().unwrap()["args"].as_array().unwrap()),
            },
        }
    }

    fn resolve_args(raw_args: &Vec<Value>) -> Vec<Arg> {
        raw_args
            .iter()
            .map(|raw_arg| Arg {
                name: raw_arg.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                arg_type: TypeClass::from_json_value(&raw_arg.as_object().unwrap()["type"]),
            })
            .collect()
    }
}

pub struct ObjectType {
    name: String,
    fields: Vec<Field>,
}

pub struct InputObjectType {
    pub name: String,
    pub args: ArgList,
}

pub struct EnumType {
    name: String,
    elems: Vec<String>,
}

pub enum Type {
    Object(ObjectType),
    Interface(ObjectType),
    InputObject(InputObjectType),
    Enum(EnumType),
}

impl Type {
    pub fn from_json_value(node: &Value) -> Option<Type> {
        let object = node.as_object().unwrap();
        let name = object["name"].as_str().unwrap().to_string();
        let kind = object["kind"].as_str().unwrap();

        match kind {
            "OBJECT" | "INTERFACE" => {
                let fields = object["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|field_def| Field::from_json_value(field_def))
                    .collect();

                if kind == "OBJECT" {
                    Some(Type::Object(ObjectType { name, fields }))
                } else {
                    Some(Type::Interface(ObjectType { name, fields }))
                }
            }
            "INPUT_OBJECT" => {
                let args_elems = object["inputFields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|arg_def| Arg::from_json_value(arg_def))
                    .collect();
                let args = ArgList { elems: args_elems };

                Some(Type::InputObject(InputObjectType { name, args }))
            }
            "ENUM" => {
                let elems = object["enumValues"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| {
                        value.as_object().unwrap()["name"]
                            .as_str()
                            .unwrap()
                            .to_string()
                    })
                    .collect();

                Some(Type::Enum(EnumType { name, elems }))
            }
            _ => None,
        }
    }

    pub fn field_names(&self, prefix: &str) -> Vec<SuggestionElem> {
        match self {
            Type::Object(object_type) | Type::Interface(object_type) => object_type
                .fields
                .iter()
                .filter_map(|field| {
                    if field.name.starts_with(prefix) {
                        Some(SuggestionElem {
                            name: field.name.clone(),
                            kind: format!("{}", field.field_type),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            Type::InputObject(input_object) => input_object
                .args
                .elems
                .iter()
                .filter_map(|arg| {
                    if arg.name.starts_with(prefix) {
                        Some(SuggestionElem {
                            name: arg.name.clone(),
                            kind: format!("{}", arg.arg_type),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            Type::Enum(enum_type) => enum_type
                .elems
                .iter()
                .filter_map(|enum_value| {
                    if enum_value.starts_with(prefix) {
                        Some(SuggestionElem {
                            name: enum_value.clone(),
                            kind: "Enum".to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    pub fn field(&self, name: String) -> Option<&Field> {
        match self {
            Type::Object(object_type) | Type::Interface(object_type) => {
                for field in &object_type.fields {
                    if field.name == name {
                        return Some(field);
                    }
                }
                None
            }
            Type::InputObject(_) => None,
            Type::Enum(_) => None,
        }
    }
}

pub struct Schema {
    types: Vec<Type>,
    pub query_root_name: String,
    pub mutation_root_name: String,
}

impl Schema {
    pub fn new(net_ops: &NetOps, schema_cache_file_path: &PathBuf, reload_schema: bool) -> Schema {
        let schema: Value = Schema::fetch_schema(&net_ops, schema_cache_file_path, reload_schema);

        let query_root_name = schema.as_object().unwrap()["data"].as_object().unwrap()["__schema"]
            .as_object()
            .unwrap()["queryType"]
            .as_object()
            .unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();

        let mutation_root_name = schema.as_object().unwrap()["data"].as_object().unwrap()
            ["__schema"]
            .as_object()
            .unwrap()["mutationType"]
            .as_object()
            .unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();

        Schema {
            types: Schema::read_types(&schema),
            query_root_name,
            mutation_root_name,
        }
    }

    fn fetch_schema(
        net_ops: &NetOps,
        schema_cache_file_path: &PathBuf,
        reload_schema: bool,
    ) -> Value {
        if reload_schema
            || !fs::exists(schema_cache_file_path).expect("Failed checking schema cache file")
        {
            let response_body = net_ops.fetch_live_schema();

            let mut cache =
                File::create(schema_cache_file_path).expect("Failed creating schema cache");
            cache
                .write_all(response_body.as_bytes())
                .expect("Failed saving schema");

            serde_json::from_str(&response_body).unwrap()
        } else {
            let cache = File::open(schema_cache_file_path).expect("Failed opening schema cache");
            serde_json::from_reader(cache).unwrap()
        }
    }

    pub fn type_definition(&self, name: &String) -> Option<&Type> {
        for ty in &self.types {
            match ty {
                Type::Object(object_type) | Type::Interface(object_type) => {
                    if &object_type.name == name {
                        return Some(ty);
                    }
                }
                Type::InputObject(input_object) => {
                    if &input_object.name == name {
                        return Some(ty);
                    }
                }
                Type::Enum(enum_type) => {
                    if &enum_type.name == name {
                        return Some(ty);
                    }
                }
            }
        }

        None
    }

    fn read_types(schema: &Value) -> Vec<Type> {
        schema.as_object().unwrap()["data"].as_object().unwrap()["__schema"]
            .as_object()
            .unwrap()["types"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|type_def| Type::from_json_value(type_def))
            .collect()
    }

    pub fn field_type(&self, type_definition: &Type, field_name: String) -> Result<&Type, String> {
        type_definition
            .field(field_name.clone())
            .ok_or(format!("Field {} not found", field_name))
            .and_then(|field_definition| {
                field_definition
                    .field_type
                    // FIXME: I'm not sure about this one. Removing non-null wrapper is ok but list might not.
                    //        check if this lies for list types.
                    .underlying_type_name()
                    .ok_or(format!("Field type of {} not found", field_name))
            })
            .and_then(|field_type_name| {
                self.type_definition(&field_type_name).ok_or(format!(
                    "Definition of type {} of field {} not found",
                    field_type_name, field_name
                ))
            })
    }
}
