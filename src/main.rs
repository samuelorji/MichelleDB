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
#[tokio::main]
async fn main() -> io::Result<()> {
    let db = DB::new()?;
    let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", 9001))?;
    let server = Service::new(listener,db)?;
    server.await
}


