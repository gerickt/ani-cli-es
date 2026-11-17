use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use std::time::Duration;

#[derive(Clone)]
pub struct JkClient {
    client: reqwest::Client,
    pub base_url: String,
}

impl JkClient {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
            ),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("es-ES,es;q=0.9,en;q=0.8"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(false)
            .local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
            .timeout(Duration::from_secs(25))
            .connect_timeout(Duration::from_secs(8))
            .build()
            .context("Error al inicializar reqwest Client")?;

        Ok(Self {
            client,
            base_url: "https://jkanime.net".to_string(),
        })
    }

    pub async fn get_text(&self, url: &str) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Error al conectar con {}", url))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .with_context(|| format!("Error al leer respuesta de {}", url))?;
        if status.as_u16() == 429 || text.contains("Fuiste bloqueado por nuestro firewall") {
            anyhow::bail!("Rate limit de JKAnime (bloqueo de firewall por exceso de peticiones). Espera un momento.");
        }
        if !status.is_success() {
            anyhow::bail!("Error HTTP {} al obtener {}", status, url);
        }
        Ok(text)
    }

    pub async fn post_ajax(
        &self,
        url: &str,
        token: &str,
        form: &[(&str, &str)],
    ) -> Result<String> {
        let resp = self
            .client
            .post(url)
            .header("X-CSRF-TOKEN", token)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", &self.base_url)
            .form(form)
            .send()
            .await
            .with_context(|| format!("Error en POST AJAX a {}", url))?;
        let status = resp.status();
        let text = resp.text().await?;
        if status.as_u16() == 429 || text.contains("Fuiste bloqueado por nuestro firewall") {
            anyhow::bail!("Rate limit de JKAnime (bloqueo de firewall por exceso de peticiones). Espera un momento.");
        }
        if !status.is_success() {
            anyhow::bail!("Error HTTP {} en POST {}", status, url);
        }
        Ok(text)
    }
}
