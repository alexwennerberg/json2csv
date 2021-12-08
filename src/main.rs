/// Command line interface that handles parsing input
use clap::{App, Arg};

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};

mod convert;
// TODO: parse json array using the code :  https://github.com/serde-rs/json/commit/55f5929c852484b863641fb6f876f4dcb69b96b8

fn main() -> Result<(), Box<Error>> {
    let m = App::new("json2csv")
        .version("0.1.0")
        .author("Alex Wennerberg <alex@alexwennerberg.com>")
        .about("Converts JSON into CSV")
        .arg(Arg::with_name("INPUT").help("Input file. If not present, reads from stdin"))
        .arg(
            Arg::with_name("output")
                .help("Output file. If not present, writes to stdout")
                .short("o")
                .long("output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("flatten")
                .help("Flatten nested jsons and arrays")
                .short("F")
                .long("flatten"),
        )
        .arg(
            Arg::with_name("unwind-on")
                .help("Unwind an array into multiple keys, similar to mongo")
                .short("U")
                .long("unwind-on")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("samples")
                .help("Analyze first N number of lines for header fields, default to 1")
                .short("N")
                .long("sample-lines")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fields")
                .help("Optionally specify fields to include. If not specified, pulls fields from first record.")
                .short("f")
                .takes_value(true)
                .multiple(true)
                .long("fields"),
        ).arg(
            Arg::with_name("delimiter")
                .help("Optionally specify delimiter to use. Use $'\\t' for tab. If not specified, uses comma.")
                .short("d")
                .takes_value(true)
                .long("delimiter"),
        )
        .get_matches();
    // read from stdin or file https://stackoverflow.com/a/49964042
    // TODO: Don't panic on nonexistent file
    let mut reader: Box<BufRead> = match m.value_of("INPUT") {
        Some(i) => Box::new(BufReader::new(File::open(i).unwrap())),
        None => Box::new(BufReader::new(io::stdin())),
    };

    let unwind_on = match m.value_of("unwind-on") {
        Some(f) => Option::from(String::from(f)),
        None => None,
    };
    let flatten = m.is_present("flatten");
    let writer = io_writer(m.value_of("output"))?;
    let fields = match m.values_of("fields") {
        Some(f) => Some(f.collect()),
        None => None,
    };
    //default to comma
    let delimiter = match m.value_of("delimiter") {
        Some(d) => Some(String::from(d)),
        None => None,
    };
    //default to 1
    let samples = match m.value_of("samples") {
        Some(n) => Some(n.parse::<usize>().unwrap()),
        None => Some(1),
    };
    convert::write_json_to_csv(reader, writer, fields, delimiter, flatten, unwind_on,samples)
}

// From https://github.com/BurntSushi/xsv/blob/master/src/config.rs
fn io_writer(path: Option<&str>) -> io::Result<Box<io::Write + 'static>> {
    Ok(match path {
        None => Box::new(io::stdout()),
        Some(ref p) => Box::new(fs::File::create(p)?),
    })
}
