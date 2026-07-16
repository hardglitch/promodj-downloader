use crate::log;
use crate::ui::MyApp;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use strum::{Display, EnumIter};

#[derive(Debug, Default, Copy, Clone, Display, EnumIter, PartialEq)]
pub enum ProxyType {
    #[default]
    Http,
    HttpAuth,
    Socks5,
}
impl ProxyType {
    pub fn from<T: Display + AsRef<str>>(string: T) -> Self {
        match string.as_ref() {
            "Http" => Self::Http,
            "HttpAuth" => Self::HttpAuth,
            "Socks5" => Self::Socks5,
            _ => Self::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub address: SocketAddr,
    pub login: Option<String>,
    pub password: Option<String>,
}
impl Default for Proxy {
    fn default() -> Self {
        Self {
            proxy_type: Default::default(),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 8080),
            login: None,
            password: None,
        }
    }
}
impl Display for Proxy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let protocol = self.proxy_type.to_string().to_lowercase();
        let address = self.address.to_string(); // "host:port"
        let auth =
            if let Some(login) = &self.login && let Some(password) = &self.password {
                format!("{login}:{password}@")
            }
            else { "".to_owned() };

        // "protocol://login:password@host:port"
        write!(f, "{protocol}://{auth}{address}")
    }
}
impl MyApp {
    pub fn save_proxy(&mut self, proxy: &Proxy) {
        if let Err(e) = self.proxy_logic(proxy) {
            log!("Proxy: {e}");
        }
    }
    fn proxy_logic(&mut self, proxy: &Proxy) -> anyhow::Result<()> {
        let proxy_url = reqwest::Proxy::http(proxy.to_string())?;
        let client = reqwest::Client::builder()
            .proxy(proxy_url)
            .build()?;

        self.client = client;
        Ok(())
    }
}
