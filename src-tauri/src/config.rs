use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    #[serde(rename = "socks5")]
    Socks5,
    #[serde(rename = "http")]
    Http,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Socks5
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub protocol: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            protocol: ProxyType::Socks5,
            host: "127.0.0.1".to_string(),
            port: 10808,
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchType {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "prefix")]
    Prefix,
    #[serde(rename = "wildcard")]
    Wildcard,
}

impl Default for MatchType {
    fn default() -> Self {
        MatchType::Exact
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRule {
    pub id: String,
    pub name: String,
    pub path: String,
    pub match_type: MatchType,
    pub include_children: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub bypass_lan: bool,
    pub block_udp_443: bool,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            bypass_lan: true,
            block_udp_443: true,
            auto_start: false,
            minimize_to_tray: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DomainMatchType {
    #[serde(rename = "exact")]
    Exact,       // 完整精确匹配，如 api.github.com
    #[serde(rename = "suffix")]
    Suffix,      // 后缀 / 主域名匹配，如 github.com 或 .github.com
    #[serde(rename = "keyword")]
    Keyword,     // 包含关键字，如 google
    #[serde(rename = "prefix")]
    Prefix,      // 前缀匹配，如 api.
}

impl Default for DomainMatchType {
    fn default() -> Self {
        DomainMatchType::Suffix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRule {
    pub id: String,
    pub pattern: String,
    pub match_type: DomainMatchType,
    pub enabled: bool,
}

impl DomainRule {
    pub fn matches(&self, domain: &str) -> bool {
        if !self.enabled || self.pattern.trim().is_empty() {
            return false;
        }
        let domain_lower = domain.to_lowercase();
        let pattern_lower = self.pattern.trim().to_lowercase();

        match self.match_type {
            DomainMatchType::Exact => domain_lower == pattern_lower,
            DomainMatchType::Suffix => {
                let p = pattern_lower.trim_start_matches('.');
                domain_lower == p || domain_lower.ends_with(&format!(".{}", p))
            }
            DomainMatchType::Keyword => domain_lower.contains(&pattern_lower),
            DomainMatchType::Prefix => domain_lower.starts_with(&pattern_lower),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub proxy: ProxyConfig,
    pub rules: Vec<AppRule>,
    #[serde(default)]
    pub domain_rules: Vec<DomainRule>,
    pub settings: AppSettings,
}

impl AppConfig {
    /// Per-user application directory; also holds the extracted WinDivert files.
    pub fn app_dir() -> PathBuf {
        let dir = dirs::config_dir()
            .map(|d| d.join("Marga"))
            .unwrap_or_else(|| PathBuf::from("."));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn config_path() -> PathBuf {
        Self::app_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }
}
