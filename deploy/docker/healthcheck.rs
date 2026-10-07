//! Minimal, dependency-free container HEALTHCHECK probe for `chirp-server`.
//!
//! The production runtime image is `distroless/cc`, which has no shell, `curl`, or `wget`, so the
//! Docker `HEALTHCHECK` instruction cannot shell out to a standard HTTP client. This standalone
//! program is compiled with plain `rustc -O` in the Docker build stage (it is std-only — no crates,
//! no Cargo project, so it does not touch the workspace manifest) and copied into the runtime image.
//!
//! It performs ONE `GET /healthz` against the server over loopback and maps the outcome to the exit
//! code Docker's `HEALTHCHECK` consumes: `0` = healthy (HTTP 200), non-zero = unhealthy. The target
//! port is derived from the very same `CHIRP_BIND_ADDR` the server reads, so the probe tracks the
//! configured port with no second knob (default `8080`).

use std::env;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::exit;
use std::time::Duration;

/// Resolve the loopback port to probe from `CHIRP_BIND_ADDR` (`host:port`, e.g. `0.0.0.0:8080`).
///
/// Takes the segment after the last `:` so IPv6-style hosts do not confuse the split, validates it
/// is all-digits, and falls back to `8080` (the image default) when the variable is unset or malformed.
fn probe_port() -> String {
    env::var("CHIRP_BIND_ADDR")
        .ok()
        .and_then(|addr| addr.rsplit(':').next().map(str::to_owned))
        .filter(|port| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()))
        .unwrap_or_else(|| "8080".to_owned())
}

/// Issue the probe. Returns `Ok(true)` only when the response status line carries ` 200 `.
fn healthy() -> std::io::Result<bool> {
    let timeout = Duration::from_secs(3);
    let socket: SocketAddr = format!("127.0.0.1:{}", probe_port())
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad probe address"))?;

    let mut stream = TcpStream::connect_timeout(&socket, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    // HTTP/1.0 + `Connection: close` so the server closes the socket when done and `read_to_end`
    // terminates without us having to parse `Content-Length`.
    stream.write_all(b"GET /healthz HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let response = String::from_utf8_lossy(&raw);
    let status_line = response.lines().next().unwrap_or_default();
    // Status line looks like `HTTP/1.0 200 OK`; match the surrounded code to avoid false positives.
    Ok(status_line.contains(" 200 ") || status_line.ends_with(" 200"))
}

fn main() {
    match healthy() {
        Ok(true) => exit(0),
        _ => exit(1),
    }
}
