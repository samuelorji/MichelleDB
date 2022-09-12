set -e

count=50
for run in {1..50}; do
    jq -c '.[]' "$1" | while read data; do
        curl -X POST -H 'Content-Type: application/json' -d "$data" http://localhost:9001/docs
    done
done