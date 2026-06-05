# RustAdmin Server Program

[![build](https://github.com/rustadministrator/rustadmin-server/actions/workflows/build.yaml/badge.svg)](https://github.com/rustadministrator/rustadmin-server/actions/workflows/build.yaml)

[**Download**](https://github.com/rustadministrator/rustadmin-server/releases)

[**Upstream Manual**](https://rustdesk.com/docs/en/self-host/)

[**Upstream FAQ**](https://github.com/rustdesk/rustdesk/wiki/FAQ)

[**Upstream OSS to Pro migration guide**](https://rustdesk.com/docs/en/self-host/rustdesk-server-pro/installscript/#convert-from-open-source)

Self-host your own RustAdmin server, it is free and open source.

## How to build manually

```bash
cargo build --release
```

Three executables will be generated in target/release.

- hbbs - RustAdmin ID/Rendezvous server
- hbbr - RustAdmin relay server
- rustadmin-utils - RustAdmin CLI utilities

You can find updated binaries on the [Releases](https://github.com/rustadministrator/rustadmin-server/releases) page.

For upstream Pro features, [RustDesk Server Pro](https://rustdesk.com/pricing.html) might suit you better.

If you want to develop your own upstream-compatible server, [rustdesk-server-demo](https://github.com/rustdesk/rustdesk-server-demo) might be a better and simpler start for you than this repo.

## Installation

Please follow the upstream [self-hosting doc](https://rustdesk.com/docs/en/self-host/rustdesk-server-oss/)
