FROM em-ci-rust:1.95

SHELL ["/bin/bash", "-euxo", "pipefail", "-c"]

ARG RUST_STABLE=1.95.0
ARG CARGO_DENY_VERSION=0.19.6
ARG CARGO_FUZZ_VERSION=0.13.1
ARG CARGO_MUTANTS_VERSION=27.0.0
ARG TYPOS_VERSION=1.46.2

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl nasm tar \
    && rm -rf /var/lib/apt/lists/*

RUN rustup toolchain install "${RUST_STABLE}" --profile minimal --component rustfmt --component clippy \
    && rustup default "${RUST_STABLE}" \
    && rustup toolchain install nightly --profile minimal

RUN install_archive() { \
      local url="$1"; \
      local sha="$2"; \
      local binary="$3"; \
      local tmp; \
      tmp="$(mktemp -d)"; \
      curl -fsSL "$url" -o "$tmp/archive.tar.gz"; \
      echo "${sha}  ${tmp}/archive.tar.gz" | sha256sum -c -; \
      tar -xzf "$tmp/archive.tar.gz" -C "$tmp"; \
      local src; \
      src="$(find "$tmp" -type f -name "$binary" | head -n 1)"; \
      test -n "$src"; \
      install -m 0755 "$src" "/usr/local/cargo/bin/$binary"; \
      rm -rf "$tmp"; \
    }; \
    install_archive \
      "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
      "0021d321c781f0159a150ca308859ad93ccce64a887b22ad2e129f096a8a2c07" \
      "cargo-deny"; \
    install_archive \
      "https://github.com/rust-fuzz/cargo-fuzz/releases/download/${CARGO_FUZZ_VERSION}/cargo-fuzz-${CARGO_FUZZ_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
      "86edc9b8f20c29ff07c3e5fcd67bc3f0784b3b960a5428635060f20d50555461" \
      "cargo-fuzz"; \
    install_archive \
      "https://github.com/sourcefrog/cargo-mutants/releases/download/v${CARGO_MUTANTS_VERSION}/cargo-mutants-x86_64-unknown-linux-gnu.tar.gz" \
      "5083ce59bf9195ce9bb218278b609bbd183be897ca53671bad4df588fc7a9d7d" \
      "cargo-mutants"; \
    install_archive \
      "https://github.com/crate-ci/typos/releases/download/v${TYPOS_VERSION}/typos-v${TYPOS_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
      "d68c1a9c5abd8de11f7749edfa414087c8bc828e89064714487d23c89f36b06e" \
      "typos"; \
    rustc --version; \
    cargo --version; \
    cargo +nightly --version; \
    cargo deny --version; \
    cargo fuzz --version; \
    cargo mutants --version; \
    typos --version; \
    nasm -v; \
    sccache --version || true
