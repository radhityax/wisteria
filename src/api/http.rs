use std::sync::Arc;
use anyhow::Result;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::block::{Block, parse_cid};
use crate::store::BlockStore;
use crate::pin::PinManager;

pub struct HttpConfig {
    pub port: u16,
    pub store: Arc<dyn BlockStore>,
    pub pinner: PinManager,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            store: Arc::new(crate::store::memory::MemoryBlockStore::new()),
            pinner: PinManager::new(),
        }
    }
}

pub async fn serve(config: HttpConfig) -> Result<()> {
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("http api & gateway listening on {}", addr);

    loop {
        let (mut stream, peer) = listener.accept().await?;
        let store = config.store.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut stream, store).await {
                log::warn!("request from {} failed: {}", peer, e);
            }
        });
    }
}

async fn handle_connection(
    stream: &mut tokio::net::TcpStream,
    store: Arc<dyn BlockStore>,
) -> Result<()> {
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    if n == 0 { return Ok(()); }

    let request = String::from_utf8_lossy(&buf[..n]);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 { return Ok(()); }

    let method = parts[0];
    let path = parts[1];

    // Gateway: GET /ipfs/<cid>
    if let Some(cid_str) = path.strip_prefix("/ipfs/") {
        if method != "GET" { return write_status(stream, 405).await; }
        let cid = match parse_cid(cid_str) {
            Ok(c) => c,
            Err(_) => return write_status(stream, 400).await,
        };
        match store.get(&cid).await? {
            Some(block) => {
                let data = block.data();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nX-Content-Type-Options: nosniff\r\n\r\n",
                    data.len()
                );
                stream.write_all(header.as_bytes()).await?;
                stream.write_all(data).await?;
            }
            None => write_status(stream, 404).await?,
        }
        return Ok(());
    }

    // API: POST /api/v0/block
    if path == "/api/v0/block" && method == "POST" {
        let body = extract_body(&request)?;
        let block = Block::new(body)?;
        store.put(&block).await?;
        let json = format!("{{\"cid\":\"{}\"}}\n", block.cid());
        write_json(stream, 200, &json).await?;
        return Ok(());
    }

    // API: GET /api/v0/block/<cid> or DELETE
    if let Some(cid_str) = path.strip_prefix("/api/v0/block/") {
        let cid = match parse_cid(cid_str) {
            Ok(c) => c,
            Err(_) => return write_status(stream, 400).await,
        };
        if method == "GET" {
            match store.get(&cid).await? {
                Some(block) => {
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                        block.data().len()
                    );
                    stream.write_all(header.as_bytes()).await?;
                    stream.write_all(block.data()).await?;
                }
                None => write_status(stream, 404).await?,
            }
        } else if method == "DELETE" {
            store.remove(&cid).await?;
            write_json(stream, 200, "{\"status\":\"removed\"}\n").await?;
        }
        return Ok(());
    }

    // API: GET /api/v0/block (list)
    if path == "/api/v0/block" && method == "GET" {
        let cids = store.list().await?;
        let list: String = cids.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(",");
        write_json(stream, 200, &format!("[{}]\n", list)).await?;
        return Ok(());
    }

    // API: GET /api/v0/id
    if path == "/api/v0/id" && method == "GET" {
        write_json(stream, 200, "{\"peer_id\":\"local\",\"addresses\":[]}\n").await?;
        return Ok(());
    }

    write_status(stream, 404).await
}

fn extract_body(request: &str) -> Result<Vec<u8>> {
    if let Some(body_start) = request.find("\r\n\r\n") {
        let header_part = &request[..body_start];
        let body = &request[body_start + 4..];
        for line in header_part.lines() {
            if line.to_lowercase().starts_with("content-length:") {
                if let Ok(len) = line.trim_start_matches("content-length:")
                    .trim().parse::<usize>()
                {
                    return Ok(body.as_bytes()[..len.min(body.len())].to_vec());
                }
            }
        }
        Ok(body.as_bytes().to_vec())
    } else {
        Ok(vec![])
    }
}

async fn write_status(stream: &mut tokio::net::TcpStream, code: u16) -> Result<()> {
    let (reason, body) = match code {
        200 => ("OK", "ok"),
        400 => ("Bad Request", "bad request"),
        404 => ("Not Found", "not found"),
        405 => ("Method Not Allowed", "method not allowed"),
        _ => ("Internal Server Error", "error"),
    };
    let resp = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
        code, reason, body.len(), body
    );
    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

async fn write_json(stream: &mut tokio::net::TcpStream, code: u16, json: &str) -> Result<()> {
    let reason = match code {
        200 => "OK",
        _ => "Error",
    };
    let resp = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        code, reason, json.len(), json
    );
    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}


