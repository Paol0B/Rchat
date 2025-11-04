"""
Network module for RChat Qt Client
Gestisce la connessione TLS e la comunicazione con il server
"""

import ssl
import socket
import struct
import threading
from typing import Callable, Optional
from PyQt6.QtCore import QObject, pyqtSignal


class NetworkClient(QObject):
    """Client di rete per la comunicazione con il server RChat"""
    
    # Segnali Qt per comunicazione thread-safe
    message_received = pyqtSignal(bytes)
    connection_established = pyqtSignal()
    connection_lost = pyqtSignal(str)
    error_occurred = pyqtSignal(str)
    
    def __init__(self, host: str, port: int, insecure: bool = False):
        super().__init__()
        self.host = host
        self.port = port
        self.insecure = insecure
        self.socket: Optional[ssl.SSLSocket] = None
        self.running = False
        self.receive_thread: Optional[threading.Thread] = None
        
    def connect(self) -> bool:
        """Connette al server con TLS"""
        try:
            # Crea contesto SSL
            if self.insecure:
                # Modalità insicura per certificati self-signed
                context = ssl.create_default_context()
                context.check_hostname = False
                context.verify_mode = ssl.CERT_NONE
            else:
                # Modalità sicura - verifica certificato
                context = ssl.create_default_context()
                try:
                    # Prova percorsi multipli per il certificato
                    cert_paths = [
                        'server.crt',
                        '../server.crt',
                        '../../server.crt',
                        '/home/paol0b/sources/Rchat/server.crt'
                    ]
                    cert_loaded = False
                    for cert_path in cert_paths:
                        try:
                            context.load_verify_locations(cert_path)
                            cert_loaded = True
                            break
                        except FileNotFoundError:
                            continue
                    
                    if not cert_loaded:
                        self.error_occurred.emit(
                            "Certificato server non trovato (server.crt).\n"
                            "Usa modalità insecure per testing."
                        )
                        return False
                except Exception as e:
                    self.error_occurred.emit(f"Errore caricamento certificato: {str(e)}")
                    return False
            
            # Crea socket TCP
            raw_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            raw_socket.settimeout(5.0)  # Timeout più breve per connessione rapida
            
            # Wrap con TLS
            self.socket = context.wrap_socket(
                raw_socket,
                server_hostname=self.host
            )
            
            # Connetti
            self.socket.connect((self.host, self.port))
            self.socket.settimeout(None)  # Rimuovi timeout dopo connessione
            self.running = True
            
            # Avvia thread di ricezione
            self.receive_thread = threading.Thread(
                target=self._receive_loop,
                daemon=True
            )
            self.receive_thread.start()
            
            self.connection_established.emit()
            return True
            
        except socket.timeout:
            self.error_occurred.emit(
                f"Timeout connessione a {self.host}:{self.port}\n"
                "Il server non risponde."
            )
            return False
        except ConnectionRefusedError:
            self.error_occurred.emit(
                f"Connessione rifiutata da {self.host}:{self.port}\n"
                "Il server non è in esecuzione o la porta è errata."
            )
            return False
        except socket.gaierror:
            self.error_occurred.emit(
                f"Host non trovato: {self.host}\n"
                "Verifica l'indirizzo inserito."
            )
            return False
        except Exception as e:
            self.error_occurred.emit(f"Errore connessione: {str(e)}")
            return False
    
    def disconnect(self):
        """Disconnette dal server"""
        self.running = False
        if self.socket:
            try:
                self.socket.close()
            except:
                pass
            self.socket = None
    
    def send_message(self, data: bytes) -> bool:
        """Invia un messaggio al server (formato: lunghezza u32 BE + dati)"""
        if not self.socket or not self.running:
            return False
        
        try:
            # Prepara header con lunghezza del messaggio (u32 big-endian)
            length = len(data)
            header = struct.pack('>I', length)
            
            # Invia header + dati
            self.socket.sendall(header + data)
            return True
            
        except Exception as e:
            self.error_occurred.emit(f"Errore invio: {str(e)}")
            self.running = False
            return False
    
    def _receive_loop(self):
        """Loop di ricezione messaggi (esegue in thread separato)"""
        while self.running and self.socket:
            try:
                # Leggi header (u32 big-endian con lunghezza)
                header = self._recv_exact(4)
                if not header:
                    break
                
                length = struct.unpack('>I', header)[0]
                
                # Validazione lunghezza
                if length == 0 or length > 1024 * 1024:  # Max 1MB
                    self.error_occurred.emit(
                        f"Lunghezza messaggio invalida: {length}"
                    )
                    break
                
                # Leggi dati
                data = self._recv_exact(length)
                if not data:
                    break
                
                # Emetti segnale con i dati ricevuti
                self.message_received.emit(data)
                
            except Exception as e:
                if self.running:
                    self.connection_lost.emit(f"Connessione persa: {str(e)}")
                break
        
        self.running = False
        if self.socket:
            try:
                self.socket.close()
            except:
                pass
    
    def _recv_exact(self, n: int) -> Optional[bytes]:
        """Riceve esattamente n byte dal socket"""
        data = b''
        while len(data) < n:
            try:
                chunk = self.socket.recv(n - len(data))
                if not chunk:
                    return None
                data += chunk
            except Exception:
                return None
        return data
