use std::collections::HashMap;
use serde_json::Error;
use std::io;
use std::io::ErrorKind;
use std::iter::Map;
use std::net::TcpListener;
use std::sync::Arc;
use actix_web::{App, HttpServer, web,error,HttpResponse};
use actix_web::dev::{ Server as WebServer};
use serde::Deserialize;
use crate::routes::*;
use crate::db::DB;
use crate::Document;
use crate::service::Service;

pub struct Server {}
impl Server {
    pub fn new(listener : TcpListener,db:DB) -> io::Result<WebServer> {

        #[cfg(feature = "re_index")] {
            println!("reindexing");
            if let Some(e) = Service::reIndex(&db) {
                println!("{}",e);
            }
        }
        let db = web::Data::new(db);
        let server = HttpServer::new(move || {
            App::new()
                .route("/status", web::get().to(greet))
                .service(addDoc)
                .service(getById)
                .service(getDoc)
                .app_data(db.clone())
        }).listen(listener)?
            .run();
        Ok(server)
    }




}