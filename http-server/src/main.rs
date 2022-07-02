mod handler;
mod render;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use pcloud::http::HttpClientBuilder;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    // The host to bind the server
    #[clap(long, default_value = "localhost", env = "HOST")]
    host: String,
    // The port to bind the server
    #[clap(long, default_value = "8080", env = "PORT")]
    port: u16,
}

impl Config {
    pub fn binding(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Config::parse();
    let client = HttpClientBuilder::from_env()
        .build()
        .expect("couldn't build client");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(handler::RootFolder::from_env()))
            .service(web::resource("{tail:.*}").to(handler::handle))
    })
    .bind(args.binding())?
    .run()
    .await
}
