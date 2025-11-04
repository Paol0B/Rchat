"""
Stili e temi per l'UI di RChat
"""

DARK_THEME = """
QMainWindow {
    background-color: #1e1e2e;
}

QWidget {
    background-color: #1e1e2e;
    color: #cdd6f4;
    font-family: 'Segoe UI', 'Ubuntu', sans-serif;
    font-size: 11pt;
}

/* Barra del titolo personalizzata */
#TitleBar {
    background-color: #11111b;
    border-bottom: 2px solid #89b4fa;
    padding: 8px;
}

#TitleLabel {
    color: #89b4fa;
    font-size: 14pt;
    font-weight: bold;
}

/* Pulsanti */
QPushButton {
    background-color: #89b4fa;
    color: #11111b;
    border: none;
    border-radius: 6px;
    padding: 10px 20px;
    font-weight: bold;
    font-size: 11pt;
}

QPushButton:hover {
    background-color: #74c7ec;
}

QPushButton:pressed {
    background-color: #94e2d5;
}

QPushButton:disabled {
    background-color: #45475a;
    color: #6c7086;
}

QPushButton#DangerButton {
    background-color: #f38ba8;
}

QPushButton#DangerButton:hover {
    background-color: #eba0ac;
}

QPushButton#SecondaryButton {
    background-color: #45475a;
    color: #cdd6f4;
}

QPushButton#SecondaryButton:hover {
    background-color: #585b70;
}

/* Campi di input */
QLineEdit, QTextEdit {
    background-color: #313244;
    border: 2px solid #45475a;
    border-radius: 8px;
    padding: 12px;
    color: #cdd6f4;
    selection-background-color: #89b4fa;
}

QLineEdit:focus, QTextEdit:focus {
    border: 2px solid #89b4fa;
}

QLineEdit::placeholder, QTextEdit::placeholder {
    color: #6c7086;
}

/* Area messaggi */
#ChatArea {
    background-color: #181825;
    border: 1px solid #313244;
    border-radius: 12px;
}

/* Scrollbar */
QScrollBar:vertical {
    background-color: #181825;
    width: 12px;
    border-radius: 6px;
}

QScrollBar::handle:vertical {
    background-color: #45475a;
    border-radius: 6px;
    min-height: 20px;
}

QScrollBar::handle:vertical:hover {
    background-color: #585b70;
}

QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
    height: 0px;
}

/* Label */
QLabel {
    color: #cdd6f4;
}

QLabel#StatusLabel {
    color: #a6e3a1;
    font-size: 10pt;
}

QLabel#ErrorLabel {
    color: #f38ba8;
    font-size: 10pt;
}

QLabel#WarningLabel {
    color: #f9e2af;
    font-size: 10pt;
}

/* Gruppi e frame */
QGroupBox {
    border: 2px solid #45475a;
    border-radius: 8px;
    margin-top: 12px;
    padding-top: 8px;
    font-weight: bold;
}

QGroupBox::title {
    subcontrol-origin: margin;
    subcontrol-position: top left;
    padding: 0 8px;
    color: #89b4fa;
}

QFrame#MessageFrame {
    background-color: #313244;
    border-radius: 12px;
    padding: 12px;
    margin: 4px;
}

QFrame#MyMessageFrame {
    background-color: #89b4fa;
}

QFrame#SystemMessageFrame {
    background-color: #45475a;
}

/* ComboBox */
QComboBox {
    background-color: #313244;
    border: 2px solid #45475a;
    border-radius: 8px;
    padding: 8px;
    color: #cdd6f4;
}

QComboBox:hover {
    border: 2px solid #89b4fa;
}

QComboBox::drop-down {
    border: none;
}

QComboBox::down-arrow {
    image: none;
    border-left: 5px solid transparent;
    border-right: 5px solid transparent;
    border-top: 5px solid #cdd6f4;
    margin-right: 8px;
}

QComboBox QAbstractItemView {
    background-color: #313244;
    border: 2px solid #45475a;
    selection-background-color: #89b4fa;
    color: #cdd6f4;
}

/* CheckBox */
QCheckBox {
    spacing: 8px;
}

QCheckBox::indicator {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: 2px solid #45475a;
    background-color: #313244;
}

QCheckBox::indicator:checked {
    background-color: #89b4fa;
    border: 2px solid #89b4fa;
}

QCheckBox::indicator:hover {
    border: 2px solid #89b4fa;
}

/* Menu e Dialog */
QDialog {
    background-color: #1e1e2e;
}

QMessageBox {
    background-color: #1e1e2e;
}

QMessageBox QLabel {
    color: #cdd6f4;
}

/* Tooltip */
QToolTip {
    background-color: #313244;
    color: #cdd6f4;
    border: 1px solid #45475a;
    border-radius: 4px;
    padding: 4px;
}
"""

LIGHT_THEME = """
QMainWindow {
    background-color: #eff1f5;
}

QWidget {
    background-color: #eff1f5;
    color: #4c4f69;
    font-family: 'Segoe UI', 'Ubuntu', sans-serif;
    font-size: 11pt;
}

#TitleBar {
    background-color: #e6e9ef;
    border-bottom: 2px solid #1e66f5;
    padding: 8px;
}

#TitleLabel {
    color: #1e66f5;
    font-size: 14pt;
    font-weight: bold;
}

QPushButton {
    background-color: #1e66f5;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 10px 20px;
    font-weight: bold;
}

QPushButton:hover {
    background-color: #04a5e5;
}

QPushButton:pressed {
    background-color: #209fb5;
}

QPushButton:disabled {
    background-color: #9ca0b0;
    color: #bcc0cc;
}

QPushButton#DangerButton {
    background-color: #d20f39;
}

QPushButton#DangerButton:hover {
    background-color: #e64553;
}

QPushButton#SecondaryButton {
    background-color: #9ca0b0;
    color: #4c4f69;
}

QPushButton#SecondaryButton:hover {
    background-color: #8c8fa1;
}

QLineEdit, QTextEdit {
    background-color: #e6e9ef;
    border: 2px solid #9ca0b0;
    border-radius: 8px;
    padding: 12px;
    color: #4c4f69;
}

QLineEdit:focus, QTextEdit:focus {
    border: 2px solid #1e66f5;
}

#ChatArea {
    background-color: #dce0e8;
    border: 1px solid #9ca0b0;
    border-radius: 12px;
}

QScrollBar:vertical {
    background-color: #dce0e8;
    width: 12px;
    border-radius: 6px;
}

QScrollBar::handle:vertical {
    background-color: #9ca0b0;
    border-radius: 6px;
}

QScrollBar::handle:vertical:hover {
    background-color: #8c8fa1;
}

QLabel#StatusLabel {
    color: #40a02b;
}

QLabel#ErrorLabel {
    color: #d20f39;
}

QLabel#WarningLabel {
    color: #df8e1d;
}

QFrame#MessageFrame {
    background-color: #e6e9ef;
    border-radius: 12px;
    padding: 12px;
    margin: 4px;
}

QFrame#MyMessageFrame {
    background-color: #1e66f5;
}
"""
