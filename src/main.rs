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
            Arg::with_name("get-headers")
                .help("Read input and list all headers present only")
                .short("g")
                .long("get-headers"),
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
            Arg::with_name("no-header")
                .help("Exclude the header from the output")
                .short("H")
                .long("no-header"),
        )
        .arg(
            Arg::with_name("fields")
                .help("Optionally specify fields to include")
                .short("f")
                .takes_value(true)
                .multiple(true)
                .long("fields"),
        )
        .arg(
            Arg::with_name("delimiter")
                .help("Output csv delimiter. Must be a single ASCII character.")
                .short("d")
                .long("delimiter")
                .takes_value(true)
                .default_value(","),
        )
        .get_matches();
    // read from stdin or file https://stackoverflow.com/a/49964042
    let mut reader: Box<BufRead> = match m.value_of("INPUT") {
        Some(i) => Box::new(BufReader::new(File::open(i).unwrap())),
        None => Box::new(BufReader::new(io::stdin())),
    };

    //output writer with csv settings

    // TODO: set csv configuration variables via command line:
    // https://docs.rs/csv/1.0.7/csv/struct.WriterBuilder.html
    // TODO: Implement these options:
    // -u --unwind
    // -S --flatten-separator
    // additional csv settings?
    // TODO: map unwind and flatten transformations here
    //
    // TODO: refactor redundancy
    let csv_config = convert::Config {
        unwind_on: match m.value_of("unwind_on") {
            Some(f) => Option::from(String::from(f)),
            None => None,
        },
        no_header: m.is_present("no-header"),
        flatten: m.is_present("flatten"),
        delimiter: m.value_of("delimiter").unwrap().as_bytes()[0],
    };
    let writer = io_writer(m.value_of("output"))?;
    let fields = match m.values_of("fields") {
        Some(f) => Some(f.collect()),
        None => None,
    };
    if m.is_present("get-headers") {
        convert::get_headers(reader, &csv_config);
        return Ok(());
    }
    convert::write_json_to_csv(reader, writer, fields, &csv_config)
    // todo validate valid delimiter
}

// From https://github.com/BurntSushi/xsv/blob/master/src/config.rs
fn io_writer(path: Option<&str>) -> io::Result<Box<io::Write + 'static>> {
    Ok(match path {
        None => Box::new(io::stdout()),
        Some(ref p) => Box::new(fs::File::create(p)?),
    })
}
