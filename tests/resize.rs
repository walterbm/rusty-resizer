mod support;
use std::io::Cursor;

use image::{guess_format, io::Reader as ImageReader, GenericImageView, ImageFormat};
use support::spawn_app;

#[actix_rt::test]
async fn test_resize_requires_source_query_params() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/resize", address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_client_error());

    let text = response.text().await.expect("Failed to read response text");
    assert_eq!("Query deserialize error: missing field `source`", text);
}

#[actix_rt::test]
async fn test_resize_returns_error_if_image_source_is_invalid() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!(
            "{}/resize?source=img.jpg&height=100&width=100",
            address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_client_error());

    let text = response.text().await.expect("Failed to read response text");
    assert_eq!("Invalid Request For Image", text);
}

#[actix_rt::test]
async fn test_resize_returns_error_if_image_host_is_not_allowed() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_image_one = "https://content.com/test.jpg";

    // Act
    let response = client
        .get(format!(
            "{}/resize?source={}&width=100&height=100",
            address, test_image_one
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_client_error());

    let text = response.text().await.expect("Failed to read response text");
    assert_eq!("Image Host Is Not Allowed", text);
}

#[actix_rt::test]
async fn test_resize_can_resize_an_image() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_image_one = "https://raw.githubusercontent.com/walterbm/rusty-resizer/main/tests/fixtures/test-image-one.jpg";

    // Act
    let response = client
        .get(format!(
            "{}/resize?source={}&width=100&height=100",
            address, test_image_one
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/jpeg",
        "content type is equal to image/jpeg"
    );
    assert_eq!(
        response.headers().get("Cache-Control").unwrap(),
        "max-age=3600",
        "cache control max age is equal to 3600"
    );

    let bytes = response
        .bytes()
        .await
        .expect("Failed to read response bytes");

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image");
    let (width, height) = image.dimensions();
    assert_eq!(width, 100, "width is equal to 100px");
    assert_eq!(height, 100, "height is equal to 100px");
}

#[actix_rt::test]
async fn test_resize_can_resize_an_image_with_only_one_dimension_and_preserve_aspect_ratio() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_image_one = "https://raw.githubusercontent.com/walterbm/rusty-resizer/main/tests/fixtures/test-image-two.jpg";

    // Act
    let response = client
        .get(format!(
            "{}/resize?source={}&width=1000",
            address, test_image_one
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/jpeg",
        "content type is equal to image/jpeg"
    );
    assert_eq!(
        response.headers().get("Cache-Control").unwrap(),
        "max-age=3600",
        "cache control max age is equal to 3600"
    );

    let bytes = response
        .bytes()
        .await
        .expect("Failed to read response bytes");

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image");
    let (width, height) = image.dimensions();
    assert_eq!(width, 1000, "width is equal to 1000px");
    assert_eq!(height, 750, "height is equal to 750px");
}

#[actix_rt::test]
async fn test_resize_can_resize_an_image_with_one_outsized_dimension_and_preserve_aspect_ratio() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_image_one = "https://raw.githubusercontent.com/walterbm/rusty-resizer/main/tests/fixtures/test-image-two.jpg";

    // Act
    let response = client
        .get(format!(
            "{}/resize?source={}&width=1000&height=9999",
            address, test_image_one
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/jpeg",
        "content type is equal to image/jpeg"
    );
    assert_eq!(
        response.headers().get("Cache-Control").unwrap(),
        "max-age=3600",
        "cache control max age is equal to 3600"
    );

    let bytes = response
        .bytes()
        .await
        .expect("Failed to read response bytes");

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image");
    let (width, height) = image.dimensions();
    assert_eq!(width, 1000, "width is equal to 1000px");
    assert_eq!(height, 750, "height is equal to 750px");
}

#[actix_rt::test]
async fn test_resize_can_noop_if_target_dimensions_are_the_same_as_target_image() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_image_one = "https://raw.githubusercontent.com/walterbm/rusty-resizer/main/tests/fixtures/test-image-one.jpg";

    // Act
    let response = client
        .get(format!(
            "{}/resize?source={}&width=2250&height=2250",
            address, test_image_one
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/jpeg",
        "content type is equal to image/jpeg"
    );
    assert_eq!(
        response.headers().get("Cache-Control").unwrap(),
        "max-age=3600",
        "cache control max age is equal to 3600"
    );

    let bytes = response
        .bytes()
        .await
        .expect("Failed to read response bytes");

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image");
    let (width, height) = image.dimensions();
    assert_eq!(width, 2250, "width is equal to 2250px");
    assert_eq!(height, 2250, "height is equal to 2250px");
}

#[actix_rt::test]
async fn test_resize_can_handle_floating_point_target_dimensions() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_image_one = "https://raw.githubusercontent.com/walterbm/rusty-resizer/main/tests/fixtures/test-image-one.jpg";

    // Act
    let response = client
        .get(format!(
            "{}/resize?source={}&width=225.0&height=225.0",
            address, test_image_one
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/jpeg",
        "content type is equal to image/jpeg"
    );
    assert_eq!(
        response.headers().get("Cache-Control").unwrap(),
        "max-age=3600",
        "cache control max age is equal to 3600"
    );

    let bytes = response
        .bytes()
        .await
        .expect("Failed to read response bytes");

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image");
    let (width, height) = image.dimensions();
    assert_eq!(width, 225, "width is equal to 225px");
    assert_eq!(height, 225, "height is equal to 225px");
}

#[actix_rt::test]
async fn test_resize_can_reduce_image_file_size_even_when_dimensions_do_not_change() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    // test image one has dimensions of 2250px x 2250px and a file size of 1723518 bytes
    let test_image_one = "https://raw.githubusercontent.com/walterbm/rusty-resizer/main/tests/fixtures/test-image-one.jpg";

    // Act
    let response = client
        .get(format!("{}/resize?source={}", address, test_image_one))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/jpeg",
        "content type is equal to image/jpeg"
    );
    assert_eq!(
        response.headers().get("Cache-Control").unwrap(),
        "max-age=3600",
        "cache control max age is equal to 3600"
    );

    let bytes = response
        .bytes()
        .await
        .expect("Failed to read response bytes");

    assert!(
        bytes.len() < 1723518,
        "response bytes should be less than original image bytes (1723518)"
    );

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image");
    let (width, height) = image.dimensions();

    assert_eq!(width, 2250, "width is equal to 2250px");
    assert_eq!(height, 2250, "height is equal to 2250px");
}

#[actix_rt::test]
async fn test_resize_can_convert_the_format_of_an_image() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    // test image one is a JPEG image
    let test_image_one = "https://raw.githubusercontent.com/walterbm/rusty-resizer/main/tests/fixtures/test-image-one.jpg";

    // Act
    let response = client
        .get(format!(
            "{}/resize?source={}&width=225&height=225&format=webp",
            address, test_image_one
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "image/webp",
        "content type is equal to image/webp"
    );
    assert_eq!(
        response.headers().get("Cache-Control").unwrap(),
        "max-age=3600",
        "cache control max age is equal to 3600"
    );

    let bytes = response
        .bytes()
        .await
        .expect("Failed to read response bytes");

    assert_eq!(guess_format(&bytes).unwrap(), ImageFormat::WebP);

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .expect("Failed to decode image");
    let (width, height) = image.dimensions();

    assert_eq!(width, 225, "width is equal to 225px");
    assert_eq!(height, 225, "height is equal to 225px");
}
