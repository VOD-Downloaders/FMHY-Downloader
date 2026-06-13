# FMHY Downloader

A docker container for downloading VODs off of certain [freemediaheckyeah](https://fmhy.net/video) sites.

> [!WARNING]
> This software is currently in alpha stages, there may be bugs and breaking changes to the API.

## Features

- VOD Indexing from various [freemediaheckyeah](https://fmhy.net/video) sites. // TODO
- (Bulk) VOD Downloading from [freemediaheckyeah](https://fmhy.net/video) sites.
- Easy navigatable WebUI
- VOD Validation // TODO
- Prowlarr indexer API // TODO

## Installation

Pre-built images are published to the GitHub Container Registry at
[`ghcr.io/vod-downloaders/fmhy-downloader`](https://github.com/VOD-Downloaders/FMHY-Downloader/pkgs/container/fmhy-downloader).
Available tags: `latest` (newest release), `nightly` (latest `dev` build), and per-version tags (e.g. `0.1.0-alpha2`).

The recommended way to run is with Docker Compose. Create a `compose.yaml`:

```yaml
services:
  fmhy_downloader:
    image: ghcr.io/vod-downloaders/fmhy-downloader:latest
    container_name: fmhy_downloader
    volumes:
      - ./config:/config
      - ./output:/output
    environment:
      - LOG_LEVEL=info
      - FLARESOLVERR_URL=http://flaresolverr:8191/v1
    ports:
      - 8080:8080
    depends_on:
      flaresolverr:
        condition: service_healthy
    restart: unless-stopped

  fmhy_bridge:
    image: ghcr.io/ggjorven/fmhy-bridge:latest
    container_name: fmhy_bridge
    environment:
      - LOG_LEVEL=info
    ports:
      - 3000:3000
    restart: unless-stopped

  flaresolverr:
    image: ghcr.io/flaresolverr/flaresolverr:latest
    container_name: flaresolverr
    environment:
      - LOG_LEVEL=info
    healthcheck:
      test: sh -c "curl https://www.google.com && curl http://localhost:8191 && curl http://localhost:8191/health"
      interval: 5s
      timeout: 10s
      retries: 3
      start_period: 10s
    restart: unless-stopped
```

Then start it:

```bash
docker compose up -d
```

> [!NOTE]
> `flaresolverr` is only required for Cloudflare-protected sites. If you don't need it, remove the service,
> the `depends_on` block, and the `FLARESOLVERR_URL` environment variable.

## Usage

The WebUI is served on [`http://localhost:8080`](http://localhost:8080). 

Use the sidebar to navigate between pages:

1. **Search** – // TODO: Search for a series
2. **Streams** – Shows available streams for selected episode or movie.
3. **Downloads** – Shows the downloads you've started. Finished files land in the `./output` directory.

### Configuration

The container is configured through environment variables:

| Variable | Default | Description |
|---|---|---|
| `LOG_LEVEL` | `info` | Log verbosity: `debug` / `info` / `warning` / `error` |
| `WEBUI_PORT` | `8080` | Port the WebUI/API listens on |
| `FLARESOLVERR_URL` | - | FlareSolverr endpoint (may be empty) |

## API

The docker container exposes an HTTP server with callable API functions, listed below:

| Type | Endpoint | Description | Input | Output |
|---|---|---|---|---|
| `GET` | `/health` | Healthcheck endpoint | - | `{ health }` |
| `GET` | `/api/indexers` | Retrieve active indexers | - | `{ indexers }` |
| `POST` | `/api/indexers/create` | Create an active indexer from a specification | `{ indexer }` | - |
| `POST` | `/api/indexers/delete` | Delete an active indexer | `{ name }` | - |
| `GET` | `/api/indexers/specifications` | Retrieve usable indexer specifications | - | `{ indexers }` |
| `POST` | `/api/indexers/specifications/refresh` | Refetch indexer specifications from GitHub | - | `{ indexers }` |
| `POST` | `/api/streams` | Analyze a URL and list the available streams | `{ indexer_name, input_url }` | `{ streams }` |
| `POST` | `/api/download` | Start a VOD download | `{ indexer_name, stream, output_file }` | `{ id }` |

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
| [thiserror](https://crates.io/crates/thiserror) | 2.0 | MIT / Apache-2.0 | Easy error creation |
| [chrono](https://crates.io/crates/chrono) | 0.4 | MIT / Apache-2.0 | Timestamp handling |
| [tokio](https://crates.io/crates/tokio) | 1 | MIT | A runtime for writing reliable asynchronous applications |
| [serde](https://crates.io/crates/serde) | 1 | MIT / Apache-2.0 | Serialization/deserialization framework |
| [serde_json](https://crates.io/crates/serde_json) | 1 | MIT / Apache-2.0 | JSON parsing for API payloads |
| [serde_url_params](https://crates.io/crates/serde_url_params) | 0.2 | MIT / Apache-2.0 | JSON parsing for URLs |
| [axum](https://crates.io/crates/axum) | 0.8 | MIT | HTTP routing and request-handling |
| [tower_http](https://crates.io/crates/tower_http) | 0.6 | MIT | Easy web file serving |
| [base64](https://crates.io/crates/base64) | 0.22 | MIT / Apache-2.0 | Base64 encoder and decoder |
| [rand](https://crates.io/crates/rand) | 0.10 | MIT / Apache-2.0 | Random number generator |
| [url](https://crates.io/crates/url) | 2.5 | MIT / Apache-2.0 | Rust implementation of the URL standard. |
| [async_trait](https://crates.io/crates/async_trait) | 0.1 | MIT / Apache-2.0 | Allow async functions in traits |
| [chromiumoxide](https://crates.io/crates/chromiumoxide) | 0.9 | MIT / Apache-2.0 | A high-level API for interacting with the Chrome DevTools. |
| [futures](https://crates.io/crates/futures) | 0.3 | MIT / Apache-2.0 | Abstractions for asynchronous programming. |
| [reqwest](https://crates.io/crates/reqwest) | 0.13 | MIT / Apache-2.0 | Blocking HTTP client for Dispatcharr API communication |
| [symphonia](https://crates.io/crates/symphonia) | 0.6 | MPL-2.0 | Audio/video container handling (MKV, MP4/ISO) |

## License

This project is licensed under the **GNU Affero General Public License v3.0**. See [LICENSE](LICENSE.txt) for the full license text.
