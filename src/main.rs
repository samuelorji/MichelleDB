mod service;
mod routes;
mod db;
mod lexer;

use std::{fmt::format, io, net::TcpListener};
use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use serde::Deserialize;
use service::Service;
use db::*;

use serde_json::{json, Value};

use walkdir::WalkDir;

// fn main() {
//     for entry in WalkDir::new("/Users/samuelorji/.michelleDB").into_iter()
//         .filter(|e| {
//             if let Ok(entry) = e {
//                 !Path::new(entry.path()).is_dir()
//             }  else {
//                 false
//             }
//         }) {
//         println!("{}", entry.unwrap().path().display());
//     }
// }

fn main() {
    // The type of `john` is `serde_json::Value`
    let john = json!({
        "name": "John Doe",
        "age": 43,
        "star": {
            "begin" :"yes",
            "end": "yes"
        },
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });
     let stuff = ["star", "begin"];
      let mut res= &Value::Null;
    for item in stuff {
        if(res.is_null()) {
            res = &john[item]
        } else {
            res = &res[item]
        }
    }
//
//
//
   // let start = &john[stuff[0]];
    //let res = &john["star"]["begin"];
    //let res = john.index("star").index("begin");

    println!("first phone number: {}",res);

    // Convert to a string of JSON and print it out
    println!("{}", john.to_string());

 }
//
// #[tokio::main]
// async fn main() -> io::Result<()> {
//     let db = DB::new()?;
//     let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", 9001))?;
//     let server = Service::new(listener,db)?;
//     server.await
// }


