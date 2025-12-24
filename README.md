# Hytale Username Checker

A fast, modern desktop application for bulk checking Hytale username availability.

Built with Rust and [iced](https://github.com/iced-rs/iced) for a native, cross-platform experience.

## Features

- **Bulk Checking** - Check hundreds of usernames at once
- **Multi-Proxy Support** - HTTP, HTTPS, SOCKS4, SOCKS5 with automatic rotation
- **Concurrent Requests** - Configurable thread count for faster checking
- **Real-time Results** - See results as they come in with filtering tabs
- **Export** - Save available usernames to a text file

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/tahakcbg/hytale-username-checker/releases) page.

### Build from Source

Requires [Rust](https://rustup.rs/) 1.75+

```bash
git clone https://github.com/tahakcbg/hytale-username-checker.git
cd hytale-username-checker
cargo build --release
```

Binary will be at `target/release/hytale-checker`

## Usage

1. Enter usernames (one per line) in the left panel
2. (Optional) Configure proxy settings:
   - Click "Proxy Settings" to expand
   - Select proxy type (HTTP/HTTPS/SOCKS4/SOCKS5)
   - Add proxies (one per line, format: `host:port` or `user:pass@host:port`)
3. Adjust delay and thread count as needed
4. Click "Start Check"
5. View results in the tabs (All/Available/Taken/Errors)
6. Export available usernames with the "Export" button

## Proxy Format

```
# Without authentication
127.0.0.1:8080
proxy.example.com:3128

# With authentication
user:password@127.0.0.1:8080
```

## License

MIT
