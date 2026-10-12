use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use anyhow::{Context, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const TIMEOUT: Duration = Duration::from_secs(10);
const LONGEST_STATUS_LINE: u64 = 1024;

pub async fn check(listen: SocketAddr) -> anyhow::Result<()> {
    match status(listen, astrum_api::STATUS).await? {
        200 => Ok(()),
        code => bail!("{} answered {code}", astrum_api::STATUS),
    }
}

pub async fn status(listen: SocketAddr, path: &str) -> anyhow::Result<u16> {
    let target = target(listen);
    timeout(TIMEOUT, request(target, path))
        .await
        .with_context(|| format!("{target} did not answer within {TIMEOUT:?}"))?
}

pub fn target(listen: SocketAddr) -> SocketAddr {
    match listen {
        SocketAddr::V4(address) if address.ip().is_unspecified() => {
            SocketAddr::new(Ipv4Addr::LOCALHOST.into(), address.port())
        }
        SocketAddr::V6(address) if address.ip().is_unspecified() => {
            SocketAddr::new(Ipv6Addr::LOCALHOST.into(), address.port())
        }
        other => other,
    }
}

async fn request(target: SocketAddr, path: &str) -> anyhow::Result<u16> {
    let mut stream = TcpStream::connect(target)
        .await
        .with_context(|| format!("cannot connect to {target}"))?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: {target}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    let mut head = Vec::new();
    stream
        .take(LONGEST_STATUS_LINE)
        .read_to_end(&mut head)
        .await
        .with_context(|| format!("{target} closed the connection"))?;
    parse_status_line(&head).with_context(|| format!("{target} did not answer in HTTP"))
}

fn parse_status_line(head: &[u8]) -> Option<u16> {
    let line = head.split(|byte| *byte == b'\n').next()?;
    let line = std::str::from_utf8(line).ok()?;
    let mut parts = line.split_ascii_whitespace();
    parts
        .next()
        .filter(|version| version.starts_with("HTTP/"))?;
    parts.next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_on_every_interface_is_checked_on_the_loopback() {
        assert_eq!(
            target("0.0.0.0:8081".parse().unwrap()),
            "127.0.0.1:8081".parse().unwrap()
        );
        assert_eq!(
            target("[::]:8081".parse().unwrap()),
            "[::1]:8081".parse().unwrap()
        );
        assert_eq!(
            target("10.0.0.5:8081".parse().unwrap()),
            "10.0.0.5:8081".parse().unwrap()
        );
    }

    #[test]
    fn the_status_line_gives_the_code() {
        assert_eq!(
            parse_status_line(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n"),
            Some(200)
        );
        assert_eq!(
            parse_status_line(b"HTTP/1.1 503 Service Unavailable\r\n"),
            Some(503)
        );
        assert_eq!(parse_status_line(b"SSH-2.0-OpenSSH_9.6\r\n"), None);
        assert_eq!(parse_status_line(b""), None);
    }
}
