/// Tools to convert a json to a csv
extern crate csv;
use serde_json::{json, Deserializer, Value};
use std::collections::HashSet;
use std::error::Error;
use std::io::{BufRead, Write};
use std::str;

mod unwind_json;

/// Get the headers from the json, if fields are not uniform throughout. Works with
/// Unwind and flatten. Use this function and then specify fields explicitly to 
/// write_json_to_csv  if fields are not uniform
pub fn get_headers(mut rdr: impl BufRead, flatten: bool, unwind_on: Option<String>) -> HashSet<String> {
    let stream = Deserializer::from_reader(&mut rdr)
        .into_iter::<Value>()
        .flat_map(|item| preprocess(item.unwrap(), flatten, &unwind_on));
    let mut headers = HashSet::new();
    for item in stream {
        for key in item.as_object().unwrap().keys() {
            headers.insert(key.to_string());
        }
    }
    headers
}

/// Take a reader and a writer, read the json from the reader,
/// write to the writer. Perform flatten and unwind transofmrations
/// Sorts output fields by default
pub fn write_json_to_csv(
    mut rdr: impl BufRead,
    wtr: impl Write,
    fields: Option<Vec<&str>>,
    flatten: bool,
    unwind_on: Option<String>
) -> Result<(), Box<Error>> {
    let mut csv_writer = csv::WriterBuilder::new()
        .from_writer(wtr);
    let mut stream = Deserializer::from_reader(&mut rdr)
        .into_iter::<Value>()
        .flat_map(|item| preprocess(item.unwrap(), flatten, &unwind_on));
    let first_item = stream.next().unwrap();
    let headers = match fields {
        Some(f) => f,
        None => first_item
            .as_object()
            .unwrap()
            .keys()
            .map(|a| a.as_str())
            .collect(),
    };
    csv_writer.write_record(convert_header_to_csv_record(&headers)?)?;
    csv_writer.write_record(convert_json_record_to_csv_record(&headers, &first_item)?)?;
    for item in stream {
        csv_writer.write_record(convert_json_record_to_csv_record(&headers, &item)?)?;
    }
    Ok(())
}

/// Handle the flattening and unwinding of a value 
/// Note that when unwinding a large array, all the array values
/// are held in memory. This could be improved.
fn preprocess(item: Value, flatten: bool, unwind_on: &Option<String>) -> Vec<Value> {
    let mut container: Vec<Value> = Vec::new();
    match unwind_on {
        Some(f) => container.extend(unwind_json::unwind_json(item, f)), // push all items
        None => container.push(item),
    }
    if flatten {
        let mut output: Vec<Value> = Vec::new();
        for item in container {
            let mut flat_value: Value = json!({});
            flatten_json::flatten(&item, &mut flat_value, None, true).unwrap();
            output.push(flat_value);
        }
        return output;
    }
    container
}

pub fn convert_header_to_csv_record(headers: &Vec<&str>) -> Result<Vec<String>, Box<Error>> {
    let mut record = Vec::new();
    for item in headers {
        record.push(String::from(item.clone()));
    }
    Ok(record)
}

pub fn convert_json_record_to_csv_record(
    headers: &Vec<&str>,
    json_map: &Value,
) -> Result<Vec<String>, Box<Error>> {
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

#[cfg(test)]
mod test {
    use super::*;

    fn run_test(input: &str,
        expected: &str,
        fields: Option<Vec<&str>>,
        flatten: bool,
        unwind_on: Option<String>
        ) { 
        let mut sample_json = input.as_bytes();
        let mut output = Vec::new();
        write_json_to_csv(sample_json, &mut output, fields, flatten, unwind_on).unwrap();
        let str_out = str::from_utf8(&output).unwrap();
        assert_eq!(str_out, expected)
    }

    #[test]
    fn test_first_row_params_only() {
        run_test(
            r#"{ "a": 1, "b": 2}
            {"a": 3, "c": 2}"#,
            "a,b\n1,2\n3,\n",
            None,
            false,
            None
        )
    }

    #[test]
    fn test_flatten() {
        run_test(r#"{"b": {"nested": {"A": 2}}}"#, "b.nested.A\n2\n", None, true, None);
        run_test(r#"{"array": [1,2] }"#, "array.0,array.1\n1,2\n", None, true, None);
    }

    #[test]
    fn test_unwind() {
        run_test(r#"{"b": [1,2], "a": 3}"#, "a,b\n3,1\n3,2\n", None, false, Option::from(String::from("b")));
    }

    #[test]
    fn test_fields() {
        run_test(r#"{"a": "a", "b": "b"}"#, "a\na\n", Option::from(vec!("a")), false, None)
    }

    #[test]
    fn test_unwind_and_flatten() {
        run_test(r#"{"b": [1,2], "a": 3}"#, "a,b\n3,1\n3,2\n", None, true, Option::from(String::from("b")));
    }
}
