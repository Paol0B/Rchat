use common::{ClientMessage, ServerMessage};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_rustls::rustls::pki_types::CertificateDer;
use tokio_rustls::TlsAcceptor;
use zeroize::Zeroize;
use clap::Parser;

mod chat;
use chat::ChatState;

const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB max

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Porta del server
    #[arg(short, long, default_value_t = 6666)]
    port: u16,

    /// Host del server
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🔒 Rchat Server v0.1.0");
    println!("🚀 Avvio server su {}:{}...", args.host, args.port);

    // Stato globale del server
    let state = Arc::new(ChatState::new(false)); // Il parametro non è più usato

    // Configura TLS
    let tls_acceptor = configure_tls()?;

    // Bind sulla porta
    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    println!("✅ Server in ascolto su {}:{}", args.host, args.port);
    println!("⚠️  ATTENZIONE: Tutti i dati sono volatili e NON persistiti su disco");
    println!();

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("📡 Nuova connessione da {}", addr);

        let state = Arc::clone(&state);
        let acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if let Err(e) = handle_client(tls_stream, state, addr.to_string()).await {
                        eprintln!("❌ Errore gestione client {}: {}", addr, e);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Errore TLS handshake con {}: {}", addr, e);
                }
            }
        });
    }
}

async fn handle_client(
    stream: tokio_rustls::server::TlsStream<TcpStream>,
    state: Arc<ChatState>,
    client_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);
    let mut current_chat: Option<String> = None;

    // Split dello stream per leggere e scrivere concorrentemente
    let (mut read_half, mut write_half) = tokio::io::split(stream);

    // Task per inviare messaggi al client
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

    // Loop per ricevere messaggi dal client
    loop {
        // Leggi lunghezza messaggio
        let mut len_buf = [0u8; 4];
        if read_half.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        if msg_len == 0 || msg_len > MAX_MESSAGE_SIZE {
            break;
        }

        // Leggi messaggio
        let mut msg_buf = vec![0u8; msg_len];
        if read_half.read_exact(&mut msg_buf).await.is_err() {
            break;
        }

        let msg: ClientMessage = match bincode::deserialize(&msg_buf) {
            Ok(m) => m,
            Err(_) => break,
        };

        // Zeroizza il buffer
        msg_buf.zeroize();

        // Gestisci il messaggio
        match msg {
            ClientMessage::CreateChat {
                room_id,
                chat_type,
                username,
            } => {
                // Il client ha generato il chat_code localmente e ci invia solo il room_id (hash)
                // Il server non conosce mai il chat_code originale
                state.create_chat(room_id.clone(), chat_type.clone()).await;
                let _ = state.join_chat(&room_id, username.clone(), tx.clone()).await;

                current_chat = Some(room_id.clone());

                let _ = tx
                    .send(ServerMessage::ChatCreated {
                        room_id,
                        chat_type,
                    })
                    .await;
            }

            ClientMessage::JoinChat {
                room_id,
                username,
            } => {
                match state.join_chat(&room_id, username.clone(), tx.clone()).await {
                    Ok((chat_type, count)) => {
                        current_chat = Some(room_id.clone());

                        let _ = tx
                            .send(ServerMessage::JoinedChat {
                                room_id: room_id.clone(),
                                chat_type,
                                participant_count: count,
                            })
                            .await;

                        // Notifica gli altri partecipanti
                        state
                            .broadcast_user_event(&room_id, username, true)
                            .await;
                    }
                    Err(e) => {
                        let _ = tx.send(ServerMessage::Error { message: e }).await;
                    }
                }
            }

            ClientMessage::SendMessage {
                room_id,
                encrypted_payload,
            } => {
                // Il server NON decripta, inoltra solo
                state
                    .broadcast_message(&room_id, encrypted_payload, &client_id)
                    .await;
            }

            ClientMessage::LeaveChat { room_id } => {
                if let Some(username) = state.leave_chat(&room_id, &client_id).await {
                    state
                        .broadcast_user_event(&room_id, username, false)
                        .await;
                }
                current_chat = None;
            }
        }
    }

    // Cleanup alla disconnessione
    if let Some(room_id) = current_chat {
        if let Some(username) = state.leave_chat(&room_id, &client_id).await {
            state.broadcast_user_event(&room_id, username, false).await;
        }
    }

    println!("👋 Client {} disconnesso", client_id);
    Ok(())
}

fn configure_tls() -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
    use rustls::ServerConfig;
    use rustls_pemfile::{certs, private_key};
    use std::fs::File;
    use std::io::BufReader;

    // Carica certificato e chiave (self-signed per demo)
    let cert_path = "server.crt";
    let key_path = "server.key";

    // Genera certificati se non esistono
    if !std::path::Path::new(cert_path).exists() {
        eprintln!("⚠️  Certificati TLS non trovati. Genera con:");
        eprintln!("   openssl req -x509 -newkey rsa:4096 -nodes -keyout server.key -out server.crt -days 365 -subj '/CN=localhost'");
        return Err("Certificati TLS mancanti".into());
    }

    let cert_file = File::open(cert_path)?;
    let key_file = File::open(key_path)?;

    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);

    let certs: Vec<CertificateDer> = certs(&mut cert_reader).collect::<Result<_, _>>()?;
    let key = private_key(&mut key_reader)?.ok_or("Nessuna chiave privata trovata")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
