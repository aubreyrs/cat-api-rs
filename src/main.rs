use actix_web::{web, App, HttpResponse, HttpServer, Responder, get};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Serialize)]
struct CatPicture {
    id: usize,
    name: String,
    url: String,
}

async fn get_cat_picture(file_name: String) -> Result<Vec<u8>, HttpResponse> {
    let path = PathBuf::from("cats").join(file_name);
    let mut file = File::open(&path).await.map_err(|e| {
        HttpResponse::InternalServerError().body(format!("errowr : {}", e))
    })?;

    let mut contents = vec![];
    file.read_to_end(&mut contents).await.map_err(|e| {
        HttpResponse::InternalServerError().body(format!("errowr: {}", e))
    })?;
    Ok(contents)
}

#[get("/cat")]
async fn cat_pictures(cat_map: web::Data<Mutex<HashMap<usize, String>>>) -> impl Responder {
    let paths = tokio::fs::read_dir("cats").await.unwrap();
    let mut cat_pictures: Vec<CatPicture> = Vec::new();

    let mut dir = paths;
    let mut id = 0;
    let mut map = cat_map.lock().unwrap();
    while let Some(entry) = dir.next_entry().await.unwrap() {
        let file_name = entry.file_name().into_string().unwrap();
        map.insert(id, file_name.clone());
        cat_pictures.push(CatPicture {
            id,
            name: file_name.clone(),
            url: format!("/cat/{}", id),
        });
        id += 1;
    }

    HttpResponse::Ok().json(cat_pictures)
}

#[get("/cat/{id}")]
async fn cat_picture(id: web::Path<usize>, cat_map: web::Data<Mutex<HashMap<usize, String>>>) -> impl Responder {
    let map = cat_map.lock().unwrap();
    if let Some(file_name) = map.get(&id.into_inner()) {
        match get_cat_picture(file_name.to_string()).await {
            Ok(contents) => HttpResponse::Ok().content_type("image/jpeg").body(contents),
            Err(err) => err,
        }
    } else {
        HttpResponse::NotFound().body("file not found 3:")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cat_map: web::Data<Mutex<HashMap<usize, String>>> = web::Data::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(cat_map.clone())
            .service(cat_pictures)
            .service(cat_picture)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
