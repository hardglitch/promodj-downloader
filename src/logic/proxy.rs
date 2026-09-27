use crate::ui::MyApp;
use crate::utils::security::{encrypt_and_write, read_and_decrypt};
use anyhow::anyhow;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use x_log::log;
use strum::{Display, EnumIter};
use tokio::sync::RwLock;

#[derive(Debug, Default, Copy, Clone, Display, EnumIter, PartialEq)]
pub enum ProxyType {
    #[default]
    Http,
    Https,
    Socks5,
}
impl ProxyType {
    pub fn from<T: Display + AsRef<str>>(string: T) -> Option<Self> {
        if string.as_ref().eq_ignore_ascii_case("Http") { Some(Self::Http) }
        else if string.as_ref().eq_ignore_ascii_case("Https") { Some(Self::Https) }
        else if string.as_ref().eq_ignore_ascii_case("Socks5") { Some(Self::Socks5) }
        else { None }
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
impl Proxy {
    fn parse(value: &[u8]) -> Option<Self> {
        let string = String::from_utf8(value.to_vec()).ok()?;

        let (protocol, last) = string.split_once("://")?;

        let (first, port) = last.rsplit_once(':')?;
        let port = port.parse::<u16>().ok()?;

        let (login, password, last) =
            if let Some((login, last)) = first.split_once(':') &&
               let Some((password, last)) = last.split_once('@')
            {
                (Some(login.to_owned()), Some(password.to_owned()), last)
            }
            else { (None, None, first) };

        let mut last = last.split( '.');
        let host_p1 = last.next()?.parse::<u8>().ok()?;
        let host_p2 = last.next()?.parse::<u8>().ok()?;
        let host_p3 = last.next()?.parse::<u8>().ok()?;
        let host_p4 = last.next()?.parse::<u8>().ok()?;

        let proxy_type = ProxyType::from(protocol)?;

        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(host_p1, host_p2, host_p3, host_p4)), port);

        let proxy = Self {
            proxy_type,
            address,
            login,
            password,
        };

        Some(proxy)
    }
}
impl MyApp {
    pub fn save_proxy(&mut self) {
        let host =
            match self.proxy_host.parse::<IpAddr>() {
                Ok(ip) => ip,
                Err(e) => {
                    log!("The Proxy parsing failed: {e}");
                    return
                }
            };
        let port =
            match self.proxy_port.parse::<u16>() {
                Ok(port) => port,
                Err(e) => {
                    log!("The Proxy parsing failed: {e}");
                    return
                }
            };
        let address = SocketAddr::new(host, port);

        let login =
            if let Ok(s) = self.proxy_login.try_read() && s.len() > 1 { Some(s.clone()) }
            else { None };

        let password =
            if let Ok(s) = self.proxy_password.try_read() && s.len() > 1 { Some(s.clone()) }
            else { None };

        let proxy = Proxy {
            proxy_type: self.proxy_type,
            address,
            login,
            password,
        };

        match self.proxy_logic(&proxy) {
            Ok(_) => {
                let proxy_data = proxy.to_string();
                if let Err(e) = encrypt_and_write(&proxy_data) { log!("Proxy: {e}"); }
            }
            Err(e) => log!("Proxy: {e}")
        }
    }
    pub fn load_proxy(&mut self) -> anyhow::Result<()> {
        let proxy_data = read_and_decrypt()?;
        match Proxy::parse(&proxy_data) {
            Some(proxy) => self.proxy_logic(&proxy)?,
            None => return Err(anyhow!("A Proxy parsing failed"))
        }
        Ok(())
    }
    fn proxy_logic(&mut self, proxy: &Proxy) -> anyhow::Result<()> {
        if self.use_proxy {
            let req_proxy = reqwest::Proxy::all(proxy.to_string())?;
            let client = reqwest::Client::builder()
                .proxy(req_proxy)
                .build()?;
            *self.client.try_write()? = client;
        }

        self.proxy_type = proxy.proxy_type;
        self.proxy_host = proxy.address.ip().to_string();
        self.proxy_port = proxy.address.port().to_string();

        if let Some(login) = &proxy.login {
            self.proxy_login = Arc::new(RwLock::new(login.clone()));
        }
        if let Some(password) = &proxy.password {
            self.proxy_password = Arc::new(RwLock::new(password.clone()));
        }

        Ok(())
    }
}
