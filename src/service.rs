use std::collections::HashMap;
use std::io;
use std::iter::Map;
use std::net::TcpListener;
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use serde::Deserialize;
use crate::routes::*;

pub struct Service {}
impl Service {
    pub fn new(listener : TcpListener) -> io::Result<Server> {
        let server = HttpServer::new( || {
            App::new()
                .route("/status", web::get().to(greet))
                .route("/docs", web::post().to(addDoc))
        }).listen(listener)?
            .run();
        Ok(server)
    }
}