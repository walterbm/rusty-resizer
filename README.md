# Rusty Resizer

![CI](https://github.com/walterbm/rusty-resizer/actions/workflows/ci.yml/badge.svg)

## About

An _experimental_ image resizing http service that wraps [ImageMagick](https://imagemagick.org) to provide safe & accurate dynamic image resizing. The best way to utilize the Rusty Resizer is through a Docker container and behind a CDN.

## Building

### From Docker Image

The Rusty Resizer is wrapped in a minimal Docker container (image size is less than < 100MBs) that can be easily mounted as a standalone service:

```
docker run -p 8080:8080 --env ALLOWED_HOSTS=raw.githubusercontent.com ghcr.io/walterbm/rusty-resizer:latest
```

### From Source

0. [Install Rust](https://www.rust-lang.org/tools/install)
1. [Install ImageMagick](https://imagemagick.org/script/download.php)
2. Install Dependencies
   ```sh
   cargo install --path .
   ```
3. Start the Server
   ```sh
   cargo run
   ```

## Usage

Start the Rusty Resizer server (either through Docker or with Cargo). By default the server will start on port `8080`.

The server only exposes two endpoints:

1. `/resize` to resize images
2. `/ping` as a health check

Once the server is running images can be dynamically resized through the `/resize` endpoint. For example:

```sh
curl localhost:8080/resize?source=image.jpeg&height=100&width=100&quality=85&format=webp
```

`/resize` accepts four query parameters:

- `source`: **required** to specify the full url of the target image
- `height` & `width`: the resized image's dimensions (if `height` or `width` are alone the other dimension is computed to preserve the aspect ratio)
- `quality`: optionally set the compression quality for image formats that accept compression (e.g. jpeg)
- `format`: convert the source to another format during the resize operation (e.g. png -> jpeg) and if set to `format=auto` attempt to automatically convert the source image to `WebP` based on client's `Accept` header

## Configuration

The Rusty Resizer accepts all its configuration options through ENV variables:

| ENV var                  | description                                                                            | default    |
| ------------------------ | -------------------------------------------------------------------------------------- | ---------- |
| `ALLOWED_HOSTS`          | **required** list of image hosts that will be accepted for resizing                    |            |
| `DEFAULT_QUALITY`        | default compression quality for image formats that accept compression (e.g. jpeg)      | 85         |
| `CACHE_EXPIRATION_HOURS` | used to populate `Cache-Control` & `Expires` headers in the final resized response     | 2880 hours |
| `CACHE_JITTER_SECONDS`   | help give `Cache-Control` & `Expires` headers some variance to avoid a thundering herd | 0          |
| `STATSD_HOST`            | StatsD host to accept metric data (metrics are only emitted when this is present)      |            |
| `WORKERS`                | number of HTTP workers                                                                 | 4          |
| `PORT`                   | TCP port to bind the server                                                            | 8080       |
| `ENV`                    | environment the server is running in                                                   | local      |

## Security

For security, and to mitigate some of the worst [vulnerabilities in ImageMagick](https://imagetragick.com/), the Rusty Resizer requires an `$ALLOWED_HOSTS` ENV variable.

Only images originating from hosts explicitly listed in `$ALLOWED_HOSTS` will be accepted by the Rusty Resizer. For example to make sure the following request works as expected `ALLOWED_HOSTS` should be set to `raw.githubusercontent.com`:

```sh
curl localhost:8080/resize?source=https://raw.githubusercontent.com/image.jpeg&height=100&width=100
```

## Deployment

For best results deploy the Rusty Resizer behind a CDN to help amortize the cost of resizing an image. If the CDN respects standard cache headers the cache time for the the resized images can be controlled through the `CACHE_EXPIRATION_HOURS` ENV option.

If using automatic content negotiation with the `format=auto` parameter make sure the CDN in front of the the Rusty Resizer respects the outgoing `Vary` header and/or can be configured to incorporate the incoming `Accept` header into the cache key. To optimize cache performance add some pre-processing to the CDN to normalize the `Accept` header.

## Test

Run the test suite with:

```sh
cargo test
```

## License

Distributed under the MIT License. See `LICENSE` for more information.
