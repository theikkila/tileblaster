use std::env;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Error};
use r2d2_sqlite::{self, SqliteConnectionManager};
use actix_cors::Cors;
use futures::{future::ok, stream::once};

mod db;
use db::{Pool, Queries};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Tileblaster: simplest for serving MBtiles")
}



#[get("/{zoom}/{x}/{y}.png")]
async fn zoomtile(path_address: web::Path<db::TileAddress>, db: web::Data<Pool>) -> impl Responder {
    let address = path_address.into_inner();
    let t = db::execute(&db, Queries::GetTile { address }).await;
    if let Ok(tile) = t {
        let data = tile.tile_data;
        let body = once(ok::<_, Error>(web::Bytes::copy_from_slice(&data)));
        HttpResponse::Ok().content_type("image/png").streaming(body)
    } else {
        HttpResponse::NotFound().body("Not found")
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let map_file = args.get(1).expect("");
    let port = args.get(2).and_then(|p| {
        p.parse::<u16>().ok()
    }).unwrap_or(8080);

    let manager = SqliteConnectionManager::file(map_file);
    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(zoomtile)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}