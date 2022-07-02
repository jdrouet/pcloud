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
    // The root folder
    #[clap(long, default_value = "/", env = "ROOT_FOLDER")]
    root_folder: String,
}

impl Config {
    pub fn binding(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn root_folder(&self) -> crate::handler::RootFolder {
        crate::handler::RootFolder::new(self.root_folder.clone())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Config::parse();
    let client = HttpClientBuilder::from_env()
        .build()
        .expect("couldn't build client");
    let client = web::Data::new(client);
    let root = web::Data::new(args.root_folder());
    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .app_data(root.clone())
            .service(web::resource("{tail:.*}").to(handler::handle))
    })
    .bind(args.binding())?
    .run()
    .await
}
