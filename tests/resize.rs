mod support;
use std::io::Cursor;

use support::spawn_app;
use image::{GenericImageView, io::Reader as ImageReader};

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
        .get(format!("{}/resize?source=img.jpg&height=100&width=100", address))
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

    // Act
    let response = client
        .get(format!("{}/resize?source=https://www.publicdomainpictures.net/pictures/90000/velka/giant-mexican-flag-waves.jpg&width=100&height=100", address))
        .send()
        .await
        .expect("Failed to execute request.");
        
    // Assert
    assert!(response.status().is_success());

    let bytes = response.bytes().await.expect("Failed to read response bytes");

    let image = ImageReader::new(Cursor::new(bytes)).with_guessed_format().unwrap().decode().expect("Failed to decode image");
    let (width, height) = image.dimensions();
    assert_eq!(width, 100, "width is equal to 100px");
    assert_eq!(height, 66, "height is equal to 66px");
}
