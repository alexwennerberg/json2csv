# json2csv

**ALPHA**

## Introduction

I love using the Javascript [json2csv](https://github.com/zemirco/json2csv) library. It's a great tool for following the [Data Science at the Command Line](https://www.datascienceatthecommandline.com/) philosophy. I wanted to attempt a rewrite in Rust in order to improve the performance and add some features I have missed.

```
json2csv 0.1.0
Alex Wennerberg <alex@alexwennerberg.com>
Converts JSON into CSV

USAGE:
    json2csv [FLAGS] [OPTIONS] [--] [INPUT]

FLAGS:
    -F, --flatten    Flatten nested jsons and arrays
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --fields <fields>...       Optionally specify fields to include
    -o, --output <output>          Output file. If not present, writes to stdout
    -U, --unwind-on <unwind-on>    Unwind an array into multiple keys, similar to mongo

ARGS:
    <INPUT>    Input file. If not present, reads from stdin
```

## Installation

This tool is still in alpha. Install from cargo:

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

If your json structure is not uniform, you may want to run `json2csv input.json --get-headers` first, which will list all the headers. Then pass these headers like `json2csv input.json --fields foo bar`

If your json is nested, you may want to use [jq](https://stedolan.github.io/jq/) to do some pre-processing. `--flatten` will flatten all nested arrays in a json, such that they will have the format field.nested_field or field.nested_array.0, etc. Combine this with `--get-headers` to get all nested values. 

You can use the `--unwind-on` option to "unwind" the json on a key. That is to say, to split a record that contains an array into an array of records containing each value of that array.

I don't include any of the formatting options that are present in the Javascript json2csv. This is following the Do One Thing and Do It Well principle -- this should just convert JSON to CSV, for any sort of reformatting or post-processing you can pipe the data into BurntSushi's excellent [xsv](https://github.com/BurntSushi/xsv) library. 

Submit a pull request for any feature requests!
