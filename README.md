# Rusty Resizer

## About

An _experimental_ image resizing http service written in Rust aiming for high concurrency with low memory usage.

### Installation

0. [Install Rust](https://www.rust-lang.org/tools/install)
1. [Install ImageMagick](https://imagemagick.org/script/download.php)
2. Install Dependencies
   ```sh
   cargo install
   ```

## Usage

Currently this can only be build and run using Cargo. To start the server run:

```sh
cargo run
```

By default the server will start on port `8080` but the port can be changed by setting the `$PORT` ENV variable before starting the server.

Once the server is running images can be dynamically resized through the `/resize` endpoint. For example:

```
curl localhost::8080/resize?source=image.jpeg&height=100&width=100&quality=85
```

## Test

Run the (limited) test suite with:

```sh
cargo test
```

## License

Distributed under the MIT License. See `LICENSE` for more information.
