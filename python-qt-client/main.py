#!/usr/bin/env python3
"""
RChat Qt Client - End-to-End Encrypted Chat
Entry point dell'applicazione
"""

import sys
from PyQt6.QtWidgets import QApplication
from rchat.ui.main_window import MainWindow


def main():
    """Main entry point"""
    app = QApplication(sys.argv)
    app.setApplicationName("RChat")
    app.setOrganizationName("RChat")
    
    # Crea e mostra finestra principale
    window = MainWindow()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
