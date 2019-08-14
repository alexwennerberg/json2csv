set -x
time json2csv -i sample.json > /dev/null
time json2csv -i sample.json --flatten > /dev/null
time json2csv -i sample.json --unwind "tags" > /dev/null
time json2csv -i sample.json --unwind "tags" --flatten > /dev/null
time json2csv -i sample.json --fields "guid" > /dev/null
