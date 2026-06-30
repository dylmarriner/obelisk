//! Local optimization proxy. Point a client's API base URL at
//! http://127.0.0.1:6767 (plain HTTP on loopback); Obelisk captures the bearer
//! token, re-originates each request over HTTPS to the real upstream, streams
//! the response back, and accounts tokens through the ledger so the dashboard
//! sees the whole pipeline. Blocking std::net + ureq — no async runtime.

use crate::ledger;
use crate::squeeze::est_tokens;
use anyhow::{anyhow, Context, Result};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn serve(port: u16, upstream: &str) -> Result<i32> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .with_context(|| format!("bind {addr} (already in use?)"))?;
    let upstream = upstream.trim_end_matches('/').to_string();
    eprintln!("obelisk proxy on http://{addr}  ->  {upstream}");
    eprintln!("  export ANTHROPIC_BASE_URL=http://{addr}");
    eprintln!("  Ctrl-C to stop.\n");
    for conn in listener.incoming() {
        match conn {
            Ok(c) => {
                let up = upstream.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle(c, &up) {
                        eprintln!("[proxy] {e}");
                    }
                });
            }
            Err(e) => eprintln!("[proxy] accept: {e}"),
        }
    }
    Ok(0)
}

fn handle(mut client: TcpStream, upstream: &str) -> Result<()> {
    let mut reader = BufReader::new(client.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or_else(|| anyhow!("no method"))?.to_string();
    let path = parts.next().ok_or_else(|| anyhow!("no path"))?.to_string();

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let t = line.trim_end_matches(['\r', '\n']);
        if t.is_empty() {
            break;
        }
        if let Some((k, v)) = t.split_once(':') {
            let (key, val) = (k.trim().to_string(), v.trim().to_string());
            let lk = key.to_ascii_lowercase();
            if lk == "content-length" {
                content_length = val.parse().unwrap_or(0);
            }
            if lk == "authorization" {
                if let Some(tok) = val.strip_prefix("Bearer ") {
                    let red = format!("bearer:{}…", tok.chars().take(6).collect::<String>());
                    let _ = ledger::stash(&red, "proxy_bearer", "captured");
                }
            }
            if !matches!(lk.as_str(), "host" | "connection" | "content-length"
                | "proxy-connection" | "transfer-encoding") {
                headers.push((key, val));
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    let in_tok = est_tokens(&String::from_utf8_lossy(&body));

    let url = format!("{upstream}{path}");
    let mut req = ureq::request(&method, &url);
    for (k, v) in &headers {
        req = req.set(k, v);
    }
    let resp = if body.is_empty() { req.call() } else { req.send_bytes(&body) };

    let (status, stext, rheaders, mut rreader): (u16, String, Vec<(String, String)>, Box<dyn Read>) =
        match resp {
            Ok(r) | Err(ureq::Error::Status(_, r)) => {
                let status = r.status();
                let stext = r.status_text().to_string();
                let mut hs = Vec::new();
                for name in r.headers_names() {
                    if let Some(v) = r.header(&name) {
                        if !matches!(name.to_ascii_lowercase().as_str(),
                            "connection" | "transfer-encoding" | "content-length" | "keep-alive") {
                            hs.push((name.clone(), v.to_string()));
                        }
                    }
                }
                (status, stext, hs, r.into_reader())
            }
            Err(e) => {
                let msg = format!("obelisk proxy upstream error: {e}");
                let _ = write!(client,
                    "HTTP/1.1 502 Bad Gateway\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    msg.len(), msg);
                return Ok(());
            }
        };

    let mut rbody = Vec::new();
    rreader.read_to_end(&mut rbody)?;
    let out_tok = est_tokens(&String::from_utf8_lossy(&rbody));
    let _ = ledger::record_event("proxy", &format!("{method} {path}"),
                                 in_tok + out_tok, in_tok + out_tok);

    let mut head = format!("HTTP/1.1 {status} {stext}\r\n");
    for (k, v) in &rheaders {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\nConnection: close\r\n\r\n", rbody.len()));
    client.write_all(head.as_bytes())?;
    client.write_all(&rbody)?;
    client.flush()?;
    Ok(())
}
