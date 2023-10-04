# Tileblaster

Tileblaster is server for serving (raster) MBTiles formatted maps. It is written in Rust and uses the [actix-web](https://actix.rs/) framework.


## Installation

### From source

1. Install Rust and Cargo (https://www.rust-lang.org/tools/install)
2. Clone this repository
3. Run `cargo build --release`
4. The binary will be in `target/release/tileblaster`

### Usage

```
USAGE:
    tileblaster <mbtiles> <port>

Example:
    tileblaster mymap.mbtiles 8080
```

## License

Tileblaster is licensed under the MIT license. See the [LICENSE](LICENSE) file for more information.