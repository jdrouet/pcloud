mod handler;
mod render;

use actix_web::{web, App, HttpServer};
use clap::{crate_authors, crate_description, crate_version, Parser};
use pcloud::credentials::Credentials;
use pcloud::http::HttpClient;
use pcloud::region::Region;

#[derive(Parser)]
#[clap(about = crate_description!(), author = crate_authors!(), version = crate_version!())]
struct Command {
    #[clap(
        long,
        env = "PCLOUD_USERNAME",
        about = "Username to connect to Pcloud."
    )]
    pcloud_username: String,
    #[clap(
        long,
        env = "PCLOUD_PASSWORD",
        about = "Password to connect to Pcloud."
    )]
    pcloud_password: String,
    #[clap(
        long,
        env = "PCLOUD_REGION",
        default_value = "eu",
        about = "Region to connect to Pcloud."
    )]
    pcloud_region: String,

    #[clap(
        long,
        env = "HOST",
        default_value = "localhost",
        about = "Host to bind the server."
    )]
    host: String,
    #[clap(
        long,
        env = "PORT",
        default_value = "8080",
        about = "Port to bind the server."
    )]
    port: String,
}

impl Command {
    fn client(&self) -> HttpClient {
        let region = Region::from_name(&self.pcloud_region).expect("Invalid region");
        let creds = Credentials::UserPassword {
            username: self.pcloud_username.clone(),
            password: self.pcloud_password.clone(),
        };
        HttpClient::new(creds, region)
    }

    fn binding(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let cmd = Command::parse();

    let client = cmd.client();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(handler::RootFolder::from_env()))
            .service(web::resource("{tail:.*}").to(handler::handle))
    })
    .bind(cmd.binding())?
    .run()
    .await
}
