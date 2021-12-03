pub mod calendar;
pub mod schedule;

fn main() {
    println!("Hello, world");
}

/*
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
//use pam_auth::Authenticator;
use pam::Authenticator;

#[get("/ok")]
async fn ok() -> impl Responder {
    HttpResponse::Ok().body("Service is up!")
}

#[get("/v1/api/calendars")]
async fn list_calendars(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(ok)
        //.service(echo)
        //.route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
*/
