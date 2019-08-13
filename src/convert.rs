extern crate csv;
use serde_json::{json, Deserializer, Value};
use std::collections::HashSet;
use std::error::Error;
use std::io::{BufRead, Write};
use std::str;

mod unwind_json;

// I misunderstand public structs I think
// TODO remove config object
pub struct Config {
    pub flatten: bool,
    pub unwind_on: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            flatten: false,
            unwind_on: None,
        }
    }
}

// TODO break up this function. use that function that returns self pattern for configuration
// instead of config struct

// TODO: allow unwind_on for multipleitems
// TODO Return result
pub fn get_headers(mut rdr: impl BufRead, config: &Config) -> HashSet<String> {
    // TODO DRY this
    let stream = Deserializer::from_reader(&mut rdr)
        .into_iter::<Value>()
        .flat_map(|item| preprocess(item.unwrap(), config.flatten, &config.unwind_on));
    let mut headers = HashSet::new();
    for item in stream {
        for key in item.as_object().unwrap().keys() {
            headers.insert(key.to_string());
        }
    }
    headers
}

pub fn write_json_to_csv(
    mut rdr: impl BufRead,
    wtr: impl Write,
    fields: Option<Vec<&str>>,
    config: &Config,
) -> Result<(), Box<Error>> {
    let mut csv_writer = csv::WriterBuilder::new()
        .from_writer(wtr);
    let mut stream = Deserializer::from_reader(&mut rdr)
        .into_iter::<Value>()
        .flat_map(|item| preprocess(item.unwrap(), config.flatten, &config.unwind_on));
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
            output.push(item);
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
//
#[cfg(test)]
mod test {
    use super::*;

    fn run_test(input: &str, expected: &str, config: &Config) {
        let mut sample_json = input.as_bytes();
        let mut output = Vec::new();
        write_json_to_csv(sample_json, &mut output, None, config).unwrap();
        let str_out = str::from_utf8(&output).unwrap();
        assert_eq!(str_out, expected)
    }

    #[test]
    fn simple_test() {
        run_test(r#"{ "a": 1 }"#, "a\n1\n", &Config::default())
    }

    #[test]
    fn test_first_row_params_only() {
        run_test(
            r#"{ "a": 1, "b": 2}
            {"a": 3, "c": 2}"#,
            "a,b\n1,2\n3,\n",
            &Config::default(),
        )
    }

    #[test]
    fn test_no_header() {
        let mut config = Config::default();
        config.no_header = true;
        run_test(r#"{"a":1}"#, "1\n", &config)
    }

    #[test]
    fn test_flatten() {
        let mut config = Config::default();
        config.flatten = true;
        run_test(r#"{"b": {"nested": {"A": 2}}}"#, "b.nested.A\n2\n", &config);
        run_test(r#"{"array": [1,2] }"#, "array.0,array.1\n1,2\n", &config)
    }

    #[test]
    fn test_delimiter() {
        let mut config = Config::default();
        config.delimiter = b'|';
        run_test(r#"{"a":1, "b": 2}"#, "a|b\n1|2\n", &config)
    }
}
