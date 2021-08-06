use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use pcloud::{entry::Entry, entry::Folder, PCloudApi};
use std::env;

fn server_host() -> String {
    env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into())
}

fn server_port() -> String {
    env::var("PORT").unwrap_or_else(|_| "3000".into())
}

fn server_address() -> String {
    format!("{}:{}", server_host(), server_port())
}

fn format_folder(folder: Folder) -> String {
    let mut body = String::new();
    if let Some(parent_id) = folder.parent_folder_id {
        body.push_str(&format!(
            "<li><a href=\"/folders/{}\">..</a></list>",
            parent_id,
        ));
    }
    let mut children = folder.contents.unwrap_or_default();
    children.sort();
    for entry in children.iter() {
        let path = match &entry {
            Entry::Folder(child) => {
                format!("/folders/{}", child.folder_id)
            }
            Entry::File(child) => {
                format!("/files/{}", child.file_id)
            }
        };
        body.push_str(&format!(
            "<li><a href=\"{}\">{}</a></list>",
            path,
            entry.name()
        ));
    }
    let index_of = format!("Index of {}", folder.path.unwrap_or(folder.name));
    let html = format!(
        "<html>\
         <head><title>{}</title></head>\
         <body><h1>{}</h1>\
         <ul>\
         {}\
         </ul></body>\n</html>",
        index_of, index_of, body
    );
    html
}

#[get("/")]
async fn redirect_root() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/folders/0"))
        .finish()
}

#[get("/folders/{folder_id}")]
async fn list_folder(pcloud: web::Data<PCloudApi>, folder_id: web::Path<usize>) -> impl Responder {
    match pcloud.list_folder(*folder_id).await {
        Ok(result) => HttpResponse::Ok().body(format_folder(result)),
        Err(err) => {
            HttpResponse::InternalServerError().json(format!("something went wrong: {:?}", err))
        }
    }
}

#[get("/files/{file_id}")]
async fn download_file(pcloud: web::Data<PCloudApi>, file_id: web::Path<usize>) -> impl Responder {
    match pcloud.get_link_file(*file_id).await {
        Ok(result) => HttpResponse::Found()
            .append_header(("Location", result.as_str()))
            .finish(),
        Err(err) => {
            HttpResponse::InternalServerError().json(format!("something went wrong: {:?}", err))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let pcloud = PCloudApi::from_env();
    let address = server_address();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pcloud.clone()))
            .service(redirect_root)
            .service(list_folder)
            .service(download_file)
    })
    .bind(&address)?
    .run()
    .await
}
