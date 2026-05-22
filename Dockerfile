###############################################################################
# Build dummy .deb packages to satisfy Chromium's dependency tree (without the hundreds of MB)
# Borrowed from: https://github.com/FlareSolverr/FlareSolverr
###############################################################################
FROM debian:bookworm-slim AS dummy-builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends equivs \
    # --- Dummy libgl1-mesa-dri (GPU mesa driver, not needed in headless) ---
    && equivs-control libgl1-mesa-dri \
    && printf 'Section: misc\nPriority: optional\nStandards-Version: 3.9.2\nPackage: libgl1-mesa-dri\nVersion: 99.0.0\nDescription: Dummy package for libgl1-mesa-dri\n' \
        >> libgl1-mesa-dri \
    && equivs-build libgl1-mesa-dri \
    && mv libgl1-mesa-dri_*.deb /libgl1-mesa-dri.deb \
    # --- Dummy adwaita-icon-theme (GTK icons, irrelevant for headless) ---
    && equivs-control adwaita-icon-theme \
    && printf 'Section: misc\nPriority: optional\nStandards-Version: 3.9.2\nPackage: adwaita-icon-theme\nVersion: 99.0.0\nDescription: Dummy package for adwaita-icon-theme\n' \
        >> adwaita-icon-theme \
    && equivs-build adwaita-icon-theme \
    && mv adwaita-icon-theme_*.deb /adwaita-icon-theme.deb

###############################################################################
# Install rust dependencies once (reused)
###############################################################################
FROM rust:slim-bookworm AS chef
 
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef
 
WORKDIR /build
 
###############################################################################
# Compute the dependency fingerprint from Cargo.toml + Cargo.lock (only reruns when those files change)
###############################################################################
FROM chef AS planner
 
# Only needs a minimal main instead of all source
RUN mkdir src && echo 'fn main() {}' > src/main.rs
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo chef prepare --recipe-path recipe.json
 
###############################################################################
# Build the depdencies when Cargo.toml + Cargo.lock change
###############################################################################
FROM chef AS rust-builder
 
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
 
###############################################################################
# Compile actual vod_downloader
###############################################################################
COPY src/ ./src
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release

###############################################################################
# Actual container with chromium and rust runtime
###############################################################################
FROM debian:bookworm-slim

ARG APP_USER=voddownloader
ARG APP_BIN=vod_downloader

# Bring in the dummy packages
COPY --from=dummy-builder /*.deb /tmp/

WORKDIR /app

RUN dpkg -i /tmp/libgl1-mesa-dri.deb \
    && dpkg -i /tmp/adwaita-icon-theme.deb \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        # Chromium and its WebDriver
        chromium \
        chromium-common \
        chromium-driver \
        # Virtual framebuffer — lets Chromium think there's a display
        xvfb \
        xauth \
        # dumb-init: PID 1 that reaps zombie Chromium subprocesses properly
        dumb-init \
        # Utilities
        ca-certificates \
        curl \
        procps \
    # Purge hardware-video-decode libs (unused in headless, saves ~20 MB)
    && rm -f /usr/lib/x86_64-linux-gnu/libmfxhw* \
    && rm -f /usr/lib/x86_64-linux-gnu/mfx/* \
    # Clean up apt artefacts and temp debs
    && rm -rf /var/lib/apt/lists/* /tmp/*.deb \
    # Create a non-root user for the app (never run Chromium as root in prod)
    && useradd --home-dir /app --shell /bin/sh --create-home ${APP_USER} \
    # Move chromedriver next to the app
    && mv /usr/bin/chromedriver /app/chromedriver \
    # Config volume directory
    && mkdir /config \
    && chown -R ${APP_USER}:${APP_USER} /app /config

# Copy the compiled Rust binary from the build stage
COPY --from=rust-builder /build/target/release/${APP_BIN} /app/${APP_BIN}
RUN chmod +x /app/${APP_BIN} \
    && chown ${APP_USER}:${APP_USER} /app/${APP_BIN}

# Copy web files
COPY web/ ./web

VOLUME /config

USER ${APP_USER}

# Chromium writes crash reports here; create it upfront to avoid runtime errors
RUN mkdir -p "/app/.config/chromium/Crash Reports/pending"

EXPOSE 8080

# HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=3 CMD curl -f http://localhost:8080/health || exit 1

# dumb-init as PID 1 ensures clean signal forwarding and zombie reaping
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/app/vod_downloader"]
