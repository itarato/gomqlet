use std::fs::File;

use serde_json::Value;

pub struct Field {
    pub name: String,
}

pub struct ObjectType {
    name: String,
    fields: Vec<Field>,
}

pub enum Type {
    Object(ObjectType),
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
            .as_str()
            .unwrap()
            .to_string();

        let mutation_root_name = schema.as_object().unwrap()["data"].as_object().unwrap()
            ["__schema"]
            .as_object()
            .unwrap()["mutationType"]
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
                _ => continue,
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
                            .map(|field_def| {
                                let field_name = field_def.as_object().unwrap()["name"].to_string();
                                Field { name: field_name }
                            })
                            .collect();

                        Some(Type::Object(ObjectType { name, fields }))
                    }
                    _ => None,
                }
            })
            .collect()
    }
}
