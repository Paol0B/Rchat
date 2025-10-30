use common::{ChatKey, IdentityKey, ChainKey};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use zeroize::Zeroize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    pub verified: bool, // Message signature verified
    pub sent: bool,     // Message successfully sent to server
    pub message_id: Option<String>, // Unique ID for tracking
}

/// Pending message waiting for ACK
#[derive(Clone)]
pub struct PendingMessage {
    pub message_id: String,
    pub room_id: String,
    pub encrypted_payload: Vec<u8>,
    pub sent_at: std::time::Instant,
    pub retry_count: u8,
}

pub struct App {
    pub username: String,
    pub mode: AppMode,
    pub input: String,
    pub current_chat_code: Option<String>,
    pub pending_chat_code: Option<String>, // Codice chat generato localmente in attesa di conferma
    pub chat_key: Option<ChatKey>,
    pub identity_key: IdentityKey,        // Ed25519 keypair for signing
    pub chain_key: Option<ChainKey>,      // Forward secrecy chain
    pub sequence_number: u64,             // Message counter
    pub messages: Vec<ChatMessage>,
    pub status_message: String,
    pub scroll_offset: usize,
    pub numeric_codes: bool, // Usa codici numerici invece di base64
    pub user_left_at: Option<std::time::Instant>, // Timestamp when a user left
    pub closing_in_seconds: Option<u8>,   // Countdown for auto-close
    pub pending_messages: Vec<PendingMessage>, // Messages waiting for ACK
}

impl App {
    pub fn new(username: String, numeric_codes: bool) -> Self {
        Self {
            username,
            mode: AppMode::Welcome,
            input: String::new(),
            current_chat_code: None,
            pending_chat_code: None,
            chat_key: None,
            identity_key: IdentityKey::generate(),
            chain_key: None,
            sequence_number: 0,
            messages: Vec::new(),
            status_message: String::new(),
            scroll_offset: 0,
            numeric_codes,
            user_left_at: None,
            closing_in_seconds: None,
            pending_messages: Vec::new(),
        }
    }

    pub fn scroll_up(&mut self) {
        // Scroll up = aumenta offset = vai verso i messaggi più vecchi
        if !self.messages.is_empty() {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }
    }

    pub fn scroll_down(&mut self) {
        // Scroll down = diminuisci offset = vai verso i messaggi più recenti
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        // Bottom = offset 0 = mostra gli ultimi messaggi
        self.scroll_offset = 0;
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Cleanup: zeroizza dati sensibili
        self.input.zeroize();
        self.messages.clear();
    }
}

/// Generate a consistent color for a username based on its hash
fn username_color(username: &str) -> Color {
    let mut hasher = DefaultHasher::new();
    username.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Use hash to select from a palette of distinguishable colors
    // Avoid black, white, and colors too similar to system colors
    let colors = [
        Color::Cyan,
        Color::Green,
        Color::Blue,
        Color::Magenta,
        Color::Red,
        Color::LightCyan,
        Color::LightGreen,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightRed,
        Color::Yellow,
        Color::LightYellow,
    ];
    
    colors[(hash as usize) % colors.len()]
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
    let ascii_art = [
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
    let menu_items = [
        "Press:",
        "",
        "[1] Create new chat",
        "[2] Join a chat",
        "[Q] Quit",
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
    let footer = Paragraph::new("⚠️  All messages are volatile and never persisted")
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

    let menu_items = [
        "Select chat type:",
        "",
        "[1] 1:1 Chat (max 2 participants)",
        "[2] Group Chat (max 8 participants)",
        "",
        "[ESC] Go back",
    ];

    let menu_lines: Vec<Line> = menu_items
        .iter()
        .map(|s| Line::from(Span::styled(*s, Style::default().fg(Color::White))))
        .collect();

    let menu = Paragraph::new(menu_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Create Chat"));
    f.render_widget(menu, chunks[0]);

    let footer = Paragraph::new("A secure code will be generated to share")
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

    let instructions = Paragraph::new("Enter the chat code:")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Join"));
    f.render_widget(instructions, chunks[0]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Code"));
    f.render_widget(input, chunks[1]);

    let footer = Paragraph::new("[ENTER] Confirm | [CTRL+V / SHIFT+INS / Right Click] Paste | [ESC] Cancel")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_waiting(f: &mut Frame) {
    let area = f.area();
    let msg = Paragraph::new("⏳ Waiting for server response...")
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
    // scroll_offset = 0 significa mostra gli ultimi messaggi (bottom)
    // scroll_offset > 0 significa scroll up verso i messaggi più vecchi
    let start_idx = if total_messages > message_area_height {
        // Se ci sono più messaggi dell'area disponibile
        let max_offset = total_messages.saturating_sub(message_area_height);
        let actual_offset = app.scroll_offset.min(max_offset);
        max_offset.saturating_sub(actual_offset)
    } else {
        // Se ci sono meno messaggi, mostra tutti dall'inizio
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
            
            // Determine message status and colors
            let (status_mark, status_color, content_color) = if !m.sent {
                ("✗", Color::Red, Color::Red)  // Not sent
            } else if m.verified {
                ("✓", Color::White, Color::White)  // Sent and verified
            } else {
                ("⚠", Color::Yellow, Color::Yellow)  // Sent but not verified
            };
            
            let user_color = username_color(&m.username);
            
            // Create a line with colored spans
            let mut spans = vec![
                Span::styled(format!("[{}] ", time), Style::default().fg(Color::Gray)),
                Span::styled(format!("{} ", status_mark), Style::default().fg(status_color)),
            ];
            
            // Add "NOT SENT" label for failed messages
            if !m.sent {
                spans.push(Span::styled("[NOT SENT] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            }
            
            spans.extend(vec![
                Span::styled("<", Style::default().fg(Color::Gray)),
                Span::styled(m.username.clone(), Style::default().fg(user_color).add_modifier(Modifier::BOLD)),
                Span::styled(">: ", Style::default().fg(Color::Gray)),
                Span::styled(m.content.clone(), Style::default().fg(content_color)),
            ]);
            
            let line = Line::from(spans);
            
            ListItem::new(line)
        })
        .collect();

    let scroll_indicator = if total_messages > message_area_height && app.scroll_offset > 0 {
        format!(" (↑ {} older messages)", app.scroll_offset)
    } else {
        String::new()
    };

    let messages_list = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Messages (E2EE){}", scroll_indicator)),
    );
    f.render_widget(messages_list, chunks[1]);

    // Input
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Message"));
    f.render_widget(input, chunks[2]);

    // Footer
    let (footer_text, footer_color) = if let Some(seconds) = app.closing_in_seconds {
        (
            format!("⚠️  CHAT CLOSING IN {} SECONDS - Other user left ⚠️", seconds),
            Color::Red
        )
    } else if app.status_message.is_empty() {
        (
            "[ENTER] Send | [↑↓] Scroll | [ESC] Exit | [CTRL+C] Terminate".to_string(),
            Color::Gray
        )
    } else {
        (app.status_message.clone(), Color::Gray)
    };
    
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(footer_color).add_modifier(Modifier::BOLD))
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
