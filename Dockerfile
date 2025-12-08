# Build the daemon
FROM debian:stable-slim AS buildenv

ENV APT_PACKAGES build-essential ca-certificates curl
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update \
    && apt-get upgrade --yes \
    && apt-get install --yes --no-install-recommends ${APT_PACKAGES}

RUN useradd --system --uid=10000 rust
USER rust
WORKDIR /home/rust

RUN curl --tlsv1.3 --output rustup.sh https://sh.rustup.rs \
    && sh rustup.sh -y --profile minimal

COPY --chown=rust:rust ./ ./rtsp2hls
RUN .cargo/bin/cargo install --path=./rtsp2hls


# Build the real container
FROM debian:stable-slim

ENV APT_PACKAGES ca-certificates gstreamer1.0-tools gstreamer1.0-rtsp
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update \
    && apt-get upgrade --yes \
    && apt-get install --yes --no-install-recommends ${APT_PACKAGES} \
    && apt-get clean

COPY --from=buildenv --chown=root:root /home/rust/.cargo/bin/rtsp2hls /usr/bin/

RUN useradd --system --create-home --home=/home/rtsp2hls --shell=/sbin/nologin --uid=10000 rtsp2hls
USER rtsp2hls

ENTRYPOINT ["/usr/bin/rtsp2hls"]
