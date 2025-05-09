# pCloud API Client Library

[![tests](https://github.com/jdrouet/pcloud/actions/workflows/testing.yml/badge.svg)](https://github.com/jdrouet/pcloud/actions/workflows/testing.yml)

This library provides a client for interacting with the pCloud API, implementing the [HTTP Json protocol](https://docs.pcloud.com/protocols/http_json_protocol/).
It offers a set of tools and utilities to facilitate operations with files, folders, and streaming services on pCloud.

## Features

- **File Management**: Upload, download, and manage files in your pCloud account.
- **Folder Management**: Create, rename, move, and delete folders.
- **Streaming**: Stream audio and video content, with customizable parameters like bit rate and resolution.
- **Error Handling**: Robust error handling and response parsing from the pCloud API.
- **Flexible Request Building**: Customize API requests with various parameters such as content type, speed limits, and more.

## Example Usage

Below is an example of how to use the library to interact with the pCloud API:

```rust
use pcloud::{Client, Credentials, Region};
use pcloud::file::FileIdentifier;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .with_credentials(Credentials::access_token("token"))
        .build()
        .unwrap();

    // Get file link with file id
    match client.get_file_link(12345).await {
        Ok(link) => println!("File link: {}", link.first_link().unwrap()),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Get file link with file path
    match client.get_file_link("/path/to/file.txt").await {
        Ok(link) => println!("File link: {}", link.first_link().unwrap()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Error Handling

The library provides comprehensive error handling. Errors from the API are returned as custom `Error` types, which can be easily matched and handled in your application. All errors implement the `Debug` and `Display` traits for easy logging and display.

## Contributing

Contributions to this project are welcome. Please feel free to submit issues or pull requests with enhancements, bug fixes, or other improvements. Ensure to write tests for new functionality and adhere to the project’s code style.

## License

This project is licensed under the MIT License.
