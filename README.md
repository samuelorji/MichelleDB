# MichelleDB

A very simple dynamically mapped document database with lucene-like filters and indexes in Rust. 

Majorly influenced by this [blog post](https://notes.eatonphil.com/documentdb.html)

## Run
Run with Rust 1.62+
```
cargo run 
``` 
To reindex documents, run with `re_index` feature
```
cargo run --features re_index
```

## Usage
With the server running on port 9001 in a terminal, in another terminal: 

### Add document

```bash
curl -X POST -H 'Content-Type: application/json' -d '{"name": "Ogechi", "age": "45"}' http://localhost:9001/docs

{"body":{"id":"b6a921c3-eb7d-4946-ba05-c0edad0eff4c"},"status":"ok"}

```

### GET Document:
From the example above:
```bash
 curl http://localhost:9001/docs/b6a921c3-eb7d-4946-ba05-c0edad0eff4c | jq   
 
 {
  "body": {
    "body": {
      "age": "45",
      "name": "Ogechi"
    },
    "id": "b6a921c3-eb7d-4946-ba05-c0edad0eff4c"
  },
  "status": "ok"
}

```

### GET Document with filters
```bash
curl http://localhost:9001/docs?q=name:Ogechi | jq 
{
  "body": {
    "count": 1,
    "documents": [
      {
        "body": {
          "age": "45",
          "name": "Ogechi"
        },
        "id": "b6a921c3-eb7d-4946-ba05-c0edad0eff4c"
      }
    ]
  },
  "status": "ok"
}

```
By default, only indexes are used and if you want range queries e.g `<` `>` without a direct (equal) query
to skip the index and result in a scan, you can easily 
set the optional `skipIndex` query to true which will scan the table applying the range filter
```bash
curl -G --data-urlencode 'q=age:<50' --data-urlencode 'skipIndex=true' http://localhost:9001/docs | jq
{
  "body": {
    "count": 1,
    "documents": [
      {
        "body": {
          "age": "45",
          "name": "Ogechi"
        },
        "id": "b6a921c3-eb7d-4946-ba05-c0edad0eff4c"
      }
    ]
  },
  "status": "ok"
}

```

### Load test
To load in mock data, simply run:
```bash
./script.sh movies.json
```
and `Ctrl+C` when you're tired of waiting :)


[Sample implementation with mutexes](https://github.com/samuelorji/MichelleDB/pull/1)