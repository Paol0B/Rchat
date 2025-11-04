"""
Main Window per RChat Qt Client
"""

import sys
import time
from datetime import datetime
from typing import Optional
from PyQt6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QLineEdit, QTextEdit, QScrollArea, QFrame,
    QStackedWidget, QMessageBox, QDialog, QComboBox, QCheckBox,
    QApplication
)
from PyQt6.QtCore import Qt, QTimer, pyqtSlot, QPropertyAnimation, QEasingCurve
from PyQt6.QtGui import QFont, QClipboard, QTextCursor

from ..network import NetworkClient
from ..controller import ChatController
from .styles import DARK_THEME, LIGHT_THEME


class ConnectionDialog(QDialog):
    """Dialog per connessione al server"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Connetti a RChat")
        self.setModal(True)
        self.setFixedSize(450, 400)
        
        layout = QVBoxLayout()
        layout.setSpacing(16)
        layout.setContentsMargins(24, 24, 24, 24)
        
        # Titolo
        title = QLabel("🔒 RChat - Connessione Sicura")
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        title_font = QFont()
        title_font.setPointSize(16)
        title_font.setBold(True)
        title.setFont(title_font)
        layout.addWidget(title)
        
        # Username
        layout.addWidget(QLabel("Nome utente:"))
        self.username_input = QLineEdit()
        self.username_input.setPlaceholderText("Inserisci il tuo username")
        layout.addWidget(self.username_input)
        
        # Server
        layout.addWidget(QLabel("Server:"))
        self.host_input = QLineEdit("127.0.0.1")
        layout.addWidget(self.host_input)
        
        # Porta
        layout.addWidget(QLabel("Porta:"))
        self.port_input = QLineEdit("6666")
        layout.addWidget(self.port_input)
        
        # Opzioni
        self.insecure_check = QCheckBox("Accetta certificati self-signed (INSECURE)")
        layout.addWidget(self.insecure_check)
        
        self.numeric_check = QCheckBox("Usa codici numerici a 6 cifre")
        layout.addWidget(self.numeric_check)
        
        # Pulsanti
        btn_layout = QHBoxLayout()
        self.connect_btn = QPushButton("Connetti")
        self.connect_btn.clicked.connect(self.accept)
        self.cancel_btn = QPushButton("Annulla")
        self.cancel_btn.setObjectName("SecondaryButton")
        self.cancel_btn.clicked.connect(self.reject)
        btn_layout.addWidget(self.cancel_btn)
        btn_layout.addWidget(self.connect_btn)
        layout.addLayout(btn_layout)
        
        layout.addStretch()
        self.setLayout(layout)
    
    def get_connection_info(self):
        """Ritorna le informazioni di connessione"""
        return {
            'username': self.username_input.text().strip(),
            'host': self.host_input.text().strip(),
            'port': int(self.port_input.text().strip()),
            'insecure': self.insecure_check.isChecked(),
            'numeric_codes': self.numeric_check.isChecked()
        }


class WelcomeScreen(QWidget):
    """Schermata di benvenuto"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QVBoxLayout()
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(32)
        
        # ASCII Art
        ascii_art = QLabel(
            "╦═╗┌─┐┬ ┬┌─┐┌┬┐\n"
            "╠╦╝│  ├─┤├─┤ │ \n"
            "╩╚═└─┘┴ ┴┴ ┴ ┴ \n\n"
            "🔒 End-to-End Encrypted Chat"
        )
        ascii_art.setAlignment(Qt.AlignmentFlag.AlignCenter)
        font = QFont("Courier New", 14)
        font.setBold(True)
        ascii_art.setFont(font)
        layout.addWidget(ascii_art)
        
        # Menu
        menu_layout = QVBoxLayout()
        menu_layout.setSpacing(16)
        
        self.create_btn = QPushButton("📝 Crea Nuova Chat")
        self.create_btn.setMinimumHeight(50)
        menu_layout.addWidget(self.create_btn)
        
        self.join_btn = QPushButton("🔗 Unisciti a una Chat")
        self.join_btn.setMinimumHeight(50)
        menu_layout.addWidget(self.join_btn)
        
        layout.addLayout(menu_layout)
        
        # Footer
        footer = QLabel("⚠️  Tutti i messaggi sono volatili e non persistenti")
        footer.setAlignment(Qt.AlignmentFlag.AlignCenter)
        footer.setObjectName("WarningLabel")
        layout.addWidget(footer)
        
        self.setLayout(layout)


class CreateChatScreen(QWidget):
    """Schermata creazione chat"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QVBoxLayout()
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(24)
        
        title = QLabel("Crea Nuova Chat")
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        font = QFont()
        font.setPointSize(18)
        font.setBold(True)
        title.setFont(font)
        layout.addWidget(title)
        
        self.one_to_one_btn = QPushButton("👤 Chat 1:1 (max 2 partecipanti)")
        self.one_to_one_btn.setMinimumHeight(60)
        layout.addWidget(self.one_to_one_btn)
        
        self.group_btn = QPushButton("👥 Chat di Gruppo (max 8 partecipanti)")
        self.group_btn.setMinimumHeight(60)
        layout.addWidget(self.group_btn)
        
        self.back_btn = QPushButton("⬅️ Indietro")
        self.back_btn.setObjectName("SecondaryButton")
        self.back_btn.setMinimumHeight(40)
        layout.addWidget(self.back_btn)
        
        info = QLabel("Un codice sicuro verrà generato e copiato negli appunti")
        info.setAlignment(Qt.AlignmentFlag.AlignCenter)
        info.setObjectName("StatusLabel")
        layout.addWidget(info)
        
        self.setLayout(layout)


class JoinChatScreen(QWidget):
    """Schermata per unirsi a una chat"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QVBoxLayout()
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(24)
        
        title = QLabel("Unisciti a una Chat")
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        font = QFont()
        font.setPointSize(18)
        font.setBold(True)
        title.setFont(font)
        layout.addWidget(title)
        
        info = QLabel("Inserisci il codice della chat:")
        info.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(info)
        
        self.code_input = QLineEdit()
        self.code_input.setPlaceholderText("Incolla il codice chat qui...")
        self.code_input.setMinimumHeight(50)
        layout.addWidget(self.code_input)
        
        btn_layout = QHBoxLayout()
        self.back_btn = QPushButton("⬅️ Indietro")
        self.back_btn.setObjectName("SecondaryButton")
        self.join_btn = QPushButton("✅ Unisciti")
        btn_layout.addWidget(self.back_btn)
        btn_layout.addWidget(self.join_btn)
        layout.addLayout(btn_layout)
        
        tip = QLabel("💡 Puoi incollare con CTRL+V o tasto destro")
        tip.setAlignment(Qt.AlignmentFlag.AlignCenter)
        tip.setObjectName("StatusLabel")
        layout.addWidget(tip)
        
        self.setLayout(layout)


class MessageBubble(QFrame):
    """Bolla messaggio stile chat moderna"""
    
    def __init__(self, message_data: dict, is_own: bool, parent=None):
        super().__init__(parent)
        self.setObjectName("MessageFrame")
        
        layout = QVBoxLayout()
        layout.setContentsMargins(12, 8, 12, 8)
        layout.setSpacing(4)
        
        # Header con username e timestamp
        header_layout = QHBoxLayout()
        
        username_label = QLabel(message_data['username'])
        username_font = QFont()
        username_font.setBold(True)
        username_label.setFont(username_font)
        
        # Color coding per username
        if is_own:
            username_label.setStyleSheet("color: #11111b;")
        else:
            username_label.setStyleSheet(f"color: {self._get_username_color(message_data['username'])};")
        
        time_label = QLabel(self._format_timestamp(message_data['timestamp']))
        time_label.setStyleSheet("color: #6c7086; font-size: 9pt;")
        
        header_layout.addWidget(username_label)
        header_layout.addStretch()
        header_layout.addWidget(time_label)
        layout.addLayout(header_layout)
        
        # Contenuto messaggio
        content = QLabel(message_data['content'])
        content.setWordWrap(True)
        content.setTextInteractionFlags(
            Qt.TextInteractionFlag.TextSelectableByMouse |
            Qt.TextInteractionFlag.TextSelectableByKeyboard
        )
        if is_own:
            content.setStyleSheet("color: #11111b;")
        layout.addWidget(content)
        
        # Status indicator
        status_label = QLabel()
        status_label.setStyleSheet("font-size: 9pt;")
        if not message_data.get('sent', True):
            status_label.setText("✗ Non inviato")
            status_label.setStyleSheet("color: #f38ba8; font-size: 9pt;")
        elif message_data.get('verified', True):
            status_label.setText("✓")
            status_label.setStyleSheet("color: #a6e3a1; font-size: 9pt;")
        else:
            status_label.setText("⚠ Non verificato")
            status_label.setStyleSheet("color: #f9e2af; font-size: 9pt;")
        
        layout.addWidget(status_label, alignment=Qt.AlignmentFlag.AlignRight)
        
        self.setLayout(layout)
        
        # Styling per messaggi propri vs altri
        if is_own:
            self.setObjectName("MyMessageFrame")
            self.setStyleSheet("""
                #MyMessageFrame {
                    background-color: #89b4fa;
                    border-radius: 12px;
                    margin: 4px 4px 4px 60px;
                }
            """)
        else:
            self.setStyleSheet("""
                #MessageFrame {
                    background-color: #313244;
                    border-radius: 12px;
                    margin: 4px 60px 4px 4px;
                }
            """)
    
    def _format_timestamp(self, timestamp: int) -> str:
        """Formatta timestamp Unix"""
        dt = datetime.fromtimestamp(timestamp)
        return dt.strftime("%H:%M")
    
    def _get_username_color(self, username: str) -> str:
        """Genera colore consistente per username"""
        colors = [
            "#94e2d5", "#89dceb", "#74c7ec", "#89b4fa",
            "#b4befe", "#cba6f7", "#f5c2e7", "#eba0ac",
            "#f38ba8", "#fab387", "#f9e2af", "#a6e3a1"
        ]
        hash_val = sum(ord(c) for c in username)
        return colors[hash_val % len(colors)]


class ChatScreen(QWidget):
    """Schermata chat principale"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.own_username = ""
        self.setup_ui()
    
    def setup_ui(self):
        layout = QVBoxLayout()
        layout.setSpacing(8)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Header
        header = QFrame()
        header.setObjectName("TitleBar")
        header.setFixedHeight(60)
        header_layout = QHBoxLayout()
        
        self.chat_info_label = QLabel("🔒 Chat Crittografata")
        self.chat_info_label.setObjectName("TitleLabel")
        header_layout.addWidget(self.chat_info_label)
        
        header_layout.addStretch()
        
        self.leave_btn = QPushButton("🚪 Esci")
        self.leave_btn.setObjectName("DangerButton")
        self.leave_btn.setFixedHeight(40)
        header_layout.addWidget(self.leave_btn)
        
        header.setLayout(header_layout)
        layout.addWidget(header)
        
        # Area messaggi con scroll
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setObjectName("ChatArea")
        
        self.messages_container = QWidget()
        self.messages_layout = QVBoxLayout()
        self.messages_layout.setSpacing(8)
        self.messages_layout.addStretch()
        self.messages_container.setLayout(self.messages_layout)
        
        scroll.setWidget(self.messages_container)
        layout.addWidget(scroll, stretch=1)
        
        # Area input
        input_frame = QFrame()
        input_layout = QHBoxLayout()
        input_layout.setSpacing(12)
        
        self.message_input = QLineEdit()
        self.message_input.setPlaceholderText("Scrivi un messaggio...")
        self.message_input.setMinimumHeight(50)
        self.message_input.returnPressed.connect(self._on_send_clicked)
        input_layout.addWidget(self.message_input)
        
        self.send_btn = QPushButton("📤 Invia")
        self.send_btn.setMinimumSize(100, 50)
        self.send_btn.clicked.connect(self._on_send_clicked)
        input_layout.addWidget(self.send_btn)
        
        input_frame.setLayout(input_layout)
        layout.addWidget(input_frame)
        
        # Status bar
        self.status_label = QLabel()
        self.status_label.setObjectName("StatusLabel")
        self.status_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.status_label.setFixedHeight(30)
        layout.addWidget(self.status_label)
        
        self.setLayout(layout)
    
    def add_message(self, message_data: dict):
        """Aggiunge un messaggio alla chat"""
        is_own = message_data['username'] == self.own_username
        bubble = MessageBubble(message_data, is_own)
        
        # Inserisci prima dello stretch
        self.messages_layout.insertWidget(
            self.messages_layout.count() - 1,
            bubble
        )
        
        # Auto-scroll al nuovo messaggio
        QTimer.singleShot(50, self._scroll_to_bottom)
    
    def _scroll_to_bottom(self):
        """Scrolla alla fine dei messaggi"""
        scroll_area = self.parent().findChild(QScrollArea)
        if scroll_area:
            scroll_area.verticalScrollBar().setValue(
                scroll_area.verticalScrollBar().maximum()
            )
    
    def _on_send_clicked(self):
        """Handler per invio messaggio"""
        # Implementato nel MainWindow
        pass
    
    def set_status(self, message: str, is_error: bool = False):
        """Imposta messaggio di stato"""
        self.status_label.setText(message)
        if is_error:
            self.status_label.setObjectName("ErrorLabel")
        else:
            self.status_label.setObjectName("StatusLabel")
        self.status_label.style().unpolish(self.status_label)
        self.status_label.style().polish(self.status_label)


class MainWindow(QMainWindow):
    """Finestra principale dell'applicazione"""
    
    def __init__(self):
        super().__init__()
        self.setWindowTitle("RChat - End-to-End Encrypted Chat")
        self.setMinimumSize(800, 600)
        
        # State
        self.network: Optional[NetworkClient] = None
        self.controller: Optional[ChatController] = None
        self.username = ""
        self.numeric_codes = False
        
        # Applica tema
        self.setStyleSheet(DARK_THEME)
        
        # Setup UI
        self.setup_ui()
        
        # Mostra dialog di connessione
        self.show_connection_dialog()
    
    def setup_ui(self):
        """Setup dell'interfaccia utente"""
        # Widget centrale con stacked widget per navigazione
        central = QWidget()
        layout = QVBoxLayout()
        layout.setContentsMargins(0, 0, 0, 0)
        
        self.stacked_widget = QStackedWidget()
        
        # Crea schermate
        self.welcome_screen = WelcomeScreen()
        self.welcome_screen.create_btn.clicked.connect(self.show_create_chat)
        self.welcome_screen.join_btn.clicked.connect(self.show_join_chat)
        
        self.create_screen = CreateChatScreen()
        self.create_screen.one_to_one_btn.clicked.connect(lambda: self.create_chat("OneToOne"))
        self.create_screen.group_btn.clicked.connect(lambda: self.create_chat("Group"))
        self.create_screen.back_btn.clicked.connect(self.show_welcome)
        
        self.join_screen = JoinChatScreen()
        self.join_screen.join_btn.clicked.connect(self.join_chat)
        self.join_screen.back_btn.clicked.connect(self.show_welcome)
        
        self.chat_screen = ChatScreen()
        self.chat_screen.leave_btn.clicked.connect(self.leave_chat)
        self.chat_screen.send_btn.clicked.connect(self.send_message)
        
        # Aggiungi schermate allo stack
        self.stacked_widget.addWidget(self.welcome_screen)
        self.stacked_widget.addWidget(self.create_screen)
        self.stacked_widget.addWidget(self.join_screen)
        self.stacked_widget.addWidget(self.chat_screen)
        
        layout.addWidget(self.stacked_widget)
        central.setLayout(layout)
        self.setCentralWidget(central)
    
    def show_connection_dialog(self):
        """Mostra dialog di connessione"""
        dialog = ConnectionDialog(self)
        if dialog.exec() == QDialog.DialogCode.Accepted:
            info = dialog.get_connection_info()
            if not info['username']:
                QMessageBox.warning(self, "Errore", "Username richiesto!")
                QApplication.quit()
                return
            
            self.username = info['username']
            self.numeric_codes = info['numeric_codes']
            
            # Crea controller
            self.controller = ChatController(self.username, self.numeric_codes)
            self.setup_controller_signals()
            
            # Connetti al server
            self.connect_to_server(info['host'], info['port'], info['insecure'])
        else:
            QApplication.quit()
    
    def connect_to_server(self, host: str, port: int, insecure: bool):
        """Connette al server"""
        self.network = NetworkClient(host, port, insecure)
        self.network.message_received.connect(self.on_message_received)
        self.network.connection_established.connect(self.on_connected)
        self.network.connection_lost.connect(self.on_connection_lost)
        self.network.error_occurred.connect(self.on_error)
        
        # Mostra messaggio di connessione
        self.statusBar().showMessage("⏳ Connessione al server in corso...")
        
        if not self.network.connect():
            QMessageBox.critical(
                self, 
                "Errore Connessione", 
                f"Impossibile connettersi a {host}:{port}\n\n"
                "Verifica che:\n"
                "• Il server sia in esecuzione\n"
                "• Host e porta siano corretti\n"
                "• Il firewall permetta la connessione"
            )
            QApplication.quit()
    
    @pyqtSlot()
    def on_connected(self):
        """Callback connessione stabilita"""
        self.statusBar().showMessage(f"✅ Connesso come {self.username}")
    
    @pyqtSlot(str)
    def on_connection_lost(self, reason: str):
        """Callback connessione persa"""
        QMessageBox.warning(self, "Connessione Persa", reason)
        self.show_welcome()
    
    @pyqtSlot(str)
    def on_error(self, error: str):
        """Callback errore"""
        self.statusBar().showMessage(f"❌ {error}", 5000)
    
    @pyqtSlot(bytes)
    def on_message_received(self, data: bytes):
        """Callback messaggio ricevuto"""
        if self.controller:
            self.controller.handle_server_message(data)
    
    def setup_controller_signals(self):
        """Collega i segnali del controller all'UI"""
        self.controller.chat_created.connect(self.on_chat_created)
        self.controller.chat_joined.connect(self.on_chat_joined)
        self.controller.message_received.connect(self.on_message_received_decrypted)
        self.controller.user_joined.connect(self.on_user_joined)
        self.controller.user_left.connect(self.on_user_left)
        self.controller.error_occurred.connect(self.on_controller_error)
        self.controller.message_ack.connect(self.on_message_ack)
    
    @pyqtSlot(str)
    def on_chat_created(self, chat_code: str):
        """Chat creata con successo"""
        # Copia codice negli appunti
        clipboard = QApplication.clipboard()
        clipboard.setText(chat_code)
        
        # Mostra codice (solo prime 16 char se lungo)
        display_code = chat_code if len(chat_code) <= 16 else chat_code[:16] + "..."
        self.chat_screen.set_status(f"✅ Chat creata! Codice: {display_code} (copiato)")
        self.chat_screen.chat_info_label.setText(f"🔒 Chat: {display_code}")
        self.show_chat()
    
    @pyqtSlot(int)
    def on_chat_joined(self, participant_count: int):
        """Unito a chat con successo"""
        self.chat_screen.set_status(f"✅ Unito alla chat! Partecipanti: {participant_count}")
        self.show_chat()
    
    @pyqtSlot(dict)
    def on_message_received_decrypted(self, message_data: dict):
        """Messaggio ricevuto e decriptato"""
        if not message_data.get('is_own', False):
            # Messaggio da altro utente
            self.chat_screen.add_message(message_data)
            if not message_data.get('verified', True):
                self.chat_screen.set_status("⚠️ Warning: Unverified message signature!", True)
        else:
            # Nostro messaggio echoed dal server - aggiorna status
            pass
    
    @pyqtSlot(str)
    def on_user_joined(self, username: str):
        """Utente unito alla chat"""
        system_msg = {
            'username': 'SYSTEM',
            'content': f"{username} si è unito alla chat",
            'timestamp': int(time.time()),
            'verified': True,
            'sent': True
        }
        self.chat_screen.add_message(system_msg)
        self.chat_screen.set_status(f"✅ {username} si è unito")
    
    @pyqtSlot(str)
    def on_user_left(self, username: str):
        """Utente uscito dalla chat"""
        system_msg = {
            'username': 'SYSTEM',
            'content': f"⚠️ {username} ha lasciato la chat",
            'timestamp': int(time.time()),
            'verified': True,
            'sent': True
        }
        self.chat_screen.add_message(system_msg)
        self.chat_screen.set_status(f"⚠️ {username} ha lasciato la chat")
    
    @pyqtSlot(str)
    def on_controller_error(self, error: str):
        """Errore dal controller"""
        self.statusBar().showMessage(f"❌ {error}", 5000)
        self.chat_screen.set_status(error, True)
    
    @pyqtSlot(str)
    def on_message_ack(self, message_id: str):
        """ACK messaggio ricevuto"""
        # Il messaggio è stato confermato dal server
        pass
    
    def show_welcome(self):
        """Mostra schermata benvenuto"""
        self.stacked_widget.setCurrentWidget(self.welcome_screen)
    
    def show_create_chat(self):
        """Mostra schermata creazione chat"""
        self.stacked_widget.setCurrentWidget(self.create_screen)
    
    def show_join_chat(self):
        """Mostra schermata unisciti a chat"""
        self.stacked_widget.setCurrentWidget(self.join_screen)
    
    def show_chat(self):
        """Mostra schermata chat"""
        self.chat_screen.own_username = self.username
        self.stacked_widget.setCurrentWidget(self.chat_screen)
    
    def create_chat(self, chat_type: str):
        """Crea una nuova chat"""
        if not self.controller:
            QMessageBox.warning(self, "Errore", "Controller non inizializzato!")
            return
        
        if not self.network or not self.network.running:
            QMessageBox.warning(
                self, 
                "Errore Connessione", 
                "Non sei connesso al server!\n\nRiavvia l'applicazione."
            )
            return
        
        try:
            # Genera codice chat
            chat_code = self.controller.generate_chat_code()
            
            # Prepara messaggio per server
            max_participants = 8 if chat_type == "Group" else 2
            message_bytes = self.controller.create_chat(
                chat_code,
                chat_type,
                max_participants
            )
            
            # Invia al server
            if self.network.send_message(message_bytes):
                self.statusBar().showMessage("⏳ Creazione chat in corso...")
            else:
                QMessageBox.warning(
                    self, 
                    "Errore Invio", 
                    "Impossibile inviare richiesta al server.\n"
                    "La connessione potrebbe essere stata persa."
                )
        
        except Exception as e:
            QMessageBox.critical(self, "Errore", f"Errore creazione chat: {str(e)}")
    
    def join_chat(self):
        """Unisciti a una chat esistente"""
        if not self.controller:
            QMessageBox.warning(self, "Errore", "Controller non inizializzato!")
            return
        
        if not self.network or not self.network.running:
            QMessageBox.warning(
                self, 
                "Errore Connessione", 
                "Non sei connesso al server!\n\nRiavvia l'applicazione."
            )
            return
        
        chat_code = self.join_screen.code_input.text().strip()
        if not chat_code:
            QMessageBox.warning(self, "Errore", "Inserisci un codice chat valido!")
            return
        
        try:
            # Prepara messaggio per server
            message_bytes = self.controller.join_chat(chat_code)
            
            # Invia al server
            if self.network.send_message(message_bytes):
                self.statusBar().showMessage("⏳ Connessione alla chat...")
            else:
                QMessageBox.warning(
                    self, 
                    "Errore Invio", 
                    "Impossibile inviare richiesta al server.\n"
                    "La connessione potrebbe essere stata persa."
                )
        
        except Exception as e:
            QMessageBox.critical(self, "Errore", f"Errore join chat: {str(e)}")
    
    def send_message(self):
        """Invia messaggio"""
        if not self.controller:
            self.statusBar().showMessage("❌ Controller non inizializzato", 3000)
            return
        
        if not self.network or not self.network.running:
            self.statusBar().showMessage("❌ Non connesso al server", 3000)
            return
        
        content = self.chat_screen.message_input.text().strip()
        if not content:
            return
        
        try:
            # Cripta e firma messaggio
            message_bytes = self.controller.send_message(content)
            
            if message_bytes:
                # Invia al server
                if not self.network.send_message(message_bytes):
                    self.statusBar().showMessage("❌ Errore invio messaggio", 3000)
                    return
                
                # Aggiungi messaggio alla UI (non ancora confermato)
                message_data = {
                    'username': self.username,
                    'content': content,
                    'timestamp': int(time.time()),
                    'verified': True,
                    'sent': False  # Diventerà True quando riceveremo l'echo
                }
                self.chat_screen.add_message(message_data)
                
                # Invia al server
                if self.network.send_message(message_bytes):
                    self.chat_screen.message_input.clear()
                else:
                    self.chat_screen.set_status("⚠️ Errore invio messaggio", True)
            else:
                self.chat_screen.set_status("⚠️ Impossibile crittografare il messaggio", True)
        
        except Exception as e:
            self.chat_screen.set_status(f"❌ Errore: {str(e)}", True)
    
    def leave_chat(self):
        """Esci dalla chat"""
        if not self.controller or not self.network:
            self.show_welcome()
            return
        
        try:
            # Prepara messaggio leave
            message_bytes = self.controller.leave_chat()
            
            # Invia al server
            self.network.send_message(message_bytes)
            
            # Torna alla welcome
            self.show_welcome()
            self.statusBar().showMessage("✅ Uscito dalla chat")
        
        except Exception as e:
            self.show_welcome()
            self.statusBar().showMessage(f"⚠️ Errore durante l'uscita: {str(e)}")

