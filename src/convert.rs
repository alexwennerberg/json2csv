extern crate csv;

use serde_json::{Deserializer, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self};

// TODO: move code in main here

// TODO: implement flatten and unwind
//
fn flatten_record() {}

fn unwind_record() {}

pub fn convert_header_to_csv_record(headers: &Vec<&str>) -> Result<Vec<String>, Box<Error>> {
    let mut record = Vec::new();
    for item in headers {
        record.push(String::from(item.clone()));
    }
    Ok(record)
}

pub fn convert_json_record_to_csv_record(
    headers: &Vec<&str>,
    json_map: &HashMap<String, Value>,
) -> Result<Vec<String>, Box<Error>>{
    // todo move writer away from this function
    // iterate over headers
    // if header is present in record, add it
    // if not, blank string
    let mut record = Vec::new();
    for item in headers {
        let value = json_map.get(&item.to_string());
        let csv_result = match value {
            Some(header_item) => match header_item.as_str() {
                Some(s) => String::from(s),
                None => header_item.to_string(),
            },
            None => String::from(""),
        };
        record.push(csv_result)
    }
    Ok(record)
}

// TODO: add tests
