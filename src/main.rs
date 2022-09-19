mod server;
mod routes;
mod db;
mod lexer;
mod service;

use std::{fmt::format, io, net::TcpListener};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use std::sync::Mutex;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use serde::Deserialize;
use server::Server;
use db::*;

use serde_json::{json, Value};

use walkdir::WalkDir;

#[tokio::main]
async fn main() -> io::Result<()> {

    let db = DB::new()?;
    let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", 9001))?;
    let server = Server::new(listener, db)?;
    server.await
}


