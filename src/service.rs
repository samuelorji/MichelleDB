use std::collections::HashMap;
use std::io;
use std::iter::Map;
use std::net::TcpListener;
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use serde::Deserialize;
use crate::routes::*;
use crate::db::DB;

pub struct Service {}
impl Service {
    pub fn new(listener : TcpListener,db:DB) -> io::Result<Server> {

        let db = web::Data::new(db);
        let server = HttpServer::new(move  || {
            App::new()
                .route("/status", web::get().to(greet))
                .service(addDoc)
                .service(getById)
                .app_data(db.clone())
        }).listen(listener)?
            .run();
        Ok(server)
    }
}