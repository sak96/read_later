# Builder stage - tag as read_later-builder:latest
FROM ubuntu:jammy AS builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    build-essential \
    wget \
    curl \
    git \
    tar \
    unzip \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_VERSION=1.28.2
ENV RUST_TOOLCHAIN=1.91.1
ENV NODE_VERSION=24.14.1

RUN wget -q -O /tmp/nodejs.tar.xz https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz && \
    tar xf /tmp/nodejs.tar.xz -C /opt && \
    mv /opt/node-v${NODE_VERSION}-linux-x64 /opt/node && \
    rm /tmp/nodejs.tar.xz

ENV PATH="${PATH}:/opt/node/bin"

ENV GRADLE_VERSION=8.14.3

RUN wget -q -O /opt/gradle-${GRADLE_VERSION}-bin.zip https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip

RUN mkdir /opt/android
RUN mkdir /opt/java

RUN useradd -m -s /bin/bash runner
USER runner
WORKDIR /home/runner

ENV HOME=/home/runner
ENV RUSTUP_HOME=/home/runner/.rustup
ENV CARGO_HOME=/home/runner/.cargo
ENV PATH="/home/runner/.cargo/bin:${PATH}"
RUN mkdir -p /home/runner/.cargo /home/runner/.rustup

RUN git clone https://github.com/rust-lang/rustup.git -b ${RUSTUP_VERSION} /tmp/rustup && \
    cd /tmp/rustup && \
    ./rustup-init.sh -y \
        --default-toolchain ${RUST_TOOLCHAIN} \
        --target aarch64-linux-android && \
    rm -rf /tmp/rustup

RUN cargo install tauri-cli@2.9.5

# Final stage
FROM builder

RUN mkdir -p /home/runner/work/read_later/read_later
WORKDIR /home/runner/work/read_later/read_later

COPY --chown=runner:runner . .
RUN chmod +x /home/runner/work/read_later/read_later/entrypoint.sh

ENTRYPOINT ["/home/runner/work/read_later/read_later/entrypoint.sh"]