use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Error};
use r2d2_sqlite::{self, SqliteConnectionManager};
use actix_cors::Cors;
use futures::{future::ok, stream::once};

mod db;
use db::{Pool, Queries};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}



#[get("/{zoom}/{x}/{y}.png")]
async fn zoomtile(path_address: web::Path<db::TileAddress>, db: web::Data<Pool>) -> impl Responder {
    // let fut_result = vec![
    //     db::execute(&db, Queries::GetTile)
    // ];
    // let result: Result<Vec<_>, _> = futures::future::join_all(fut_result).await.into_iter().collect();
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

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = SqliteConnectionManager::file("/Users/theikkila/code/github/mbutil/tilesets/loisto.mbtiles");
    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(zoomtile)
            .service(echo)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}