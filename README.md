# Rusty Resizer

## About

An _experimental_ image resizing http service that wraps [ImageMagick](https://imagemagick.org) to provide safe & accurate dynamic image resizing. The best way to utilize the Rusty Resizer is through a Docker container.

## Building

### From Docker Image

The Rusty Resizer is wrapped in an _extremely_ minimal Docker container (image size is less than < 100MBs) that can be easily mounted as a standalone service:

```
docker build rusty-resizer:latest
docker run -p 8080:8080 --env ALLOWED_HOSTS=raw.githubusercontent.com rusty-resizer
```

### From Source

0. [Install Rust](https://www.rust-lang.org/tools/install)
1. [Install ImageMagick](https://imagemagick.org/script/download.php)
2. Install Dependencies
   ```sh
   cargo install
   ```
3. Start the Server
   ```sh
   cargo run
   ```

## Usage

Start the Rusty Resizer server (either through Docker or with Cargo). By default the server will start on port `8080`.

Once the server is running images can be dynamically resized through the `/resize` endpoint. For example:

```sh
curl localhost:8080/resize?source=image.jpeg&height=100&width=100&quality=85
```

For security, and to mitigate some of the worst [vulnerabilities in ImageMagick](https://imagetragick.com/), the Rusty Resizer requires an `$ALLOWED_HOSTS` ENV variable. Only images originating from hosts explicitly listed in `$ALLOWED_HOSTS` will be accepted by the Rusty Resizer. For example to make sure the following request works as expected `ALLOWED_HOSTS` should be set to `raw.githubusercontent.com`:

```sh
curl localhost:8080/resize?source=https://raw.githubusercontent.com/image.jpeg&height=100&width=100&quality=85
```

## Test

Run the test suite with:

```sh
cargo test
```

## License

Distributed under the MIT License. See `LICENSE` for more information.
