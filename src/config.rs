use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub refresh_interval: u64,
    pub symbols: Vec<String>,
    pub binance_api_url: String,
    pub theme: ThemeConfig,
    pub trading_pairs: TradingPairsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingPairsConfig {
    pub default_pairs: Vec<String>,
    pub custom_pairs: Vec<String>,
    pub max_display_pairs: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 20,
            symbols: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
                "BNBUSDT".to_string(),
            ],
            binance_api_url: "https://api.binance.com".to_string(),
            theme: ThemeConfig {
                primary: "#00ff00".to_string(),
                secondary: "#ffff00".to_string(),
                background: "#000000".to_string(),
                text: "#ffffff".to_string(),
            },
            trading_pairs: TradingPairsConfig {
                default_pairs: vec![
                    "BTCUSDT".to_string(),
                    "ETHUSDT".to_string(),
                    "BNBUSDT".to_string(),
                    "ADAUSDT".to_string(),
                    "DOTUSDT".to_string(),
                    "LINKUSDT".to_string(),
                    "LTCUSDT".to_string(),
                    "XRPUSDT".to_string(),
                ],
                custom_pairs: vec![],
                max_display_pairs: 20,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("binance-portal-tui");

        let config = Config::builder()
            .add_source(File::from(config_dir.join("config.toml")).required(false))
            .add_source(Environment::with_prefix("BINANCE_PORTAL"))
            .build()?;

        config.try_deserialize()
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("binance-portal-tui");
        
        std::fs::create_dir_all(&config_dir)?;
        
        let config_str = toml::to_string_pretty(self)?;
        std::fs::write(config_dir.join("config.toml"), config_str)?;
        
        Ok(())
    }

    pub fn get_all_symbols(&self) -> Vec<String> {
        let mut all_symbols = self.trading_pairs.default_pairs.clone();
        all_symbols.extend(self.trading_pairs.custom_pairs.clone());
        
        // 去重并限制数量
        let mut unique_symbols = Vec::new();
        for symbol in all_symbols {
            if !unique_symbols.contains(&symbol) && unique_symbols.len() < self.trading_pairs.max_display_pairs {
                unique_symbols.push(symbol);
            }
        }
        
        unique_symbols
    }

    pub fn add_custom_pair(&mut self, symbol: String) -> bool {
        let symbol_upper = symbol.to_uppercase();
        
        // 检查是否已经存在
        if self.trading_pairs.custom_pairs.contains(&symbol_upper) {
            return false;
        }
        
        // 检查是否超过最大数量
        if self.get_all_symbols().len() >= self.trading_pairs.max_display_pairs {
            return false;
        }
        
        self.trading_pairs.custom_pairs.push(symbol_upper);
        true
    }

    pub fn remove_custom_pair(&mut self, symbol: &str) -> bool {
        let symbol_upper = symbol.to_uppercase();
        if let Some(index) = self.trading_pairs.custom_pairs.iter().position(|s| s == &symbol_upper) {
            self.trading_pairs.custom_pairs.remove(index);
            true
        } else {
            false
        }
    }
} 