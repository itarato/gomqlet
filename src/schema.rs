use std::fs::File;

use serde_json::Value;

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
}

pub struct Arg {
    name: String,
    arg_type: TypeClass,
}

pub struct Field {
    pub name: String,
    field_type: TypeClass,
    args: Vec<Arg>,
}

impl Field {
    fn from_json_value(node: &Value) -> Field {
        let name = node.as_object().unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();

        Field {
            name,
            field_type: Field::resolve_type(&node.as_object().unwrap()["type"]),
            args: Field::resolve_args(node.as_object().unwrap()["args"].as_array().unwrap()),
        }
    }

    pub fn arg_names(&self, prefix: &String) -> Vec<String> {
        self.args
            .iter()
            .filter_map(|arg| {
                if arg.name.starts_with(prefix) {
                    Some(arg.name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn resolve_type(node: &Value) -> TypeClass {
        let kind = node.as_object().unwrap()["kind"].as_str().unwrap();
        match kind {
            "NON_NULL" => TypeClass::NonNull(Box::new(Field::resolve_type(
                &node.as_object().unwrap()["ofType"],
            ))),
            "LIST" => TypeClass::List(Box::new(Field::resolve_type(
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

    fn resolve_args(raw_args: &Vec<Value>) -> Vec<Arg> {
        raw_args
            .iter()
            .map(|raw_arg| Arg {
                name: raw_arg.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                arg_type: Field::resolve_type(&raw_arg.as_object().unwrap()["type"]),
            })
            .collect()
    }
}

pub struct ObjectType {
    name: String,
    fields: Vec<Field>,
}

pub enum Type {
    Object(ObjectType),
}

impl Type {
    pub fn field_names(&self, prefix: String) -> Vec<String> {
        match self {
            Type::Object(object_type) => object_type
                .fields
                .iter()
                .filter_map(|field| {
                    if field.name.starts_with(&prefix) {
                        Some(field.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    pub fn field(&self, name: String) -> Option<&Field> {
        match self {
            Type::Object(object_type) => {
                for field in &object_type.fields {
                    if field.name == name {
                        return Some(field);
                    }
                }
                None
            }
        }
    }
}

pub struct Schema {
    types: Vec<Type>,
    pub query_root_name: String,
    pub mutation_root_name: String,
}

impl Schema {
    pub fn new() -> Schema {
        let schema: Value =
            serde_json::from_reader(File::open("./misc/shopify.json").unwrap()).unwrap();

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

    pub fn type_definition(&self, name: String) -> Option<&Type> {
        for ty in &self.types {
            match ty {
                Type::Object(object_type) => {
                    if object_type.name == name {
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
            .filter_map(|type_def| {
                let object_type_def = type_def.as_object().unwrap();

                let name = object_type_def["name"].as_str().unwrap().to_string();
                match object_type_def["kind"].as_str().unwrap() {
                    "OBJECT" => {
                        let fields = object_type_def["fields"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|field_def| Field::from_json_value(field_def))
                            .collect();

                        Some(Type::Object(ObjectType { name, fields }))
                    }
                    // TODO: handle other types!
                    _ => None,
                }
            })
            .collect()
    }

    pub fn field_type(&self, type_definition: &Type, field_name: String) -> Result<&Type, String> {
        type_definition
            .field(field_name.clone())
            .ok_or(format!("Field {} not found", field_name))
            .and_then(|field_definition| {
                field_definition
                    .field_type
                    .underlying_type_name()
                    .ok_or(format!("Field type of {} not found", field_name))
            })
            .and_then(|field_type_name| {
                self.type_definition(field_type_name.clone()).ok_or(format!(
                    "Definition of type {} of field {} not found",
                    field_type_name, field_name
                ))
            })
    }
}
