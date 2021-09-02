mod handler;

use actix_web::{web, App, HttpServer};
use pcloud::credentials::Credentials;
use pcloud::http::HttpClient;
use pcloud::region::Region;

fn binding() -> String {
    let host = std::env::var("HOST").unwrap_or_else(|_| "localhost".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());

    format!("{}:{}", host, port)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let client = HttpClient::new(Credentials::from_env(), Region::from_env());
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(handler::RootFolder::from_env()))
            .service(web::resource("{tail:.*}").to(handler::handle))
    })
    .bind(binding())?
    .run()
    .await
}
