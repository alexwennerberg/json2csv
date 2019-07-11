use serde_json::{json, Deserializer, Value};

/// Takes a json and "Unwinds" it, based roughly on the behavior of
/// https://docs.mongodb.com/manual/reference/operator/aggregation/unwind/
/// This means, select an key, for each item in that array, return a new element
/// where the value of that key.
pub fn unwind_json(wound_json: Value, unwind_on: &String) -> Vec<Value> {
    // split unwind_on by . (or some other value, potentially specified)
    // go through the wound_json looking for the value
    // for every item in the array, clone the array
    // then replace the value of the map with that item from 
    // the array
    let mut output = Vec::new();
    output.push(wound_json);
    output
}
