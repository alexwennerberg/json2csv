/// Tools to convert a json to a csv
extern crate csv;
extern crate linked_hash_set;
use serde_json::{json, Deserializer, Value};
use linked_hash_set::LinkedHashSet;
use std::error::Error;
use std::io::{BufRead, Write};
use std::str;

mod unwind_json;


/// Take a reader and a writer, read the json from the reader,
/// write to the writer. Perform flatten and unwind transofmrations
/// Sorts output fields by default
pub fn write_json_to_csv(
    mut rdr: impl BufRead,
    wtr: impl Write,
    fields: Option<Vec<&str>>,
    delimiter: Option<String>,
    flatten: bool,
    unwind_on: Option<String>,
    samples: Option<u32>,
) -> Result<(), Box<Error>> {
    let mut csv_writer = csv::WriterBuilder::new()
        .delimiter(delimiter.unwrap_or(",".to_string()).as_bytes()[0])
        .double_quote(false)
        .from_writer(wtr);
    let stream = Deserializer::from_reader(&mut rdr)
        .into_iter::<Value>()
        .flat_map(|item| preprocess(item.unwrap(), flatten, &unwind_on));
    let mut detected_headers = LinkedHashSet::new();
    let mut count = 0u32;

    // cached_values stores items from stream that used to detect headers
    let mut cached_values = <Vec<serde_json::Value>>::new();

    for item in stream {
        cached_values.push(item.clone());
        count += 1;
        if count > samples.unwrap() {
            break;
        }
        for (key,_obj) in item.as_object().unwrap().iter() {
            detected_headers.insert_if_absent(key.to_string());
        }
        
    }
    let headers = match fields {
        Some(f) => f,
        None => detected_headers.iter().map(|x| x.as_str()).collect(),
    };
    csv_writer.write_record(convert_header_to_csv_record(&headers) ?)?;
    for item in cached_values{
        csv_writer.write_record(convert_json_record_to_csv_record(&headers, &item)?)?;
    }
    //free cached values
    cached_values=vec![];
    let stream = Deserializer::from_reader(&mut rdr)
        .into_iter::<Value>()
        .flat_map(|item| preprocess(item.unwrap(), flatten, &unwind_on));
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

    fn run_test(
        input: &str,
        expected: &str,
        fields: Option<Vec<&str>>,
        delimiter: Option<String>,
        flatten: bool,
        unwind_on: Option<String>,
        samples: Option<u32>,
    ) {
        let sample_json = input.as_bytes();
        let mut output = Vec::new();
        write_json_to_csv(sample_json, &mut output, fields, delimiter, flatten, unwind_on, samples).unwrap();
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
            None,
            false,
            None,
            Some(1),
        )
    }

    #[test]
    fn test_flatten() {
        run_test(
            r#"{"b": {"nested": {"A": 2}}}"#,
            "b.nested.A\n2\n",
            None,
            None,
            true,
            None,
            Some(1)
        );
        run_test(
            r#"{"array": [1,2] }"#,
            "array.0,array.1\n1,2\n",
            None,
            None,
            true,
            None,
            Some(1),
        );
    }

    #[test]
    fn test_unwind() {
        run_test(
            r#"{"b": [1,2], "a": 3}"#,
            "a,b\n3,1\n3,2\n",
            None,
            None,
            false,
            Option::from(String::from("b")),
            Some(1)
        );
    }

    #[test]
    fn test_fields() {
        run_test(
            r#"{"a": "a", "b": "b"}"#,
            "a\na\n",
            Option::from(vec!["a"]),
            None,
            false,
            None,
            Some(1),
        )
    }

    #[test]
    fn test_unwind_and_flatten() {
        run_test(
            r#"{"b": [{"c": 1},{"c": 2}], "a": {"c": 3}}"#,
            "a.c,b.c\n3,1\n3,2\n",
            None,
            None,
            true,
            Option::from(String::from("b")),
            Some(1),
        );
    }
}
