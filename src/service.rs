use std::collections::HashMap;
use serde_json::Error;
use std::io;
use std::io::ErrorKind;
use std::iter::Map;
use std::net::TcpListener;
use std::sync::Arc;
use actix_web::{App, HttpServer, web,error,HttpResponse};
use actix_web::dev::Server;
use serde::Deserialize;
use crate::routes::*;
use crate::db::DB;
use crate::Document;

pub struct Service {}
impl Service {
    pub fn new(listener : TcpListener,db:DB) -> io::Result<Server> {

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

    fn reIndex(db: &DB) -> Option<String> {
        let db = Arc::new(db);
        for documents in db.documents().into_iter()
            .filter(|e| { *(&e.is_ok_and(|dirEntry| !dirEntry.path().is_dir())) }) {
             let result = documents
                .map_err(|e| e.into_io_error().unwrap())
                .and_then(|filePath| {
                    std::fs::read(filePath.path()).and_then(|bytes| {
                        serde_json::from_str::<Document>(unsafe { std::str::from_utf8_unchecked(&bytes) }).map_err(|e| e.to_string()).and_then(|document| {
                            let file_name = filePath.path().file_name().and_then(|osStr| osStr.to_str()).unwrap();
                            index(file_name, &document, &db)
                        }).map_err(|e|io::Error::from(ErrorKind::InvalidData) )
                    })
                });

            if let Err(e ) = result {
                return Some(format!("{:?}",e))
            }

        }
        None
    }


}