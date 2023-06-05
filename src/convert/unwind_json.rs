use serde_json::{json, Value};

/// Takes a json and "Unwinds" it, based roughly on the behavior of
/// https://docs.mongodb.com/manual/reference/operator/aggregation/unwind/
/// This means, select an key, for each item in that array, return a new element
/// where the value of that key.
///
/// Currently holds everything in memory. When generators are moved into Stable,
/// maybe implement those
///
/// Does not handle nested arrays -- the unwound key must be at the root of the json map
/// Currently clones the json Value many times -- this is a place performance could 
/// be improved
pub fn unwind_json(wound_json: Value, unwind_on: &String) -> Vec<Value> {
    let mut output = Vec::new();
    let sub_array = wound_json.get(unwind_on).unwrap().as_array().unwrap();
    let mut wound_json_minus_unwind_key = wound_json.as_object().unwrap().clone();
    wound_json_minus_unwind_key.remove(unwind_on);
    for item in sub_array {
        let mut new_json = wound_json_minus_unwind_key.clone();
        new_json.insert(unwind_on.clone(), item.clone());
        output.push(Value::from(new_json));
    }
    output
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_test() {
        assert_eq!(unwind_json(json!({"a": 1, "b": [1,2]}), &String::from("b")), 
            *json!([{"a": 1, "b": 1}, {"a": 1, "b": 2}]).as_array().unwrap())
    }

}
