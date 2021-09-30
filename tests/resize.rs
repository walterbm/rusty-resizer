mod support;
use std::io::Cursor;

use image::{io::Reader as ImageReader, GenericImageView};
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
async fn test_resize_requires_source_and_dimensions_query_params() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/resize?source=img.jpg", address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_client_error());

    let text = response.text().await.expect("Failed to read response text");
    assert_eq!("Query deserialize error: missing field `height`", text);
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
async fn test_resize_can_resize_an_image_and_preserve_aspect_ratio() {
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
