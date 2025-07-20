use crate::{
    api::{BinanceApi, KlineData, TickerPrice},
    config::AppConfig,
};
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum InputMode {
    Normal,
    AddingPair,
}

pub struct App {
    pub config: AppConfig,
    pub api: BinanceApi,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub ticker_prices: HashMap<String, TickerPrice>,
    pub kline_data: HashMap<String, Vec<KlineData>>,
    pub selected_symbol: Option<String>,
    pub last_refresh: Instant,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let api = BinanceApi::new(config.binance_api_url.clone());
        
        Self {
            config,
            api,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            ticker_prices: HashMap::new(),
            kline_data: HashMap::new(),
            selected_symbol: None,
            last_refresh: Instant::now(),
            should_quit: false,
        }
    }

    pub async fn refresh_data(&mut self) -> Result<()> {
        let symbols = self.config.get_all_symbols();
        
        // 获取价格数据
        let prices = self.api.get_ticker_prices(&symbols).await?;
        self.ticker_prices.extend(prices);
        
        // 获取K线数据 (5分钟周期)
        for symbol in &symbols {
            let klines = self.api.get_klines(symbol, "5m", 100).await?;
            self.kline_data.insert(symbol.clone(), klines);
        }
        
        self.last_refresh = Instant::now();
        Ok(())
    }

    pub fn select_symbol(&mut self, symbol: String) {
        self.selected_symbol = Some(symbol);
    }

    pub fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= Duration::from_secs(self.config.refresh_interval)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn get_symbols(&self) -> Vec<String> {
        self.config.get_all_symbols()
    }

    pub fn add_custom_pair(&mut self, symbol: String) -> bool {
        let success = self.config.add_custom_pair(symbol);
        if success {
            // 保存配置到文件
            if let Err(e) = self.save_config() {
                eprintln!("保存配置失败: {}", e);
            }
        }
        success
    }

    pub fn remove_custom_pair(&mut self, symbol: &str) -> bool {
        let success = self.config.remove_custom_pair(symbol);
        if success {
            // 保存配置到文件
            if let Err(e) = self.save_config() {
                eprintln!("保存配置失败: {}", e);
            }
        }
        success
    }

    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.save()
    }

    pub fn enter_input_mode(&mut self) {
        self.input_mode = InputMode::AddingPair;
        self.input_buffer.clear();
    }

    pub fn exit_input_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn add_input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn remove_input_char(&mut self) {
        self.input_buffer.pop();
    }

    pub fn submit_input(&mut self) -> bool {
        if !self.input_buffer.is_empty() {
            let success = self.add_custom_pair(self.input_buffer.clone());
            self.exit_input_mode();
            success
        } else {
            self.exit_input_mode();
            false
        }
    }
} 