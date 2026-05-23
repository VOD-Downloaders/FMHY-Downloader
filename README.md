# VOD Downloader

A docker container for downloading VODs off of certain freemediaheckyeah sites.

> [!WARNING]
> This software is currently in alpha stages, there may be bugs and breaking changes to the API.

## Features

- // TODO: ...

## Usage

// TODO: ...

## Contributing

Contributions are welcome.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-change`)
3. Make your changes and ensure everything compiles (`cargo build`)
4. Run tests (`cargo test`)
5. Run the linter (`cargo clippy`)
6. Format your code (`cargo +nightly fmt` from [`rustfmt`](https://github.com/rust-lang/rustfmt))
7. Open a pull request with a clear description of what you changed and why

### Guidelines

- Keep the webui user-friendly.
- Prefer returning `Result` types over panicking.
- New dependencies should be justified since I want to minimize the attack surface of the container.

## Third-Party Libraries

| Crate | Version | License | Purpose |
|---|---|---|---|
| [colored](https://crates.io/crates/colored) | 3.1 | MPL-2.0 | Coloured printing |
| [chrono](https://crates.io/crates/chrono) | 0.4 | MIT / Apache-2.0 | Timestamp handling |
| [tokio](https://crates.io/crates/tokio) | 1 | MIT | A runtime for writing reliable asynchronous applications |
| [serde](https://crates.io/crates/serde) | 1 | MIT / Apache-2.0 | Serialization/deserialization framework |
| [serde_json](https://crates.io/crates/serde_json) | 1 | MIT / Apache-2.0 | JSON parsing for API payloads |
| [axum](https://crates.io/crates/axum) | 0.8 | MIT | HTTP routing and request-handling |
| [chromiumoxide](https://crates.io/crates/chromiumoxide) | 0.9 | MIT / Apache-2.0 | A high-level API for interacting with the Chrome DevTools. |
| [futures](https://crates.io/crates/futures) | 0.3 | MIT / Apache-2.0 | Abstractions for asynchronous programming. |
| [reqwest](https://crates.io/crates/reqwest) | 0.13 | MIT / Apache-2.0 | Blocking HTTP client for Dispatcharr API communication |
| [symphonia](https://crates.io/crates/symphonia) | 0.6 | MPL-2.0 | Audio/video container handling (MKV, MP4/ISO) |

## License

This project is licensed under the **GNU General Public License v2.0**. See [LICENSE](LICENSE.txt) for the full license text.
