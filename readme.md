# json2csv

## Introduction

I love using the https://github.com/zemirco/json2csv library. It's a great tool for following the [Data Science at the Command Line](https://www.datascienceatthecommandline.com/) philosophy. I wanted to attempt a rewrite in Rust in order to improve the performance and add some features I've been missing.

```
json2csv 0.1.0
Alex Wennerberg <alex@alexwennerberg.com>
Converts JSON into CSV

USAGE:
    json2csv [FLAGS] [OPTIONS] [--] [INPUT]

FLAGS:
    -F, --flatten        Flatten nested jsons and arrays
    -g, --get-headers    Read input and list all headers present only
    -h, --help           Prints help information
    -H, --no-header      Exclude the header from the output
    -V, --version        Prints version information

OPTIONS:
    -d, --delimiter <delimiter>    Output csv delimiter. Must be a single ASCII character. [default: ,]
    -f, --fields <fields>...       Optionally specify fields to include
    -o, --output <output>          Output file. If not present, writes to stdout

ARGS:
    <INPUT>    Input file. If not present, reads from stdin
```

## Installation

Install via [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).


```bash
cargo install json2csv
```

Or build from source:

```bash
git clone git://github.com/alexwennerberg/json2csv
cd json2csv
cargo build --release
```

Note that if you are using multiple tools named json2csv, you'll want to uninstall one of them or add an alias, such as json2csv-rs

## Usage

For simple, flat jsons with a uniform structure, simply run `json2csv input.json`

If your json structure is not uniform, you may want to run `json2csv input.json --get-headers` first, which will list all the headers, newline-delimited. Then pass these headers like `json2csv input.json --fields foo bar`

If your json is nested, you may want to use [jq](https://stedolan.github.io/jq/) to do some pre-processing. `--flatten` will flatten all nested arrays in a json, such that they will have the format field.nested_field or field.nested_array.0, etc. Combine this with `--get-headers` to get all nested values.

Submit a pull request for any feature requests!

## Benchmarks

TBD.

Thanks to Andrew Gallant's incredible work on [rust-csv](https://github.com/BurntSushi/rust-csv) and [xsv](https://github.com/BurntSushi/xsv), which provided a lot of tools and inspiration!

