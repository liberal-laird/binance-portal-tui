use crate::{
    app::App,
    api::KlineData,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Table, Row},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ].as_ref())
        .split(f.size());

    let main_area = chunks[0];
    let footer_area = chunks[1];

    // 主内容区域
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(main_area);

    // 左侧区域分为交易对列表和输入区域
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ].as_ref())
        .split(main_chunks[0]);

    draw_symbol_table(f, app, left_chunks[0]);
    draw_input_area(f, app, left_chunks[1]);
    draw_kline_chart(f, app, main_chunks[1]);
    
    // Footer 显示按键说明
    draw_footer(f, footer_area);
}

fn draw_symbol_table(f: &mut Frame, app: &App, area: Rect) {
    let symbols = app.get_symbols();
    let mut rows = Vec::new();

    for symbol in symbols {
        if let Some(price) = app.ticker_prices.get(&symbol) {
            let price_change = price.price_change.parse::<f64>().unwrap_or(0.0);
            let price_change_percent = price.price_change_percent.parse::<f64>().unwrap_or(0.0);
            let color = if price_change >= 0.0 { Color::Green } else { Color::Red };
            
            let row = Row::new(vec![
                symbol.clone(),
                format!("{:.2}%", price_change_percent),
                price.price.clone(),
            ]).style(Style::default().fg(color));
            
            // 如果是选中的交易对，添加高亮
            if app.selected_symbol.as_ref() == Some(&symbol) {
                rows.push(row.style(Style::default().fg(color).add_modifier(Modifier::BOLD)));
            } else {
                rows.push(row);
            }
        } else {
            let row = Row::new(vec![
                symbol.clone(),
                "0.00%".to_string(),
                "加载中...".to_string(),
            ]).style(Style::default().fg(Color::Yellow));
            rows.push(row);
        }
    }

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["交易对", "涨跌幅", "价格"]))
        .block(Block::default().borders(Borders::ALL).title("交易对列表"))
        .style(Style::default().fg(Color::White));

    f.render_widget(table, area);
}

fn draw_input_area(f: &mut Frame, app: &App, area: Rect) {
    let input_text = match app.input_mode {
        crate::app::InputMode::AddingPair => {
            format!("添加交易对: {}_", app.input_buffer)
        }
        crate::app::InputMode::Normal => {
            "按 'A' 添加交易对".to_string()
        }
    };

    let style = match app.input_mode {
        crate::app::InputMode::AddingPair => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        crate::app::InputMode::Normal => Style::default().fg(Color::Gray),
    };

    let paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title("交易对输入"))
        .style(style);

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = "Q:退出  R:刷新  1-5:快速选择  ↑↓:导航  A:添加交易对  D:删除交易对  S:保存配置  Enter:确认";
    
    let paragraph = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    
    f.render_widget(paragraph, area);
}

fn draw_kline_chart(f: &mut Frame, app: &App, area: Rect) {
    if let Some(selected_symbol) = &app.selected_symbol {
        // 分割区域：上方信息栏，下方K线图
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 信息栏
                Constraint::Min(0),     // K线图
            ].as_ref())
            .split(area);

        let info_area = chunks[0];
        let chart_area = chunks[1];

        // 绘制信息栏
        draw_symbol_info(f, app, selected_symbol, info_area);

        // 绘制K线图
        if let Some(klines) = app.kline_data.get(selected_symbol) {
            draw_candlestick_chart(f, klines, selected_symbol, chart_area);
        } else {
            let paragraph = Paragraph::new("加载K线数据中...")
                .block(Block::default().borders(Borders::ALL).title(format!("K线图 - {}", selected_symbol)));
            f.render_widget(paragraph, chart_area);
        }
    } else {
        let paragraph = Paragraph::new("请选择一个交易对查看K线图")
            .block(Block::default().borders(Borders::ALL).title("K线图"));
        f.render_widget(paragraph, area);
    }
}

fn draw_symbol_info(f: &mut Frame, app: &App, symbol: &str, area: Rect) {
    if let Some(price) = app.ticker_prices.get(symbol) {
        let price_change = price.price_change.parse::<f64>().unwrap_or(0.0);
        let color = if price_change >= 0.0 { Color::Green } else { Color::Red };
        
        let info_text = format!(
            "{} | 价格: {} | 涨跌: {} ({:.2}%) | 24h高: {} | 24h低: {}",
            symbol,
            price.price,
            price.price_change,
            price.price_change_percent.parse::<f64>().unwrap_or(0.0),
            price.high_24h,
            price.low_24h
        );

        let paragraph = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("交易对信息"))
            .style(Style::default().fg(color));

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new(format!("{} | 加载价格信息中...", symbol))
            .block(Block::default().borders(Borders::ALL).title("交易对信息"))
            .style(Style::default().fg(Color::Yellow));

        f.render_widget(paragraph, area);
    }
}

fn draw_candlestick_chart(f: &mut Frame, klines: &[KlineData], symbol: &str, area: Rect) {
    if klines.is_empty() {
        let paragraph = Paragraph::new("暂无K线数据")
            .block(Block::default().borders(Borders::ALL).title(format!("K线图 - {}", symbol)));
        f.render_widget(paragraph, area);
        return;
    }

    // 计算价格范围
    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;
    
    for kline in klines {
        let high = kline.high.parse::<f64>().unwrap_or(0.0);
        let low = kline.low.parse::<f64>().unwrap_or(0.0);
        min_price = min_price.min(low);
        max_price = max_price.max(high);
    }

    let price_range = max_price - min_price;
    let chart_height = area.height.saturating_sub(2) as f64;
    let chart_width = area.width.saturating_sub(2) as f64;

    for (i, kline) in klines.iter().enumerate() {
        let open = kline.open.parse::<f64>().unwrap_or(0.0);
        let close = kline.close.parse::<f64>().unwrap_or(0.0);
        let high = kline.high.parse::<f64>().unwrap_or(0.0);
        let low = kline.low.parse::<f64>().unwrap_or(0.0);
        
        let is_green = close >= open;
        let color = if is_green { Color::Green } else { Color::Red };
        
        let x = (i as f64 / klines.len() as f64 * chart_width) as u16;
        let open_y = ((max_price - open) / price_range * chart_height) as u16;
        let close_y = ((max_price - close) / price_range * chart_height) as u16;
        let high_y = ((max_price - high) / price_range * chart_height) as u16;
        let low_y = ((max_price - low) / price_range * chart_height) as u16;
        
        // 绘制K线
        let candle_char = if is_green { "█" } else { "█" };
        let wick_char = "│";
        
        // 绘制影线
        for y in low_y..=high_y {
            if y < area.height.saturating_sub(2) {
                f.render_widget(
                    Paragraph::new(wick_char).style(Style::default().fg(color)),
                    Rect::new(area.x + x, area.y + y + 1, 1, 1),
                );
            }
        }
        
        // 绘制实体
        let start_y = open_y.min(close_y);
        let end_y = open_y.max(close_y);
        for y in start_y..=end_y {
            if y < area.height.saturating_sub(2) {
                f.render_widget(
                    Paragraph::new(candle_char).style(Style::default().fg(color)),
                    Rect::new(area.x + x, area.y + y + 1, 1, 1),
                );
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("K线图 - {} (5分钟周期)", symbol));
    f.render_widget(block, area);
} 