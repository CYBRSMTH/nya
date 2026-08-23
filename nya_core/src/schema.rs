use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CORE_SCHEMA: &str = include_str!("core_schema.json");

type SchemaCollection = HashMap<String, NyaSchema>;
type NyaSchemaSteps = Vec<String>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NyaSchema {
    pub steps: NyaSchemaSteps,
}

impl NyaSchema {
    pub fn new(cmd: &str) -> Self {
        match get_schema(cmd) {
            Ok(schema) => Self {
                steps: schema,
            },
            Err(err) => { panic!("{}", err)}
        }
    }
    
    // TODO: create fn to return schema collection
    // pub fn list_schemas(&self) -> Vec<&String> {
    //     self.schemas.keys().collect()
    // }
}

fn get_schema(cmd: &str) -> Result<NyaSchemaSteps, String> {
    let mut all_schemas: SchemaCollection = SchemaCollection::new();
    let core_schema_collection: SchemaCollection = serde_json::from_str(CORE_SCHEMA)
        .map_err(|e| format!("Failed to parse {}: {}", CORE_SCHEMA, e))?;
            
    for (name, schema) in core_schema_collection {
        all_schemas.insert(name, schema);
    };
    if let Some(schema) = all_schemas.get(cmd).clone() {
        return Ok(schema.clone().steps);
    }
    Err("Get Schema(): wasn't able to successfully retrieve schema".to_string())
}

#[cfg(test)]
mod schema_tests {
    use crate::schema::NyaSchema;

    #[test]
    fn can_get_schema() {
        let found = NyaSchema::new("test_cmd");
        let steps_len: usize = 2;
        assert_eq!(found.steps.len(), steps_len);
    }
    
    #[test]
    #[should_panic]
    fn panics_for_nonexistent_schema() {
        _ = NyaSchema::new("nonexistent");
    }
}