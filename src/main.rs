use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print_help();
        return Ok(());
    }
    if args.contains(["-V", "--version"]) {
        println!("json2csv 0.2.0");
        return Ok(());
    }

    let flatten = args.contains(["-F", "--flatten"]);
    let double_quote = args.contains(["-D", "--double-quote"]);
    let no_headers = args.contains(["-H", "--no-headers"]);
    let output: Option<String> = args.opt_value_from_str(["-o", "--output"])?;
    let unwind_on: Option<String> = args.opt_value_from_str(["-U", "--unwind-on"])?;
    let samples: Option<u32> = args.opt_value_from_str(["-N", "--sample-lines"])?;
    let delimiter: Option<String> = args.opt_value_from_str(["-d", "--delimiter"])?;
    let fields_str: Option<String> = args.opt_value_from_str(["-f", "--fields"])?;
    let input: Option<String> = args.opt_free_from_str()?;

    let reader: Box<dyn BufRead> = match input {
        Some(ref path) => {
            let file = File::open(path)
                .map_err(|e| format!("Error opening file '{}': {}", path, e))?;
            Box::new(BufReader::new(file))
        }
        None => Box::new(BufReader::new(io::stdin())),
    };

    let writer: Box<dyn Write> = match output {
        Some(ref path) => {
            let file = File::create(path)
                .map_err(|e| format!("Error creating output file '{}': {}", path, e))?;
            Box::new(file)
        }
        None => Box::new(io::stdout()),
    };

    let fields: Option<Vec<&str>> = fields_str
        .as_ref()
        .map(|s| s.split(',').map(|s| s.trim()).collect());

    json2csv::write_json_to_csv(
        reader,
        writer,
        fields,
        delimiter,
        flatten,
        unwind_on,
        samples.or(Some(1)),
        double_quote,
        no_headers,
    )
}

fn print_help() {
    println!(
        "json2csv 0.2.0
Converts JSON into CSV

USAGE:
    json2csv [OPTIONS] [INPUT]

ARGS:
    <INPUT>    Input file. If not present, reads from stdin

OPTIONS:
    -o, --output <FILE>          Output file. If not present, writes to stdout
    -F, --flatten                Flatten nested JSON objects and arrays
    -U, --unwind-on <FIELD>      Unwind an array into multiple rows
    -N, --sample-lines <N>       Number of values to sample for headers (default: 1)
    -f, --fields <FIELDS>        Comma-separated list of fields to include
    -d, --delimiter <DELIM>      Field delimiter (default: comma)
    -D, --double-quote           Enable double-quote escaping (RFC 4180)
    -H, --no-headers             Omit the header row from CSV output
    -h, --help                   Print help
    -V, --version                Print version"
    );
}
