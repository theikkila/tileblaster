use std::io::Cursor;
use std::{thread::sleep, time::Duration};

use actix_web::{error, web, Error};
use image::{ImageBuffer, RgbaImage, Rgba};
use image::Rgb;
use image::RgbImage;
use imageproc::drawing::{draw_text_mut, draw_line_segment_mut};
use once_cell::sync::Lazy;
use rusqlite::OptionalExtension;
use rusqlite::Statement;
use rusttype::{Font, Scale};
use serde::Deserialize;
use serde::Serialize;

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    zoom_level: i32,
    tile_column: i32,
    tile_row: i32,
    pub tile_data: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileAddress {
    pub zoom: u32,
    pub x: i32,
    pub y: i32
}

#[allow(clippy::enum_variant_names)]
pub enum Queries {
    GetTile {address: TileAddress}
}

pub async fn execute(pool: &Pool, query: Queries) -> Result<Tile, Error> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;

    web::block(move || {
        // // simulate an expensive query, see comments at top of main.rs
        // sleep(Duration::from_secs(1));
        match query {
            Queries::GetTile {address} => get_tile(conn, address)
        }
    })
    .await?
    .map_err(error::ErrorInternalServerError)
}

fn get_tile(conn: Connection, address: TileAddress) -> Result<Tile, rusqlite::Error> {
    let ymax = 1 << address.zoom;
    let y = ymax - 1 - (address.y as i32);
    // println!("get_tile: {:?} ymax= {}, y= {}", address, ymax, y);
    conn.query_row("
        SELECT zoom_level, tile_column, tile_row, tile_data
        FROM tiles
        WHERE zoom_level = :zoom AND tile_column = :x AND tile_row = :y;
    ", &[
        (":zoom", &address.zoom.to_string()), 
        (":x", &address.x.to_string()), 
        (":y", &y.to_string())
        ], |row| {
        Ok(Tile {
            zoom_level: row.get(0)?,
            tile_column: row.get(1)?,
            tile_row: row.get(2)?,
            tile_data: row.get(3)?,
        })
    }).optional().map(|t: Option<Tile>| {
        t.unwrap_or_else(|| {
            Tile {
                zoom_level: address.zoom as i32,
                tile_column: address.x,
                tile_row: address.y,
                tile_data: generate_tile(address.zoom, address.x, address.y)
            }
        })
    }).map_err(|e| e.into())
}




// in Rust:
fn tile2wsg84(zoom: u32, xtile: i32, ytile: i32) -> (f64, f64) {
    let n = 1 << zoom;
    let lon_deg = xtile as f64 / n as f64 * 360.0 - 180.0;
    let lat_rad = (1.0 - 2.0 * ytile as f64 / n as f64).sinh().atan();
    let lat_deg = lat_rad.to_degrees();
    (lat_deg, lon_deg)
}

static FONT: Lazy<Font> = Lazy::new(||{
    Font::try_from_vec(Vec::from(include_bytes!("RobotoMono-Bold.ttf") as &[u8])).unwrap()
});

fn generate_tile(zoom: u32, x: i32, y: i32) -> Vec<u8> {
    let mut img: RgbaImage = ImageBuffer::from_pixel(256, 256, Rgba([255u8, 255u8, 255u8, 0u8]));
    // Fill image with gray color

    let height = 20.0;
    let scale = Scale {
        x: height * 2.0,
        y: height,
    };
    let c = Rgba([255u8, 255u8, 255u8, 255u8]);
    let (lat, lon) = tile2wsg84(zoom, x, y);
    let lat_text = format!("{:.4}N", lat);
    let lon_text = format!("{:.4}E", lon);
    draw_line_segment_mut(&mut img, (0.0, 0.0),(255.0, 0.0),c);
    draw_line_segment_mut(&mut img, (0.0, 0.0),(0.0, 255.0),c);
    draw_text_mut(&mut img, c, 50, 108, scale, &FONT, &lat_text);
    draw_text_mut(&mut img, c, 50, 128, scale, &FONT, &lon_text);
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png).expect("always success in writing");
    bytes
}