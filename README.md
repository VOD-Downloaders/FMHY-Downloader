# VOD Downloader

A docker container for downloading VODs off of certain freemediaheckyeah sites.

> [!WARNING]
> This software is currently in alpha stages, there may be bugs and breaking changes to the API.

## Features

- VOD Indexing from various [freemediaheckyeah](https://fmhy.net/video) sites. // TODO
- (Bulk) VOD Downloading from same [freemediaheckyeah](https://fmhy.net/video) sites.
- Easy navigatable WebUI
- VOD Validation // TODO
- Prowlarr indexer API // TODO

## Installation

// TODO: ...

## Usage

// TODO: ...

## API

The docker container exposes an HTTP server with callable API functions, listed below:

| Type | Endpoint | Description | Input | Output |
|---|---|---|---|---|
| `POST` | `/api/download` | Start VOD download | { input_url, output_file } | { TODO } |
| `GET` | `/api/downloadStatus/{id}` | Retrieve status of running download | - | { TODO } |

## Contributing

Contributions are highly appreciated (especially to the [fronted](#fronted-html-css-js) and [documentation](#documentation-markdown)).  

### Backend (Rust, Docker)

To contribute to the backend follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-change`)
3. Make your changes and ensure everything compiles (`cargo build` && `docker build .`)
4. Run tests (`cargo test`)
5. Run the linter (`cargo clippy`)
6. Format your code (`cargo +nightly fmt` from [`rustfmt`](https://github.com/rust-lang/rustfmt))
7. Open a pull request with a clear description of what you changed and why

### Fronted (HTML, CSS, JS)

Contributions to the WebUI are highly appreciated.  
To contribute to the fronted follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-change`)
3. Make your changes
4. Open a pull request with a clear description of what you changed and why

### Documentation (Markdown)

Contributions to the Documentation are highly appreciated, add your files to the [`doc/`](./doc) folder.

## Third-Party Libraries

| Crate | Version | License | Purpose |
|---|---|---|---|
| [colored](https://crates.io/crates/colored) | 3.1 | MPL-2.0 | Coloured printing |
| [chrono](https://crates.io/crates/chrono) | 0.4 | MIT / Apache-2.0 | Timestamp handling |
| [tokio](https://crates.io/crates/tokio) | 1 | MIT | A runtime for writing reliable asynchronous applications |
| [serde](https://crates.io/crates/serde) | 1 | MIT / Apache-2.0 | Serialization/deserialization framework |
| [serde_json](https://crates.io/crates/serde_json) | 1 | MIT / Apache-2.0 | JSON parsing for API payloads |
| [axum](https://crates.io/crates/axum) | 0.8 | MIT | HTTP routing and request-handling |
| [rand](https://crates.io/crates/rand) | 0.10 | MIT / Apache-2.0 | Random number generator |
| [chromiumoxide](https://crates.io/crates/chromiumoxide) | 0.9 | MIT / Apache-2.0 | A high-level API for interacting with the Chrome DevTools. |
| [futures](https://crates.io/crates/futures) | 0.3 | MIT / Apache-2.0 | Abstractions for asynchronous programming. |
| [reqwest](https://crates.io/crates/reqwest) | 0.13 | MIT / Apache-2.0 | Blocking HTTP client for Dispatcharr API communication |
| [symphonia](https://crates.io/crates/symphonia) | 0.6 | MPL-2.0 | Audio/video container handling (MKV, MP4/ISO) |

## License

This project is licensed under the **GNU General Public License v2.0**. See [LICENSE](LICENSE.txt) for the full license text.
