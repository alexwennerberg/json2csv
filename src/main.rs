extern crate clap;

use clap::{App, Arg};

use serde_json::{Deserializer, Value};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::error::Error;
use std::io::{self, Write, BufRead, BufReader};

mod convert;

// TODO: parse json array using the code :  https://github.com/serde-rs/json/commit/55f5929c852484b863641fb6f876f4dcb69b96b8

fn main() -> Result<(), Box<Error>> {
    let m = App::new("json2csv")
        .version("0.1.0")
        .author("Alex Wennerberg <alex@alexwennerberg.com>")
        .about("Converts JSON into CSV")
        .arg(Arg::with_name("INPUT").help("Input file"))
        .arg(
            Arg::with_name("get-headers")
                .help("Read input and list headers only")
                .short("g")
                .long("get-headers"),
        )
        .arg(Arg::with_name("fields")
             .help("Optionally specify fields to include")
             .short("f")
             .multiple(true)
             .long("fields"),
         )
        .get_matches();
    // TODO: specify headers via cli
    // read from stdin or file https://stackoverflow.com/a/49964042
    let mut input: Box<BufRead> = match m.value_of("INPUT") {
        Some(i) => Box::new(BufReader::new(File::open(i).unwrap())),
        None => Box::new(BufReader::new(io::stdin())),
    };
    // TODO: set csv configuration variables via command line:
    // https://docs.rs/csv/1.0.7/csv/struct.WriterBuilder.html
    // TODO: Implement these options:
    // -o --output
    // -f --fields
    // -u --unwind
    // -F --flatten
    // -S --flatten-separator
    // -H --no-header
    // -g --get-headers get all headers from the file and nothing else
    // (csv settings)

    let mut stream = Deserializer::from_reader(&mut input).into_iter::<HashMap<String, Value>>();
    // TODO: map unwind and flatten transformations here

    // read and print headers
    if m.is_present("get-headers") {
        let mut headers = HashSet::new();
        for item in stream {
            for key in item.unwrap().keys() {
                headers.insert(key.to_string());
            }
        }
        for item in headers {
            println!("{}", item)
        }
        return Ok(());
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());

    let first_item = stream.next().unwrap().unwrap();
    let headers = match m.values_of("fields") {
        Some(f) => f.collect(),
        None => first_item.keys().map(|a| a.as_str()).collect()
    };

    wtr.write_record(convert::convert_header_to_csv_record(&headers)?)?;
    wtr.write_record(convert::convert_json_record_to_csv_record(&headers, &first_item)?)?;
    for item in stream {
        wtr.write_record(convert::convert_json_record_to_csv_record(&headers, &item.unwrap())?)?;
    }
    Ok(())
}
