#![allow(non_snake_case)]

/// This file represents the raw structure of th yeager trace
use serde::{Deserialize, Serialize};
//use serde_json::Value;



#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerProcess {
    serviceName: String,
    tags: Vec<JaegerTag>,
}
    

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerLog {
    timestamp: u64,
    fields: Vec<JaegerTag>,
}
    

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerReference {
    pub refType: String,
    pub traceID: String,
    pub spanID: String,
}
    
#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerTag {
    pub key: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_id: String,
    pub value: serde_json::Value
}

impl JaegerTag {

    /// Extract the string-value or fail.
    pub fn get_string(&self) -> String {
        let serde_json::Value::String(val) = &self.value else {
            panic!("The key '{}' does not contain a string. Value = {:?}", self.key, self.value);
        };
        val.to_owned()
    }

    /// Extract the string-value and transform to u32 or fail.
    pub fn to_u32(&self) -> u32 {
        let Ok(val) = self.get_string().trim().parse() else {
            panic!("Can no translate key '{}' to u32 {:?}", self.key, self.value);
        };
        val
    }

    pub fn get_i32(&self) -> i32 {
        let serde_json::Value::Number(val) = &self.value else {
            panic!("The key '{}' does not contain a number (i32). Value = {:?}", self.key, self.value);
        };
        match val.as_i64() {
            Some(val) => val as i32,
            None =>  panic!("The key '{}' does not contain a number (i32). Value = {:?}", self.key, self.value)
        }
    }

}

pub type JaegerTags = Vec<JaegerTag>;
    
#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerSpan {
    pub traceID: String,
    pub spanID: String,
    pub flags: i32,
    pub operationName: String,
    pub references: Vec<JaegerReference>,
    pub startTime: u64,
    pub duration: u64,
    pub tags: JaegerTags,
    pub logs: Vec<JaegerLog>,
    pub processID: String,
    pub warnings: Option<Vec<String >>,
}
    
#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerItem {
    pub traceID: String,
    pub spans: Vec<JaegerSpan>,
    // processes: serde_json::Map<String, JaegerProcess>,
    // ERROR: the trait `Deserialize<'_>` is not implemented for `serde_json::Map<std::string::String, JaegerProcess>`
    pub processes: serde_json::Map<String, serde_json::Value>,
    pub warnings: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerError {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct JaegerTrace {
    pub data: Vec<JaegerItem>,
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
    pub errors: Option<Vec<JaegerError>>,
}



