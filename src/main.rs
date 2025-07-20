mod app;
mod api;
mod config;
mod event;
mod ui;

use crate::{
    app::App,
    config::AppConfig,
    event::{EventHandler, EventType, setup_terminal, restore_terminal},
    ui::draw,
};
use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = AppConfig::load().unwrap_or_default();
    
    // 设置终端
    setup_terminal()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用
    let mut app = App::new(config);
    let mut event_handler = EventHandler::new(Duration::from_millis(250));

    // 初始数据加载
    app.refresh_data().await?;

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if let Some(event) = event_handler.next_event().await? {
            match event {
                EventType::Input(key) => {
                    match app.input_mode {
                        crate::app::InputMode::AddingPair => {
                            // 输入模式下的键盘处理
                            match key {
                                KeyCode::Char(c) if c.is_alphanumeric() => {
                                    app.add_input_char(c);
                                }
                                KeyCode::Backspace => {
                                    app.remove_input_char();
                                }
                                KeyCode::Enter => {
                                    if app.submit_input() {
                                        println!("交易对添加成功并已保存!");
                                    } else {
                                        println!("添加失败: 可能已存在或超过最大数量");
                                    }
                                }
                                KeyCode::Esc => {
                                    app.exit_input_mode();
                                }
                                _ => {}
                            }
                        }
                        crate::app::InputMode::Normal => {
                            // 正常模式下的键盘处理
                            match key {
                                KeyCode::Char('q') => {
                                    app.quit();
                                }
                                KeyCode::Char('r') | KeyCode::Char(' ') => {
                                    // 手动刷新
                                    if let Err(e) = app.refresh_data().await {
                                        eprintln!("刷新数据失败: {}", e);
                                    }
                                }
                                KeyCode::Char('1') => {
                                    if app.get_symbols().len() > 0 {
                                        app.select_symbol(app.get_symbols()[0].clone());
                                    }
                                }
                                KeyCode::Char('2') => {
                                    if app.get_symbols().len() > 1 {
                                        app.select_symbol(app.get_symbols()[1].clone());
                                    }
                                }
                                KeyCode::Char('3') => {
                                    if app.get_symbols().len() > 2 {
                                        app.select_symbol(app.get_symbols()[2].clone());
                                    }
                                }
                                KeyCode::Char('4') => {
                                    if app.get_symbols().len() > 3 {
                                        app.select_symbol(app.get_symbols()[3].clone());
                                    }
                                }
                                KeyCode::Char('5') => {
                                    if app.get_symbols().len() > 4 {
                                        app.select_symbol(app.get_symbols()[4].clone());
                                    }
                                }
                                KeyCode::Char('a') => {
                                    // 进入添加交易对输入模式
                                    app.enter_input_mode();
                                }
                                KeyCode::Char('d') => {
                                    // 删除自定义交易对
                                    if let Some(selected) = app.selected_symbol.clone() {
                                        if app.remove_custom_pair(&selected) {
                                            println!("已删除交易对: {} 并已保存", selected);
                                        }
                                    }
                                }
                                KeyCode::Char('s') => {
                                    // 手动保存配置
                                    if let Err(e) = app.save_config() {
                                        println!("保存配置失败: {}", e);
                                    } else {
                                        println!("配置已保存");
                                    }
                                }
                                KeyCode::Up => {
                                    // 向上选择交易对
                                    if let Some(current) = &app.selected_symbol {
                                        let symbols = app.get_symbols();
                                        if let Some(current_index) = symbols.iter().position(|s| s == current) {
                                            if current_index > 0 {
                                                app.select_symbol(symbols[current_index - 1].clone());
                                            }
                                        }
                                    } else if !app.get_symbols().is_empty() {
                                        app.select_symbol(app.get_symbols()[0].clone());
                                    }
                                }
                                KeyCode::Down => {
                                    // 向下选择交易对
                                    if let Some(current) = &app.selected_symbol {
                                        let symbols = app.get_symbols();
                                        if let Some(current_index) = symbols.iter().position(|s| s == current) {
                                            if current_index < symbols.len() - 1 {
                                                app.select_symbol(symbols[current_index + 1].clone());
                                            }
                                        }
                                    } else if !app.get_symbols().is_empty() {
                                        app.select_symbol(app.get_symbols()[0].clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                EventType::Tick => {
                    // 检查是否需要刷新数据
                    if app.should_refresh() {
                        if let Err(e) = app.refresh_data().await {
                            eprintln!("自动刷新数据失败: {}", e);
                        }
                    }
                }
                EventType::Refresh => {
                    // 手动刷新
                    if let Err(e) = app.refresh_data().await {
                        eprintln!("刷新数据失败: {}", e);
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // 恢复终端
    restore_terminal()?;
    Ok(())
}
