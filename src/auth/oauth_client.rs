use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge,
    RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use url::Url;

use super::store::{AuthStore, OAuthTokens};

const CALLBACK_PATH: &str = "/oauth/callback";
const DISCOVERY_PATH: &str = "/.well-known/oauth-authorization-server";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthServerMetadata {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    #[serde(default)]
    pub scopes_supported: Vec<String>,
}

pub struct OAuthClient {
    store: AuthStore,
}

impl OAuthClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: AuthStore::new()?,
        })
    }

    pub async fn discover_oauth_endpoints(server_url: &str) -> Result<OAuthServerMetadata> {
        let base_url = Url::parse(server_url).context("Invalid server URL")?;
        let discovery_url = base_url
            .join(DISCOVERY_PATH)
            .context("Failed to construct discovery URL")?;

        tracing::debug!("Discovering OAuth endpoints at: {}", discovery_url);

        let client = reqwest::Client::new();
        let response = client
            .get(discovery_url.as_str())
            .send()
            .await
            .context("Failed to fetch OAuth discovery endpoint")?;

        if !response.status().is_success() {
            bail!("OAuth discovery failed with status: {}", response.status());
        }

        let metadata: OAuthServerMetadata = response
            .json()
            .await
            .context("Failed to parse OAuth discovery response")?;

        tracing::debug!("Discovered OAuth endpoints: {:?}", metadata);
        Ok(metadata)
    }

    pub async fn authenticate(
        &self,
        server_name: &str,
        server_url: &str,
        client_id: &str,
        scopes: Option<Vec<String>>,
    ) -> Result<OAuthTokens> {
        let existing_token = self.store.load_token(server_name).await?;

        if let Some(token) = existing_token {
            if !token.is_expired() {
                tracing::debug!("Using existing valid token for {}", server_name);
                return Ok(token);
            }

            if token.needs_refresh() {
                if let Some(refresh_token) = &token.refresh_token {
                    tracing::info!("Refreshing expired token for {}", server_name);
                    match self
                        .refresh_token(server_name, server_url, client_id, refresh_token)
                        .await
                    {
                        Ok(new_token) => return Ok(new_token),
                        Err(e) => {
                            tracing::warn!("Token refresh failed: {}, re-authenticating", e);
                        }
                    }
                }
            }
        }

        tracing::info!("Performing OAuth authentication for {}", server_name);
        let metadata = Self::discover_oauth_endpoints(server_url).await?;

        let tokens = self
            .perform_oauth_flow(server_name, &metadata, client_id, scopes)
            .await?;

        self.store.save_token(server_name, &tokens).await?;
        Ok(tokens)
    }

    async fn perform_oauth_flow(
        &self,
        server_name: &str,
        metadata: &OAuthServerMetadata,
        client_id: &str,
        scopes: Option<Vec<String>>,
    ) -> Result<OAuthTokens> {
        let (listener, redirect_url) = Self::create_callback_server()?;

        let oauth_client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            None,
            AuthUrl::new(metadata.authorization_endpoint.clone())
                .context("Invalid authorization endpoint")?,
            Some(TokenUrl::new(metadata.token_endpoint.clone()).context("Invalid token endpoint")?),
        )
        .set_redirect_uri(redirect_url);

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_request = oauth_client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        if let Some(scope_list) = scopes {
            for scope in scope_list {
                auth_request = auth_request.add_scope(Scope::new(scope));
            }
        }

        let (auth_url, csrf_token) = auth_request.url();

        tracing::info!("Opening browser for OAuth authorization: {}", auth_url);
        open::that(auth_url.as_str()).context("Failed to open browser")?;

        let (code, state) = Self::wait_for_callback(listener).await?;

        if &state != csrf_token.secret() {
            bail!("CSRF token mismatch");
        }

        tracing::debug!("Exchanging authorization code for token");
        let token_result = oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Token exchange failed")?;

        let expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64));

        let tokens = OAuthTokens {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            expires_at,
        };

        tracing::info!("Successfully authenticated for {}", server_name);
        Ok(tokens)
    }

    async fn refresh_token(
        &self,
        server_name: &str,
        server_url: &str,
        client_id: &str,
        refresh_token: &str,
    ) -> Result<OAuthTokens> {
        let metadata = Self::discover_oauth_endpoints(server_url).await?;

        let oauth_client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            None,
            AuthUrl::new(metadata.authorization_endpoint.clone())
                .context("Invalid authorization endpoint")?,
            Some(TokenUrl::new(metadata.token_endpoint.clone()).context("Invalid token endpoint")?),
        );

        let token_result = oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Token refresh failed")?;

        let expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64));

        let tokens = OAuthTokens {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result
                .refresh_token()
                .map(|t| t.secret().clone())
                .or_else(|| Some(refresh_token.to_string())),
            expires_at,
        };

        self.store.save_token(server_name, &tokens).await?;
        tracing::info!("Successfully refreshed token for {}", server_name);

        Ok(tokens)
    }

    fn create_callback_server() -> Result<(TcpListener, RedirectUrl)> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .context("Failed to bind callback server")?;

        let port = listener.local_addr()?.port();
        let redirect_url = RedirectUrl::new(format!("http://localhost:{}{}", port, CALLBACK_PATH))
            .context("Failed to create redirect URL")?;

        tracing::debug!("Callback server listening on port {}", port);
        Ok((listener, redirect_url))
    }

    async fn wait_for_callback(listener: TcpListener) -> Result<(String, String)> {
        listener
            .set_nonblocking(true)
            .context("Failed to set non-blocking")?;

        let listener = tokio::net::TcpListener::from_std(listener)
            .context("Failed to convert to tokio listener")?;

        let (stream, _) = listener
            .accept()
            .await
            .context("Failed to accept connection")?;

        Self::handle_callback(stream).await
    }

    async fn handle_callback(mut stream: TcpStream) -> Result<(String, String)> {
        let mut reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .await
            .context("Failed to read request line")?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            bail!("Invalid HTTP request");
        }

        let path_and_query = parts[1];
        let url = Url::parse(&format!("http://localhost{}", path_and_query))
            .context("Failed to parse callback URL")?;

        let params: HashMap<_, _> = url.query_pairs().collect();

        let code = params
            .get("code")
            .ok_or_else(|| anyhow!("No code parameter in callback"))?
            .to_string();

        let state = params
            .get("state")
            .ok_or_else(|| anyhow!("No state parameter in callback"))?
            .to_string();

        let response = "HTTP/1.1 200 OK\r\n\
                       Content-Type: text/html\r\n\
                       \r\n\
                       <html><body>\
                       <h1>Authentication Successful</h1>\
                       <p>You can close this window and return to the application.</p>\
                       </body></html>";

        stream
            .write_all(response.as_bytes())
            .await
            .context("Failed to write response")?;

        stream.flush().await.context("Failed to flush stream")?;

        Ok((code, state))
    }

    #[allow(dead_code)]
    pub async fn get_valid_token(&self, server_name: &str) -> Result<Option<String>> {
        let token = self.store.load_token(server_name).await?;

        if let Some(token) = token {
            if !token.is_expired() {
                return Ok(Some(token.access_token));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_callback_server() {
        let result = OAuthClient::create_callback_server();
        assert!(result.is_ok());

        let (_listener, redirect_url) = result.unwrap();
        assert!(redirect_url.as_str().starts_with("http://localhost:"));
        assert!(redirect_url.as_str().ends_with("/oauth/callback"));
    }

    #[tokio::test]
    async fn test_oauth_client_creation() {
        let client = OAuthClient::new();
        assert!(client.is_ok());
    }
}
