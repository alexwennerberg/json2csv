/// Tools to convert JSON to CSV
use serde_json::{json, Deserializer, Value};
use std::collections::HashSet;
use std::error::Error;
use std::io::{BufRead, Write};

mod unwind_json;

/// Take a reader and a writer, read JSON from the reader,
/// write CSV to the writer. Supports JSON arrays and newline-delimited JSON.
/// Performs optional flatten and unwind transformations.
/// Output keys are sorted alphabetically by default.
/// When `no_headers` is true, the header row is not written.
pub fn write_json_to_csv(
    mut rdr: impl BufRead,
    mut wtr: impl Write,
    fields: Option<Vec<&str>>,
    delimiter: Option<String>,
    flatten: bool,
    unwind_on: Option<String>,
    samples: Option<u32>,
    double_quote: bool,
    no_headers: bool,
) -> Result<(), Box<dyn Error>> {
    let delim_byte = delimiter.as_deref().unwrap_or(",").as_bytes()[0];
    let samples = samples.unwrap_or(1);

    let mut stream = Deserializer::from_reader(&mut rdr)
        .into_iter::<Value>();

    let mut cached_values: Vec<Value> = Vec::new();
    let mut detected_headers: Vec<String> = Vec::new();
    let mut header_set = HashSet::new();
    let mut values_read = 0u32;

    // Phase 1: Read `samples` top-level values, expanding arrays into individual records
    while values_read < samples {
        match stream.next() {
            Some(Ok(value)) => {
                values_read += 1;
                let expanded = expand_value(value, flatten, &unwind_on);
                cached_values.extend(expanded);
            }
            Some(Err(e)) => return Err(format!("Error parsing JSON: {}", e).into()),
            None => break,
        }
    }

    // Detect headers from all cached values
    for item in &cached_values {
        match item.as_object() {
            Some(obj) => {
                for (key, _) in obj.iter() {
                    if header_set.insert(key.clone()) {
                        detected_headers.push(key.clone());
                    }
                }
            }
            None => {
                return Err(
                    "JSON input contains non-object values. Each record must be a JSON object."
                        .into(),
                );
            }
        }
    }

    // Sort headers alphabetically for predictable output
    detected_headers.sort();

    let headers: Vec<&str> = match fields {
        Some(f) => f,
        None => detected_headers.iter().map(|s| s.as_str()).collect(),
    };

    if !no_headers {
        let header_record: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        write_csv_record(&mut wtr, &header_record, delim_byte, double_quote)?;
    }

    // Write cached values
    for item in &cached_values {
        let record = build_csv_record(&headers, item);
        write_csv_record(&mut wtr, &record, delim_byte, double_quote)?;
    }

    // Continue streaming remaining values from the reader
    for result in stream {
        let value = result.map_err(|e| format!("Error parsing JSON: {}", e))?;
        let expanded = expand_value(value, flatten, &unwind_on);
        for item in expanded {
            let record = build_csv_record(&headers, &item);
            write_csv_record(&mut wtr, &record, delim_byte, double_quote)?;
        }
    }

    Ok(())
}

/// Expand a top-level JSON array into individual items, then preprocess each.
/// Non-array values are preprocessed directly.
fn expand_value(value: Value, flatten: bool, unwind_on: &Option<String>) -> Vec<Value> {
    match value {
        Value::Array(arr) => arr
            .into_iter()
            .flat_map(|v| preprocess(v, flatten, unwind_on))
            .collect(),
        _ => preprocess(value, flatten, unwind_on),
    }
}

/// Handle the flattening and unwinding of a value.
fn preprocess(item: Value, flatten: bool, unwind_on: &Option<String>) -> Vec<Value> {
    let mut container: Vec<Value> = Vec::new();
    match unwind_on {
        Some(f) => container.extend(unwind_json::unwind_json(item, f)),
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

/// Build a CSV record from a JSON object, matching the given headers.
fn build_csv_record(headers: &[&str], json_map: &Value) -> Vec<String> {
    headers
        .iter()
        .map(|header| match json_map.get(*header) {
            Some(value) => match value {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                other => other.to_string(),
            },
            None => String::new(),
        })
        .collect()
}

/// Escape a CSV value, quoting it if it contains the delimiter, quotes, or newlines.
fn escape_csv(value: &str, delimiter: u8, double_quote: bool) -> String {
    let delim = delimiter as char;
    let needs_quoting =
        value.contains(delim) || value.contains('\n') || value.contains('\r') || value.contains('"');
    if !needs_quoting {
        return value.to_string();
    }
    if double_quote {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        format!("\"{}\"", value.replace('"', "\\\""))
    }
}

/// Write a single CSV record (one line) to the writer.
fn write_csv_record(
    wtr: &mut impl Write,
    record: &[String],
    delimiter: u8,
    double_quote: bool,
) -> Result<(), Box<dyn Error>> {
    let delim = String::from(delimiter as char);
    let escaped: Vec<String> = record
        .iter()
        .map(|v| escape_csv(v, delimiter, double_quote))
        .collect();
    writeln!(wtr, "{}", escaped.join(&delim))?;
    Ok(())
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
        double_quote: bool,
        no_headers: bool,
    ) {
        let sample_json = input.as_bytes();
        let mut output = Vec::new();
        write_json_to_csv(
            sample_json,
            &mut output,
            fields,
            delimiter,
            flatten,
            unwind_on,
            samples,
            double_quote,
            no_headers,
        )
        .unwrap();
        let str_out = std::str::from_utf8(&output).unwrap();
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
            false,
            false,
        )
    }

    #[test]
    fn test_no_headers() {
        run_test(
            r#"{ "a": 1, "b": 2}
            {"a": 3, "c": 2}"#,
            "1,2\n3,\n",
            None,
            None,
            false,
            None,
            Some(1),
            false,
            true,
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
            Some(1),
            false,
            false,
        );
        run_test(
            r#"{"array": [1,2] }"#,
            "array.0,array.1\n1,2\n",
            None,
            None,
            true,
            None,
            Some(1),
            false,
            false,
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
            Some(1),
            false,
            false,
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
            false,
            false,
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
            false,
            false,
        );
    }

    // Issue #3, #8: Support JSON array input
    #[test]
    fn test_array_input() {
        run_test(
            r#"[{"a": 1, "b": 2}, {"a": 3, "b": 4}]"#,
            "a,b\n1,2\n3,4\n",
            None,
            None,
            false,
            None,
            Some(1),
            false,
            false,
        )
    }

    // Issue #11: Sort output keys
    #[test]
    fn test_sorted_keys() {
        run_test(
            r#"{"z": 1, "a": 2, "m": 3}"#,
            "a,m,z\n2,3,1\n",
            None,
            None,
            false,
            None,
            Some(1),
            false,
            false,
        )
    }

    // Issue #2: Detect headers from all cached values, not just the first
    #[test]
    fn test_samples_detect_all_headers() {
        run_test(
            r#"[{"a": 1}, {"a": 2, "b": 3}]"#,
            "a,b\n1,\n2,3\n",
            None,
            None,
            false,
            None,
            Some(1),
            false,
            false,
        )
    }

    // Issue #9: Non-object values produce clear error
    #[test]
    fn test_non_object_error() {
        let input = r#""just a string""#;
        let mut output = Vec::new();
        let result = write_json_to_csv(
            input.as_bytes(),
            &mut output,
            None,
            None,
            false,
            None,
            Some(1),
            false,
            false,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-object values"));
    }
}
