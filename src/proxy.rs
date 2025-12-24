#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyType {
    #[default]
    None,
    Http,
    Https,
    Socks4,
    Socks5,
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::None => write!(f, "No Proxy"),
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Https => write!(f, "HTTPS"),
            ProxyType::Socks4 => write!(f, "SOCKS4"),
            ProxyType::Socks5 => write!(f, "SOCKS5"),
        }
    }
}

impl ProxyType {
    pub const ALL: [ProxyType; 5] = [
        ProxyType::None,
        ProxyType::Http,
        ProxyType::Https,
        ProxyType::Socks4,
        ProxyType::Socks5,
    ];

    pub fn prefix(&self) -> &'static str {
        match self {
            ProxyType::None => "",
            ProxyType::Http => "http://",
            ProxyType::Https => "https://",
            ProxyType::Socks4 => "socks4://",
            ProxyType::Socks5 => "socks5://",
        }
    }

    pub fn format_proxy(&self, proxy: &str) -> String {
        if proxy.contains("://") {
            proxy.to_string()
        } else {
            format!("{}{}", self.prefix(), proxy)
        }
    }
}
