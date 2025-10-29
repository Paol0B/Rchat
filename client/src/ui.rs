use common::ChatKey;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Welcome,
    CreateChat,
    JoinChat,
    WaitingForChatCode,
    Chat,
}

pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: i64,
}

pub struct App {
    pub username: String,
    pub mode: AppMode,
    pub input: String,
    pub current_chat_code: Option<String>,
    pub chat_key: Option<ChatKey>,
    pub messages: Vec<ChatMessage>,
    pub status_message: String,
    pub scroll_offset: usize,
}

impl App {
    pub fn new(username: String) -> Self {
        Self {
            username,
            mode: AppMode::Welcome,
            input: String::new(),
            current_chat_code: None,
            chat_key: None,
            messages: Vec::new(),
            status_message: String::new(),
            scroll_offset: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if !self.messages.is_empty() {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Cleanup: zeroizza dati sensibili
        self.input.zeroize();
        self.messages.clear();
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    match app.mode {
        AppMode::Welcome => draw_welcome(f),
        AppMode::CreateChat => draw_create_chat(f),
        AppMode::JoinChat => draw_join_chat(f, app),
        AppMode::WaitingForChatCode => draw_waiting(f),
        AppMode::Chat => draw_chat(f, app),
    }
}

fn draw_welcome(f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    // ASCII Art
    let ascii_art = vec![
        "╦═╗┌─┐┬ ┬┌─┐┌┬┐",
        "╠╦╝│  ├─┤├─┤ │ ",
        "╩╚═└─┘┴ ┴┴ ┴ ┴ ",
        "",
        "🔒 End-to-End Encrypted Chat",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━",
    ];

    let art_lines: Vec<Line> = ascii_art
        .iter()
        .map(|s| {
            Line::from(Span::styled(
                *s,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let art = Paragraph::new(art_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Welcome"));
    f.render_widget(art, chunks[0]);

    // Menu
    let menu_items = vec![
        "Premi:",
        "",
        "[1] Crea nuova chat",
        "[2] Unisciti a una chat",
        "[Q] Esci",
    ];

    let menu_lines: Vec<Line> = menu_items
        .iter()
        .map(|s| Line::from(Span::styled(*s, Style::default().fg(Color::White))))
        .collect();

    let menu = Paragraph::new(menu_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Menu"));
    f.render_widget(menu, chunks[1]);

    // Footer
    let footer = Paragraph::new("⚠️  Tutti i messaggi sono volatili e mai persistiti")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_create_chat(f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(f.area());

    let menu_items = vec![
        "Seleziona tipo di chat:",
        "",
        "[1] Chat 1:1 (max 2 partecipanti)",
        "[2] Chat di gruppo (max 8 partecipanti)",
        "",
        "[ESC] Torna indietro",
    ];

    let menu_lines: Vec<Line> = menu_items
        .iter()
        .map(|s| Line::from(Span::styled(*s, Style::default().fg(Color::White))))
        .collect();

    let menu = Paragraph::new(menu_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Crea Chat"));
    f.render_widget(menu, chunks[0]);

    let footer = Paragraph::new("Verrà generato un codice sicuro da condividere")
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[1]);
}

fn draw_join_chat(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3), Constraint::Length(3)])
        .split(f.area());

    let instructions = Paragraph::new("Inserisci il codice della chat:")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Unisciti"));
    f.render_widget(instructions, chunks[0]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Codice"));
    f.render_widget(input, chunks[1]);

    let footer = Paragraph::new("[ENTER] Conferma | [CTRL+V / SHIFT+INS / Click Destro] Incolla | [ESC] Annulla")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_waiting(f: &mut Frame) {
    let area = f.area();
    let msg = Paragraph::new("⏳ In attesa di risposta dal server...")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Rchat"));
    f.render_widget(msg, area);
}

fn draw_chat(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let chat_code_display = app
        .current_chat_code
        .as_ref()
        .map(|c| format!("Chat: {}", &c[..16.min(c.len())]))
        .unwrap_or_else(|| "Chat".to_string());
    let header = Paragraph::new(format!("🔒 {} | User: {}", chat_code_display, app.username))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Messaggi con scrolling
    let message_area_height = chunks[1].height.saturating_sub(2) as usize; // -2 per i bordi
    let total_messages = app.messages.len();
    
    // Calcola l'offset di visualizzazione
    let start_idx = if total_messages > message_area_height {
        // Se ci sono più messaggi dell'area, mostra gli ultimi
        total_messages.saturating_sub(message_area_height).saturating_sub(app.scroll_offset)
    } else {
        0
    };
    
    let end_idx = (start_idx + message_area_height).min(total_messages);
    
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .map(|m| {
            let time = format_timestamp(m.timestamp);
            let content = format!("[{}] <{}>: {}", time, m.username, m.content);
            ListItem::new(content).style(Style::default().fg(Color::White))
        })
        .collect();

    let scroll_indicator = if total_messages > message_area_height {
        format!(" ({}/{})", start_idx + 1, total_messages.saturating_sub(message_area_height))
    } else {
        String::new()
    };

    let messages_list = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Messaggi (E2EE){}", scroll_indicator)),
    );
    f.render_widget(messages_list, chunks[1]);

    // Input
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Messaggio"));
    f.render_widget(input, chunks[2]);

    // Footer
    let footer_text = if app.status_message.is_empty() {
        "[ENTER] Invia | [↑↓] Scroll | [ESC] Esci | [CTRL+C] Termina".to_string()
    } else {
        app.status_message.clone()
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[3]);
}

fn format_timestamp(timestamp: i64) -> String {
    let dt = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0)
        .unwrap_or_else(|| chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    dt.format("%H:%M").to_string()
}

// Chrono replacement per timestamp formatting
mod chrono {
    pub struct NaiveDateTime {
        timestamp: i64,
    }

    impl NaiveDateTime {
        pub fn from_timestamp_opt(timestamp: i64, _nsecs: u32) -> Option<Self> {
            Some(Self { timestamp })
        }

        pub fn format(&self, _fmt: &str) -> FormattedTime {
            FormattedTime {
                timestamp: self.timestamp,
            }
        }
    }

    pub struct FormattedTime {
        timestamp: i64,
    }

    impl std::fmt::Display for FormattedTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let hours = (self.timestamp / 3600) % 24;
            let minutes = (self.timestamp / 60) % 60;
            write!(f, "{:02}:{:02}", hours, minutes)
        }
    }
}
