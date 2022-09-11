mod service;
mod routes;
mod db;
mod lexer;

use std::{fmt::format, io, net::TcpListener};
use std::collections::HashMap;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use serde::Deserialize;
use service::Service;
use db::*;

use serde_json::json;

// fn main() {
//     // The type of `john` is `serde_json::Value`
//     let john = json!({
//         "name": "John Doe",
//         "age": 43,
//         "phones": [
//             "+44 1234567",
//             "+44 2345678"
//         ]
//     });
//     let res = &john["phoness"][0];
//
//     println!("first phone number: {}", john["phoness"][0]);
//
//     // Convert to a string of JSON and print it out
//     println!("{}", john.to_string());
//
//  }
#[tokio::main]
async fn main() -> io::Result<()> {
    let db = DB::new()?;
    let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", 9001))?;
    let server = Service::new(listener,db)?;
    server.await
}


