use clap::Parser;
use common::{ChatKey, ChatType, ClientMessage, MessagePayload, ServerMessage};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::TlsConnector;

mod ui;
use ui::*;

#[derive(Parser, Debug)]
#[command(name = "Rchat Client")]
#[command(about = "Client E2EE per Rchat", long_about = None)]
struct Args {
    /// Indirizzo IP del server
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Porta del server
    #[arg(short, long, default_value_t = 6666)]
    port: u16,

    /// Username
    #[arg(short, long)]
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup terminale
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Connetti al server
    let addr = format!("{}:{}", args.host, args.port);
    
    terminal.draw(|f| {
        let area = f.area();
        let msg = Paragraph::new(format!("🔌 Connessione a {}...", addr))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Rchat"));
        f.render_widget(msg, area);
    })?;

    let stream = match TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            cleanup_terminal(&mut terminal)?;
            eprintln!("❌ Errore di connessione: {}", e);
            return Err(e.into());
        }
    };

    // Setup TLS
    let config = configure_tls()?;
    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(args.host.clone())?;

    let stream = match connector.connect(server_name, stream).await {
        Ok(s) => s,
        Err(e) => {
            cleanup_terminal(&mut terminal)?;
            eprintln!("❌ Errore TLS handshake: {}", e);
            return Err(e.into());
        }
    };

    let app = App::new(args.username.clone());
    let result = run_app(&mut terminal, app, stream).await;

    cleanup_terminal(&mut terminal)?;

    if let Err(err) = result {
        eprintln!("❌ Errore: {}", err);
    }

    Ok(())
}

fn cleanup_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn configure_tls() -> Result<ClientConfig, Box<dyn std::error::Error>> {
    use rustls::ClientConfig;
    use rustls::RootCertStore;
    use rustls_pemfile::certs;
    use std::fs::File;
    use std::io::BufReader;

    let mut root_store = RootCertStore::empty();

    // Carica certificato del server (per demo, accetta self-signed)
    let cert_path = "server.crt";
    if std::path::Path::new(cert_path).exists() {
        let cert_file = File::open(cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
        
        for cert in certs {
            root_store.add(cert)?;
        }
    } else {
        // Per demo, usa webpki-roots (certificati pubblici)
        // In produzione dovresti validare il certificato del server
        eprintln!("⚠️  Certificato server non trovato, usando modalità insicura");
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

async fn run_app<W>(
    terminal: &mut Terminal<W>,
    mut app: App,
    stream: tokio_rustls::client::TlsStream<TcpStream>,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: ratatui::backend::Backend,
{
    let (mut read_half, mut write_half) = tokio::io::split(stream);
    let (tx, mut rx) = mpsc::channel::<ClientMessage>(100);
    let (server_tx, mut server_rx) = mpsc::channel::<ServerMessage>(100);

    // Task per inviare messaggi al server
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(data) = bincode::serialize(&msg) {
                let len = data.len() as u32;
                if write_half.write_all(&len.to_be_bytes()).await.is_err() {
                    break;
                }
                if write_half.write_all(&data).await.is_err() {
                    break;
                }
                let _ = write_half.flush().await;
            }
        }
    });

    // Task per ricevere messaggi dal server
    tokio::spawn(async move {
        loop {
            let mut len_buf = [0u8; 4];
            if read_half.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            let msg_len = u32::from_be_bytes(len_buf) as usize;

            if msg_len == 0 || msg_len > 1024 * 1024 {
                break;
            }

            let mut msg_buf = vec![0u8; msg_len];
            if read_half.read_exact(&mut msg_buf).await.is_err() {
                break;
            }

            if let Ok(msg) = bincode::deserialize::<ServerMessage>(&msg_buf) {
                let _ = server_tx.send(msg).await;
            }
        }
    });

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Gestisci eventi
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    AppMode::Welcome => match key.code {
                        KeyCode::Char('1') => {
                            app.mode = AppMode::CreateChat;
                            app.input.clear();
                        }
                        KeyCode::Char('2') => {
                            app.mode = AppMode::JoinChat;
                            app.input.clear();
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    AppMode::CreateChat => match key.code {
                        KeyCode::Char('1') => {
                            tx.send(ClientMessage::CreateChat {
                                chat_type: ChatType::OneToOne,
                                username: app.username.clone(),
                            })
                            .await?;
                            app.mode = AppMode::WaitingForChatCode;
                        }
                        KeyCode::Char('2') => {
                            tx.send(ClientMessage::CreateChat {
                                chat_type: ChatType::Group {
                                    max_participants: 8,
                                },
                                username: app.username.clone(),
                            })
                            .await?;
                            app.mode = AppMode::WaitingForChatCode;
                        }
                        KeyCode::Esc => {
                            app.mode = AppMode::Welcome;
                        }
                        _ => {}
                    },
                    AppMode::JoinChat => match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            let chat_code = app.input.clone();
                            app.input.clear();
                            tx.send(ClientMessage::JoinChat {
                                chat_code,
                                username: app.username.clone(),
                            })
                            .await?;
                            app.mode = AppMode::WaitingForChatCode;
                        }
                        KeyCode::Esc => {
                            app.mode = AppMode::Welcome;
                        }
                        _ => {}
                    },
                    AppMode::Chat => match key.code {
                        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if c == 'c' {
                                return Ok(());
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                let content = app.input.clone();
                                app.input.clear();

                                // Cripta il messaggio
                                if let Some(ref chat_code) = app.current_chat_code {
                                    if let Some(ref key) = app.chat_key {
                                        let payload =
                                            MessagePayload::new(app.username.clone(), content);
                                        if let Ok(serialized) = bincode::serialize(&payload) {
                                            if let Ok(encrypted) = key.encrypt(&serialized) {
                                                tx.send(ClientMessage::SendMessage {
                                                    chat_code: chat_code.clone(),
                                                    encrypted_payload: encrypted,
                                                })
                                                .await?;

                                                // Aggiungi ai messaggi locali
                                                app.messages.push(ChatMessage {
                                                    username: payload.username.clone(),
                                                    content: payload.content.clone(),
                                                    timestamp: payload.timestamp,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            if let Some(ref chat_code) = app.current_chat_code {
                                tx.send(ClientMessage::LeaveChat {
                                    chat_code: chat_code.clone(),
                                })
                                .await?;
                            }
                            app.mode = AppMode::Welcome;
                            app.current_chat_code = None;
                            app.chat_key = None;
                            app.messages.clear();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        // Gestisci messaggi dal server
        while let Ok(msg) = server_rx.try_recv() {
            match msg {
                ServerMessage::ChatCreated {
                    chat_code,
                    chat_type: _,
                } => {
                    app.current_chat_code = Some(chat_code.clone());
                    app.chat_key = ChatKey::derive_from_code(&chat_code).ok();
                    app.mode = AppMode::Chat;
                    app.status_message = format!("Chat creata! Codice: {}", chat_code);
                }
                ServerMessage::JoinedChat {
                    chat_code,
                    chat_type: _,
                    participant_count,
                } => {
                    app.current_chat_code = Some(chat_code.clone());
                    app.chat_key = ChatKey::derive_from_code(&chat_code).ok();
                    app.mode = AppMode::Chat;
                    app.status_message = format!(
                        "Entrato nella chat! Partecipanti: {}",
                        participant_count
                    );
                }
                ServerMessage::Error { message } => {
                    app.status_message = format!("Errore: {}", message);
                    app.mode = AppMode::Welcome;
                }
                ServerMessage::MessageReceived {
                    encrypted_payload, ..
                } => {
                    if let Some(ref key) = app.chat_key {
                        if let Ok(decrypted) = key.decrypt(&encrypted_payload) {
                            if let Ok(payload) =
                                bincode::deserialize::<MessagePayload>(&decrypted)
                            {
                                app.messages.push(ChatMessage {
                                    username: payload.username.clone(),
                                    content: payload.content.clone(),
                                    timestamp: payload.timestamp,
                                });
                            }
                        }
                    }
                }
                ServerMessage::UserJoined { username, .. } => {
                    app.status_message = format!("{} si è unito", username);
                }
                ServerMessage::UserLeft { username, .. } => {
                    app.status_message = format!("{} ha lasciato", username);
                }
            }
        }
    }
}
