# CRDB Bank to ZOHO Books CSV Converter

Dieses Skript konvertiert CRDB Bank Kontoauszüge im XLS-Format in CSV-Dateien, die in ZOHO Books importiert werden können.

## Features

- Konvertiert CRDB Bank XLS-Dateien in ZOHO Books CSV-Format
- Bereinigt Leerzeichen in Beschreibungen
- Begrenzt Beschreibungen auf maximal 99 Zeichen
- Konvertiert Datumsformat zu YYYY-MM-DD
- Entfernt Tausendertrennzeichen aus Beträgen

## Installation

1. Python 3 installieren (falls nicht vorhanden)
2. Virtuelles Environment erstellen und Abhängigkeiten installieren:

```bash
python3 -m venv venv
./venv/bin/pip install xlrd openpyxl
```

## Verwendung

### Methode 1: Batch-Konvertierung (Empfohlen) 🔥

Die einfachste Methode für mehrere Dateien:

```bash
# 1. Lege deine XLS-Dateien in den Ordner 'to_convert/'
# 2. Führe die Batch-Konvertierung aus
./convert_all.sh
```

**Features:**
- ✅ Konvertiert automatisch alle XLS-Dateien in `to_convert/`
- ✅ Speichert Ergebnisse in `converted/`
- ✅ Log-System verhindert Doppelkonvertierung
- ✅ Erkennt geänderte Dateien automatisch (via Hash)

Das Skript erstellt eine `.conversion_log.json` Datei, die trackt, welche Dateien bereits konvertiert wurden. Wenn du das Skript erneut ausführst, werden nur neue oder geänderte Dateien konvertiert.

### Methode 2: Einzeldatei-Konvertierung

Für einzelne Dateien kannst du das Einzelkonvertierungs-Skript verwenden:

```bash
./convert.sh <xls_datei> [ausgabe_csv]
```

**Beispiele:**

```bash
# Konvertierung mit automatischem Ausgabenamen
./convert.sh statement.xls
# Erstellt: statement_zoho.csv

# Konvertierung mit benutzerdefiniertem Ausgabenamen
./convert.sh statement.xls meine_ausgabe.csv
```

### Methode 3: Direkter Aufruf mit Python

```bash
./venv/bin/python3 crdb_to_zoho.py <xls_datei> [ausgabe_csv]
```

## CSV-Format

Die generierte CSV-Datei hat folgende Spalten:

- **Date**: Datum im Format YYYY-MM-DD
- **Withdrawals**: Abbuchungen (Debit)
- **Deposits**: Einzahlungen (Credit)
- **Payee**: Empfänger (leer)
- **Description**: Beschreibung (immer "Transfer")
- **Reference Number**: Transaktionsbeschreibung (max. 99 Zeichen)

Die CSV-Datei verwendet Semikolon (;) als Trennzeichen.

## Struktur der Eingabedatei

Das Skript erwartet CRDB Bank XLS-Dateien mit folgender Struktur:

- Zeilen 0-13: Header/Metadaten
- Zeile 14: Spaltenüberschriften
- Ab Zeile 15: Transaktionsdaten

## Ordnerstruktur

```
crdb_csv_conv/
├── to_convert/          # Lege hier deine XLS-Dateien ab
├── converted/           # Hier werden die CSV-Dateien gespeichert
├── example/             # Beispieldateien
├── venv/                # Python Virtual Environment
├── .conversion_log.json # Log der konvertierten Dateien (automatisch erstellt)
├── crdb_to_zoho.py      # Hauptkonvertierungs-Skript
├── batch_convert.py     # Batch-Konvertierungs-Skript
├── convert.sh           # Wrapper für Einzeldatei-Konvertierung
└── convert_all.sh       # Wrapper für Batch-Konvertierung
```

## Log-System

Das Batch-Konvertierungssystem verwendet eine `.conversion_log.json` Datei, um zu tracken:
- Welche Dateien bereits konvertiert wurden
- Wann die Konvertierung stattfand
- Hash der Originaldatei (um Änderungen zu erkennen)

Wenn eine Datei im `to_convert/` Ordner geändert wird, erkennt das System dies automatisch (via SHA256-Hash) und konvertiert die Datei erneut.

### Log zurücksetzen

Wenn du alle Dateien erneut konvertieren möchtest:

```bash
rm .conversion_log.json
./convert_all.sh
```

## Beispiele

### Batch-Konvertierung

```bash
# XLS-Dateien in den Ordner kopieren
cp /pfad/zu/statements/*.xls to_convert/

# Batch-Konvertierung ausführen
./convert_all.sh

# Ergebnisse ansehen
ls -l converted/
```

### Einzeldatei-Konvertierung

```bash
./convert.sh example/202601_Statement_TZS.xls example/output.csv
```

Dies konvertiert die Beispiel-XLS-Datei in eine ZOHO Books kompatible CSV-Datei.
