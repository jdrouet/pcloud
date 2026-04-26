use std::collections::HashMap;
use std::process::Stdio;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const RCLONE_CLIENT_ID: &str = "DnONSzyJXpm";
const RCLONE_CLIENT_SECRET: &str = "oboe7MzS0dcs95oU3sRmxIRu8";
const BIND_HOST: &str = "127.0.0.1";
const REDIRECT_HOST: &str = "localhost";
const REDIRECT_PORT: u16 = 53682;
const AUTHORIZE_URL: &str = "https://my.pcloud.com/oauth2/authorize";

const SUCCESS_BODY: &[u8] = b"<!doctype html><html><head><meta charset=\"utf-8\"><title>pcloud-cli</title></head><body><h1>Authorization complete</h1><p>You can close this window.</p></body></html>";

#[derive(clap::Parser)]
pub(crate) struct Command {
    /// OAuth2 client id used to perform the authorization flow.
    ///
    /// Defaults to the public client id used by rclone, so that authorizations
    /// granted to rclone can be reused.
    #[clap(long, env = "PCLOUD_OAUTH_CLIENT_ID", default_value = RCLONE_CLIENT_ID)]
    client_id: String,
    /// OAuth2 client secret matching the client id.
    #[clap(long, env = "PCLOUD_OAUTH_CLIENT_SECRET", default_value = RCLONE_CLIENT_SECRET)]
    client_secret: String,
    /// Skip the attempt to open the authorization URL in the default browser.
    #[clap(long)]
    no_browser: bool,
}

impl Command {
    pub(crate) async fn execute(self, _client: &pcloud::Client) -> anyhow::Result<()> {
        let bind_addr = (BIND_HOST, REDIRECT_PORT);
        let listener = TcpListener::bind(bind_addr)
            .await
            .with_context(|| format!("unable to bind {BIND_HOST}:{REDIRECT_PORT}"))?;

        let auth_url = format!(
            "{AUTHORIZE_URL}?client_id={client_id}&response_type=code&redirect_uri=http%3A%2F%2F{REDIRECT_HOST}%3A{REDIRECT_PORT}%2F",
            client_id = self.client_id,
        );

        eprintln!("Open the following URL in your browser to authorize pcloud-cli:");
        eprintln!();
        eprintln!("    {auth_url}");
        eprintln!();
        if !self.no_browser {
            if let Err(err) = open_in_browser(&auth_url) {
                tracing::debug!("unable to open browser: {err}");
            }
        }
        eprintln!("Waiting for authorization on http://{REDIRECT_HOST}:{REDIRECT_PORT}/ ...");

        let params = wait_for_callback(&listener).await?;

        let code = params
            .get("code")
            .ok_or_else(|| match params.get("error") {
                Some(err) => anyhow::anyhow!("authorization failed: {err}"),
                None => anyhow::anyhow!("missing 'code' parameter in OAuth callback"),
            })?
            .to_owned();

        let exchange_base_url = params
            .get("hostname")
            .map(|host| format!("https://{host}"))
            .unwrap_or_else(|| pcloud::US_REGION.to_string());

        tracing::debug!("exchanging code at {exchange_base_url}");
        let exchange = pcloud::Client::new(exchange_base_url, pcloud::Credentials::anonymous())?;
        let token = exchange
            .oauth2_token(&self.client_id, &self.client_secret, &code)
            .await
            .context("OAuth token exchange failed")?;

        println!("{}", token.access_token);
        eprintln!();
        eprintln!("Save it as PCLOUD_ACCESS_TOKEN or in your config file.");
        Ok(())
    }
}

async fn wait_for_callback(listener: &TcpListener) -> anyhow::Result<HashMap<String, String>> {
    let (mut stream, _) = listener.accept().await?;
    let mut buffer = Vec::with_capacity(2048);
    let mut chunk = [0u8; 1024];
    loop {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 16 * 1024 {
            anyhow::bail!("OAuth callback request exceeds 16 KiB");
        }
    }

    let request = std::str::from_utf8(&buffer).context("OAuth callback is not valid UTF-8")?;
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty OAuth callback request"))?;
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("malformed OAuth callback request line"))?;
    let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
    let params = parse_query(query);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        SUCCESS_BODY.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.write_all(SUCCESS_BODY).await?;
    stream.shutdown().await.ok();

    Ok(params)
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if query.is_empty() {
        return out;
    }
    for pair in query.split('&') {
        let (key, value) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        out.insert(percent_decode(key), percent_decode(value));
    }
    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push((hi * 16 + lo) as u8);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(target_os = "macos")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    spawn_detached("open", &[url])
}

#[cfg(target_os = "windows")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    spawn_detached("cmd", &["/C", "start", "", url])
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    spawn_detached("xdg-open", &[url])
}

fn spawn_detached(program: &str, args: &[&str]) -> std::io::Result<()> {
    std::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{parse_query, percent_decode};

    #[test]
    fn parses_query_string() {
        let params = parse_query("code=abc&hostname=eapi.pcloud.com&extra=hello%20world");
        assert_eq!(params.get("code").map(String::as_str), Some("abc"));
        assert_eq!(
            params.get("hostname").map(String::as_str),
            Some("eapi.pcloud.com")
        );
        assert_eq!(
            params.get("extra").map(String::as_str),
            Some("hello world")
        );
    }

    #[test]
    fn decodes_percent_encoding() {
        assert_eq!(percent_decode("a%2Bb"), "a+b");
        assert_eq!(percent_decode("a+b"), "a b");
        assert_eq!(percent_decode("plain"), "plain");
    }
}
