use std::collections::HashMap;
use std::io;
use std::iter::Map;
use std::net::TcpListener;
use actix_web::{App, HttpServer, web,error,HttpResponse};
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
                .service(getDoc)
                .app_data(db.clone())
                // .app_data(web::QueryConfig::default()
                //     // .error_handler(err, _req| {
                //     //     // create custom error response
                //     //     error::InternalError::from_response(err, HttpResponse::Conflict().finish())
                //     //         .into());
                //     .error_handler(|err,req|{
                //         let errMsg = format!("Expected Query parameter {:?}",&err);
                //         error::InternalError::from_response(err, HttpResponse::BadRequest()
                //             .body(errMsg)).into()
                //     }))
        }).listen(listener)?
            .run();
        Ok(server)
    }
}