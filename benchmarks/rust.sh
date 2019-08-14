set -x
time ../target/release/json2csv sample.json > /dev/null
time ../target/release/json2csv sample.json --flatten > /dev/null
time ../target/release/json2csv sample.json --unwind-on "tags" > /dev/null
time ../target/release/json2csv sample.json --unwind-on "tags" --flatten > /dev/null
time ../target/release/json2csv sample.json --fields "guid" > /dev/null
