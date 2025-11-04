#!/usr/bin/env python3
"""
Test di connessione al server RChat
"""

import sys
import time
from PyQt6.QtWidgets import QApplication
from PyQt6.QtCore import QTimer
from rchat.network import NetworkClient

def test_connection():
    """Testa la connessione al server"""
    app = QApplication(sys.argv)
    
    print("=== Test Connessione RChat ===")
    print("Tentativo di connessione a localhost:6666...")
    
    # Crea client
    client = NetworkClient("localhost", 6666, insecure=True)
    
    # Callback connessione
    connection_ok = [False]
    
    def on_connected():
        print("✅ CONNESSO!")
        connection_ok[0] = True
        QTimer.singleShot(500, app.quit)
    
    def on_error(error):
        print(f"❌ ERRORE: {error}")
        QTimer.singleShot(500, app.quit)
    
    def on_connection_lost(reason):
        print(f"⚠️  CONNESSIONE PERSA: {reason}")
        QTimer.singleShot(500, app.quit)
    
    # Connetti segnali
    client.connection_established.connect(on_connected)
    client.error_occurred.connect(on_error)
    client.connection_lost.connect(on_connection_lost)
    
    # Connetti
    if not client.connect():
        print("❌ Connessione fallita immediatamente")
        return False
    
    # Timeout 5 secondi
    QTimer.singleShot(5000, app.quit)
    
    # Event loop
    app.exec()
    
    # Disconnetti
    client.disconnect()
    
    return connection_ok[0]

if __name__ == '__main__':
    success = test_connection()
    sys.exit(0 if success else 1)
