use clap::{App, Arg};

use serde_json::{Deserializer, Value, json};
use std::collections::{HashSet};
use std::fs::{self,File};
use std::error::Error;
use std::io::{self,  BufRead, BufReader};

mod convert;

// TODO: parse json array using the code :  https://github.com/serde-rs/json/commit/55f5929c852484b863641fb6f876f4dcb69b96b8

fn main() -> Result<(), Box<Error>> {
    let m = App::new("json2csv")
        .version("0.1.0")
        .author("Alex Wennerberg <alex@alexwennerberg.com>")
        .about("Converts JSON into CSV")
        .arg(Arg::with_name("INPUT").help("Input file. If not present, reads from stdin"))
        .arg(Arg::with_name("output")
             .help("Output file. If not present, writes to stdout")
             .short("o")
             .long("output")
             .takes_value(true)
        )
        .arg(
            Arg::with_name("get-headers")
                .help("Read input and list all headers present only")
                .short("g")
                .long("get-headers"),
        )
        .arg(
            Arg::with_name("flatten")
            .help("Flatten nested jsons and arrays")
            .short("F")
            .long("flatten")
            )
        .arg(
            Arg::with_name("no-header")
            .help("Exclude the header from the output")
            .short("H")
            .long("no-header")
        )
        .arg(Arg::with_name("fields")
             .help("Optionally specify fields to include")
             .short("f")
             .takes_value(true)
             .multiple(true)
             .long("fields"),
         )
        .arg(Arg::with_name("delimiter")
             .help("Output csv delimiter. Must be a single ASCII character.")
             .short("d")
             .long("delimiter")
             .takes_value(true)
             .default_value(",")
             )
        .get_matches();

    // read from stdin or file https://stackoverflow.com/a/49964042
    let mut input: Box<BufRead> = match m.value_of("INPUT") {
        Some(i) => Box::new(BufReader::new(File::open(i).unwrap())),
        None => Box::new(BufReader::new(io::stdin())),
    };

    //output writer with csv settings
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(m.value_of("delimiter").unwrap().as_bytes()[0])
        .from_writer(io_writer(m.value_of("output"))?);

    // TODO: set csv configuration variables via command line:
    // https://docs.rs/csv/1.0.7/csv/struct.WriterBuilder.html
    // TODO: Implement these options:
    // -u --unwind
    // -F --flatten
    // -S --flatten-separator
    // -g --get-headers get all headers from the file and nothing else
    // additional csv settings?
    // copy docs
    // check license on json2csv
    // add license
    
    let mut stream = Deserializer::from_reader(&mut input).into_iter::<Value>()
        .map(|item| preprocess(item.unwrap(), m.is_present("flatten")));

    // TODO: map unwind and flatten transformations here
    

    // read and print headers
    if m.is_present("get-headers") {
        let mut headers = HashSet::new();
        for item in stream {
            for key in item.as_object().unwrap().keys() {
                headers.insert(key.to_string());
            }
        }
        for item in headers {
            println!("{}", item)
        }
        return Ok(());
    }
    // todo validate valid delimiter
    //

    let first_item = stream.next().unwrap();
    let headers = match m.values_of("fields") {
        Some(f) => f.collect(),
        None => first_item.as_object().unwrap().keys().map(|a| a.as_str()).collect()
    };

    if !m.is_present("no-header") {
        wtr.write_record(convert::convert_header_to_csv_record(&headers)?)?;
    }
    wtr.write_record(convert::convert_json_record_to_csv_record(&headers, &first_item)?)?;
    for item in stream {
        wtr.write_record(convert::convert_json_record_to_csv_record(&headers, &item)?)?;
    }
    Ok(())
}

fn preprocess(item: Value, flatten: bool) -> Value {
    if flatten {
        let mut flat_value: Value = json!({});
        flatten_json::flatten(&item, &mut flat_value, None, true).unwrap();
        return flat_value;
    }
    item
}

// From https://github.com/BurntSushi/xsv/blob/master/src/config.rs
fn io_writer(path: Option<&str>) -> io::Result<Box<io::Write+'static>> {
    Ok(match path {
	None => Box::new(io::stdout()),
	Some(ref p) => Box::new(fs::File::create(p)?),
    })
}
