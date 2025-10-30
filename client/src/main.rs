use clap::Parser;
use common::{ChatKey, ChainKey, IdentityKey, ChatType, ClientMessage, MessagePayload, ServerMessage, chat_code_to_room_id, generate_chat_code, generate_numeric_chat_code};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
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

/// Disabilita l'echo del terminale su Windows
/// Su Windows, crossterm::enable_raw_mode() non disabilita completamente l'echo,
/// quindi dobbiamo farlo manualmente usando le API Windows
#[cfg(windows)]
fn disable_windows_echo() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::windows::io::AsRawHandle;
    use std::io::stdin;
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, SetConsoleMode, 
        ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT
    };

    unsafe {
        let handle = stdin().as_raw_handle() as isize;
        let mut mode: u32 = 0;
        
        if GetConsoleMode(handle, &mut mode) != 0 {
            // Disabilita ENABLE_ECHO_INPUT per evitare la doppia visualizzazione
            mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
            if SetConsoleMode(handle, mode) == 0 {
                return Err("Failed to set console mode".into());
            }
        } else {
            return Err("Failed to get console mode".into());
        }
    }
    
    Ok(())
}

#[cfg(not(windows))]
fn disable_windows_echo() -> Result<(), Box<dyn std::error::Error>> {
    // Non necessario su Unix/Linux
    Ok(())
}

/// Copia testo nella clipboard
fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}

/// Legge testo dalla clipboard
fn get_clipboard_text() -> Result<String, Box<dyn std::error::Error>> {
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new()?;
    Ok(clipboard.get_text()?)
}

#[derive(Parser, Debug)]
#[command(name = "Rchat Client")]
#[command(about = "E2EE Client for Rchat", long_about = None)]
struct Args {
    /// Server IP address
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(short, long, default_value_t = 6666)]
    port: u16,

    /// Username
    #[arg(short, long)]
    username: String,

    /// Accept self-signed certificates (INSECURE, testing only!)
    #[arg(long, default_value_t = false)]
    insecure: bool,

    /// Use 6-digit numeric codes instead of long base64 codes
    /// WARNING: Less secure (20 bit vs 512 bit entropy)
    #[arg(long, default_value_t = false)]
    numeric_codes: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup terminale
    enable_raw_mode()?;
    disable_windows_echo()?; // Fix per doppio carattere su Windows
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
            eprintln!("❌ Connection error: {}", e);
            return Err(e.into());
        }
    };

    // Setup TLS
    let config = configure_tls(args.insecure)?;
    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(args.host.clone())?;

    let stream = match connector.connect(server_name, stream).await {
        Ok(s) => s,
        Err(e) => {
            cleanup_terminal(&mut terminal)?;
            eprintln!("❌ TLS handshake error: {}", e);
            return Err(e.into());
        }
    };

    let app = App::new(args.username.clone(), args.numeric_codes);
    let result = run_app(&mut terminal, app, stream).await;

    cleanup_terminal(&mut terminal)?;

    if let Err(err) = result {
        eprintln!("❌ Error: {}", err);
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

fn configure_tls(insecure: bool) -> Result<ClientConfig, Box<dyn std::error::Error>> {
    use rustls::ClientConfig;
    use rustls::RootCertStore;
    use rustls_pemfile::certs;
    use std::fs::File;
    use std::io::BufReader;

    let mut root_store = RootCertStore::empty();

    if insecure {
        // Modalità insicura: accetta qualsiasi certificato (solo per testing!)
        eprintln!("⚠️  INSECURE MODE: Accepting self-signed certificates");
        eprintln!("⚠️  DO NOT use in production!");
        
        use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
        use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
        use rustls::DigitallySignedStruct;
        
        #[derive(Debug)]
        struct NoVerifier;
        
        impl ServerCertVerifier for NoVerifier {
            fn verify_server_cert(
                &self,
                _end_entity: &CertificateDer<'_>,
                _intermediates: &[CertificateDer<'_>],
                _server_name: &ServerName<'_>,
                _ocsp_response: &[u8],
                _now: UnixTime,
            ) -> Result<ServerCertVerified, rustls::Error> {
                Ok(ServerCertVerified::assertion())
            }

            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, rustls::Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &CertificateDer<'_>,
                _dss: &DigitallySignedStruct,
            ) -> Result<HandshakeSignatureValid, rustls::Error> {
                Ok(HandshakeSignatureValid::assertion())
            }

            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                vec![
                    rustls::SignatureScheme::RSA_PKCS1_SHA1,
                    rustls::SignatureScheme::ECDSA_SHA1_Legacy,
                    rustls::SignatureScheme::RSA_PKCS1_SHA256,
                    rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                    rustls::SignatureScheme::RSA_PKCS1_SHA384,
                    rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
                    rustls::SignatureScheme::RSA_PKCS1_SHA512,
                    rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
                    rustls::SignatureScheme::RSA_PSS_SHA256,
                    rustls::SignatureScheme::RSA_PSS_SHA384,
                    rustls::SignatureScheme::RSA_PSS_SHA512,
                    rustls::SignatureScheme::ED25519,
                    rustls::SignatureScheme::ED448,
                ]
            }
        }
        
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();
        
        return Ok(config);
    }

    // Carica certificato del server (per demo, accetta self-signed)
    let cert_path = "server.crt";
    if std::path::Path::new(cert_path).exists() {
        let cert_file = File::open(cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
        
        for cert in certs {
            root_store.add(cert)?;
        }
        
        eprintln!("✅ Server certificate loaded from {}", cert_path);
    } else {
        eprintln!("⚠️  Server certificate not found at {}", cert_path);
        eprintln!("⚠️  Use --insecure to accept self-signed certificates");
        return Err("Certificato server mancante. Usa --insecure per testing.".into());
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

        // Check for pending messages that need retry
        let now = std::time::Instant::now();
        let mut messages_to_retry = Vec::new();
        
        app.pending_messages.retain(|pm| {
            let elapsed = now.duration_since(pm.sent_at).as_secs();
            if elapsed >= 2 {
                // Timeout - retry if under max retries
                if pm.retry_count < 3 {
                    messages_to_retry.push(pm.clone());
                    false // Remove from pending (will be re-added after retry)
                } else {
                    // Max retries reached - mark as failed
                    for msg in app.messages.iter_mut().rev() {
                        if let Some(ref msg_id) = msg.message_id {
                            if msg_id == &pm.message_id {
                                // Keep as not sent (red)
                                app.status_message = format!("⚠️  Message failed after {} retries", pm.retry_count);
                                break;
                            }
                        }
                    }
                    false // Remove from pending
                }
            } else {
                true // Keep in pending
            }
        });
        
        // Retry messages
        for mut pm in messages_to_retry {
            pm.retry_count += 1;
            pm.sent_at = now;
            
            if tx.send(ClientMessage::SendMessage {
                room_id: pm.room_id.clone(),
                encrypted_payload: pm.encrypted_payload.clone(),
                message_id: pm.message_id.clone(),
            }).await.is_ok() {
                app.pending_messages.push(pm);
            }
        }

        // Check auto-close countdown
        if let Some(left_at) = app.user_left_at {
            let elapsed = left_at.elapsed().as_secs();
            if elapsed >= 5 {
                // Time's up - close chat and return to welcome
                if let Some(ref chat_code) = app.current_chat_code {
                    let room_id = chat_code_to_room_id(chat_code);
                    let _ = tx.send(ClientMessage::LeaveChat { room_id }).await;
                }
                app.mode = AppMode::Welcome;
                app.current_chat_code = None;
                app.chat_key = None;
                app.chain_key = None;
                app.messages.clear();
                app.user_left_at = None;
                app.closing_in_seconds = None;
                app.status_message = "Chat closed - other user left".to_string();
            } else {
                // Update countdown
                let remaining = 5 - elapsed;
                if app.closing_in_seconds != Some(remaining as u8) {
                    app.closing_in_seconds = Some(remaining as u8);
                    app.status_message = format!("⚠️  Other user left - Closing in {} seconds...", remaining);
                }
            }
        }

        // Gestisci eventi (tastiera e mouse)
        if event::poll(std::time::Duration::from_millis(100))? {
            let evt = event::read()?;
            
            // Gestisci eventi mouse (paste con tasto destro)
            if let Event::Mouse(mouse) = evt {
                if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
                    // Incolla dalla clipboard quando si preme il tasto destro
                    if app.mode == AppMode::JoinChat || app.mode == AppMode::Chat {
                        match get_clipboard_text() {
                            Ok(clipboard_text) => {
                                if app.mode == AppMode::JoinChat {
                                    app.input = clipboard_text.trim().to_string();
                                    app.status_message = "✅ Codice incollato con mouse destro".to_string();
                                } else {
                                    app.input.push_str(&clipboard_text);
                                    app.status_message = "✅ Testo incollato con mouse destro".to_string();
                                }
                            }
                            Err(e) => {
                                app.status_message = format!("⚠️  Errore mouse paste: {}", e);
                            }
                        }
                    }
                }
            }
            
            // Gestisci eventi tastiera
            if let Event::Key(key) = evt {
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
                            // Genera il codice chat localmente
                            let chat_code = if app.numeric_codes {
                                generate_numeric_chat_code()
                            } else {
                                generate_chat_code()
                            };
                            let room_id = chat_code_to_room_id(&chat_code);
                            
                            // Salva il chat_code per usarlo dopo
                            app.pending_chat_code = Some(chat_code);
                            
                            tx.send(ClientMessage::CreateChat {
                                room_id,
                                chat_type: ChatType::OneToOne,
                                username: app.username.clone(),
                            })
                            .await?;
                            app.mode = AppMode::WaitingForChatCode;
                        }
                        KeyCode::Char('2') => {
                            // Genera il codice chat localmente
                            let chat_code = if app.numeric_codes {
                                generate_numeric_chat_code()
                            } else {
                                generate_chat_code()
                            };
                            let room_id = chat_code_to_room_id(&chat_code);
                            
                            // Salva il chat_code per usarlo dopo
                            app.pending_chat_code = Some(chat_code);
                            
                            tx.send(ClientMessage::CreateChat {
                                room_id,
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
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Incolla dalla clipboard con CTRL+V
                            match get_clipboard_text() {
                                Ok(clipboard_text) => {
                                    app.input = clipboard_text.trim().to_string();
                                    app.status_message = "✅ Codice incollato con CTRL+V".to_string();
                                }
                                Err(e) => {
                                    app.status_message = format!("⚠️  Errore CTRL+V: {}", e);
                                }
                            }
                        }
                        // SHIFT+Insert per incollare (standard Linux)
                        KeyCode::Insert if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            match get_clipboard_text() {
                                Ok(clipboard_text) => {
                                    app.input = clipboard_text.trim().to_string();
                                    app.status_message = "✅ Codice incollato con SHIFT+Insert".to_string();
                                }
                                Err(e) => {
                                    app.status_message = format!("⚠️  Errore SHIFT+Insert: {}", e);
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            let chat_code = app.input.clone();
                            let room_id = chat_code_to_room_id(&chat_code);
                            // NON pulire app.input qui - lo useremo dopo per derivare la chiave
                            tx.send(ClientMessage::JoinChat {
                                room_id,
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
                        KeyCode::Up => {
                            app.scroll_up();
                        }
                        KeyCode::Down => {
                            app.scroll_down();
                        }
                        KeyCode::PageUp => {
                            // Scroll veloce su
                            for _ in 0..5 {
                                app.scroll_up();
                            }
                        }
                        KeyCode::PageDown => {
                            // Scroll veloce giù
                            for _ in 0..5 {
                                app.scroll_down();
                            }
                        }
                        KeyCode::Home => {
                            // Vai all'inizio
                            app.scroll_offset = 0;
                        }
                        KeyCode::End => {
                            // Vai alla fine
                            app.scroll_to_bottom();
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

                                // Encrypt and sign the message
                                if let Some(ref chat_code) = app.current_chat_code {
                                    if let Some(ref key) = app.chat_key {
                                        if let Some(ref mut chain_key) = app.chain_key {
                                            let room_id = chat_code_to_room_id(chat_code);
                                            
                                            // Get next chain key for forward secrecy
                                            let message_key = chain_key.next();
                                            let chain_index = chain_key.index() - 1; // index after next()
                                            
                                            // Create signature data
                                            let mut sig_data = Vec::new();
                                            sig_data.extend_from_slice(content.as_bytes());
                                            sig_data.extend_from_slice(&app.sequence_number.to_le_bytes());
                                            sig_data.extend_from_slice(&chain_index.to_le_bytes());
                                            
                                            // Sign the message
                                            let signature = app.identity_key.sign(&sig_data);
                                            let public_key = app.identity_key.public_key_bytes();
                                            
                                            // Generate unique message ID
                                            let message_id = format!("{}-{}-{}", 
                                                app.username, 
                                                app.sequence_number,
                                                std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_nanos()
                                            );
                                            
                                            let payload = MessagePayload::new(
                                                app.username.clone(),
                                                content.clone(),
                                                app.sequence_number,
                                                public_key,
                                                signature,
                                                chain_index,
                                            );
                                            
                                            // Add our own message to the UI immediately
                                            // Mark as not sent initially, will be confirmed when we get ACK
                                            app.messages.push(ChatMessage {
                                                username: app.username.clone(),
                                                content: content.clone(),
                                                timestamp: std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_secs() as i64,
                                                verified: true, // Our own messages are always verified
                                                sent: false,    // Will be set to true when we get ACK
                                                message_id: Some(message_id.clone()),
                                            });
                                            
                                            app.sequence_number += 1;
                                            
                                            // Try to send the message
                                            if let Ok(serialized) = bincode::serialize(&payload) {
                                                if let Ok(encrypted) = key.encrypt_with_chain(&serialized, &message_key) {
                                                    // Add to pending messages for retry logic
                                                    app.pending_messages.push(PendingMessage {
                                                        message_id: message_id.clone(),
                                                        room_id: room_id.clone(),
                                                        encrypted_payload: encrypted.clone(),
                                                        sent_at: std::time::Instant::now(),
                                                        retry_count: 0,
                                                    });
                                                    
                                                    if tx.send(ClientMessage::SendMessage {
                                                        room_id,
                                                        encrypted_payload: encrypted,
                                                        message_id,
                                                    })
                                                    .await.is_err() {
                                                        app.status_message = "⚠️  Failed to send message".to_string();
                                                    }
                                                    // Don't mark as sent here - wait for server echo to confirm
                                                } else {
                                                    app.status_message = "⚠️  Failed to encrypt message".to_string();
                                                }
                                            } else {
                                                app.status_message = "⚠️  Failed to serialize message".to_string();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            if let Some(ref chat_code) = app.current_chat_code {
                                let room_id = chat_code_to_room_id(chat_code);
                                tx.send(ClientMessage::LeaveChat {
                                    room_id,
                                })
                                .await?;
                            }
                            app.mode = AppMode::Welcome;
                            app.current_chat_code = None;
                            app.chat_key = None;
                            app.chain_key = None;
                            app.messages.clear();
                            app.user_left_at = None;
                            app.closing_in_seconds = None;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            } // Fine gestione eventi tastiera
        }

        // Gestisci messaggi dal server
        while let Ok(msg) = server_rx.try_recv() {
            match msg {
                ServerMessage::ChatCreated {
                    room_id: _,
                    chat_type: _,
                } => {
                    // Use locally generated chat_code
                    if let Some(chat_code) = app.pending_chat_code.take() {
                        // Copy code to clipboard
                        if let Err(e) = copy_to_clipboard(&chat_code) {
                            app.status_message = format!("Chat created! Code: {} (manual copy)", chat_code);
                            eprintln!("⚠️  Cannot copy to clipboard: {}", e);
                        } else {
                            app.status_message = format!("✅ Chat created! Code copied to clipboard: {}", &chat_code[..16.min(chat_code.len())]);
                        }
                        
                        app.current_chat_code = Some(chat_code.clone());
                        app.chat_key = ChatKey::derive_from_code(&chat_code).ok();
                        app.chain_key = ChainKey::from_chat_code(&chat_code).ok();
                        app.sequence_number = 0;
                        app.mode = AppMode::Chat;
                        app.scroll_to_bottom(); // Auto-scroll on enter
                    }
                }
                ServerMessage::JoinedChat {
                    room_id: _,
                    chat_type: _,
                    participant_count,
                } => {
                    // Use chat_code from user input
                    let chat_code = app.input.clone();
                    app.input.clear();
                    
                    app.current_chat_code = Some(chat_code.clone());
                    app.chat_key = ChatKey::derive_from_code(&chat_code).ok();
                    app.chain_key = ChainKey::from_chat_code(&chat_code).ok();
                    app.sequence_number = 0;
                    app.mode = AppMode::Chat;
                    app.scroll_to_bottom(); // Auto-scroll on enter
                    app.status_message = format!(
                        "Joined chat! Participants: {}",
                        participant_count
                    );
                }
                ServerMessage::Error { message } => {
                    app.status_message = format!("Error: {}", message);
                    app.mode = AppMode::Welcome;
                }
                ServerMessage::MessageAck { message_id } => {
                    // Remove from pending messages
                    app.pending_messages.retain(|pm| pm.message_id != message_id);
                    
                    // Mark message as sent in UI
                    for msg in app.messages.iter_mut().rev() {
                        if let Some(ref msg_id) = msg.message_id {
                            if msg_id == &message_id {
                                msg.sent = true;
                                break;
                            }
                        }
                    }
                }
                ServerMessage::MessageReceived {
                    encrypted_payload,
                    message_id,
                    ..
                } => {
                    if let Some(ref key) = app.chat_key {
                        if let Some(ref mut chain_key) = app.chain_key {
                            // Try decrypting with sender's chain key index
                            let mut decrypted_payload = None;
                            
                            // Try a range of indices around the current position
                            // This handles out-of-order messages and different sender/receiver positions
                            let current_index = chain_key.index();
                            let start_index = current_index.saturating_sub(5);
                            let end_index = current_index + 20; // Look ahead more for messages from others
                            
                            for test_index in start_index..=end_index {
                                let mut test_chain = chain_key.clone();
                                test_chain.advance_to(test_index);
                                let test_key = test_chain.next();
                                
                                if let Ok(decrypted) = key.decrypt_with_chain(&encrypted_payload, &test_key) {
                                    if let Ok(payload) = bincode::deserialize::<MessagePayload>(&decrypted) {
                                        // Verify the chain_key_index matches
                                        if payload.chain_key_index != test_index {
                                            continue; // Wrong index, keep trying
                                        }
                                        
                                        // Verify signature
                                        let mut sig_data = Vec::new();
                                        sig_data.extend_from_slice(payload.content.as_bytes());
                                        sig_data.extend_from_slice(&payload.sequence_number.to_le_bytes());
                                        sig_data.extend_from_slice(&payload.chain_key_index.to_le_bytes());
                                        
                                        let verified = IdentityKey::verify(
                                            &payload.sender_public_key,
                                            &sig_data,
                                            &payload.signature
                                        ).is_ok();
                                        
                                        decrypted_payload = Some((payload, verified, test_index));
                                        break;
                                    }
                                }
                            }
                            
                            if let Some((payload, verified, used_index)) = decrypted_payload {
                                // Advance chain key PAST the used index for next message
                                // This ensures we're ready for the next message in sequence
                                chain_key.advance_to(used_index + 1);
                                
                                // Check if this is our own message (already added locally)
                                if payload.username == app.username {
                                    // This is our own message echoed back from server
                                    // Find it in messages and confirm it was sent successfully
                                    // Look for the most recent unsent message from us
                                    for msg in app.messages.iter_mut().rev() {
                                        if msg.username == app.username && !msg.sent && msg.content == payload.content {
                                            msg.sent = true;
                                            break;
                                        }
                                    }
                                } else {
                                    // This is a message from another user
                                    app.messages.push(ChatMessage {
                                        username: payload.username.clone(),
                                        content: payload.content.clone(),
                                        timestamp: payload.timestamp,
                                        verified,
                                        sent: true, // Received messages are already sent
                                        message_id: Some(message_id),
                                    });
                                    
                                    if !verified {
                                        app.status_message = "⚠️ Warning: Unverified message signature!".to_string();
                                    }
                                    
                                    // Auto-scroll on new message
                                    app.scroll_to_bottom();
                                }
                            }
                        }
                    }
                }
                ServerMessage::UserJoined { username, .. } => {
                    app.status_message = format!("✅ {} joined the chat", username);
                    
                    // Add system message to chat
                    app.messages.push(ChatMessage {
                        username: "SYSTEM".to_string(),
                        content: format!("{} joined the chat", username),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        verified: true,
                        sent: true, // System messages are always sent
                        message_id: None,
                    });
                    app.scroll_to_bottom();
                }
                ServerMessage::UserLeft { username, .. } => {
                    // Add system message to chat
                    app.messages.push(ChatMessage {
                        username: "SYSTEM".to_string(),
                        content: format!("⚠️  {} left the chat. Chat will close in 5 seconds...", username),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        verified: true,
                        sent: true, // System messages are always sent
                        message_id: None,
                    });
                    app.scroll_to_bottom();
                    
                    // Start countdown for auto-close
                    app.user_left_at = Some(std::time::Instant::now());
                    app.closing_in_seconds = Some(5);
                    app.status_message = format!("⚠️  {} left the chat - Closing in 5 seconds...", username);
                }
            }
        }
    }
}
