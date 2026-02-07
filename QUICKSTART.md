# Quick Start Guide 🚀

## Erste Schritte

### 1. Setup (nur einmal)

```bash
# Virtual Environment wurde bereits erstellt
# Falls nicht vorhanden:
python3 -m venv venv
./venv/bin/pip install -r requirements.txt
```

### 2. XLS-Dateien konvertieren

**Einfachster Weg - Batch-Konvertierung:**

```bash
# Schritt 1: Kopiere deine CRDB Bank XLS-Dateien in den Ordner
cp /pfad/zu/deinen/statements/*.xls to_convert/

# Schritt 2: Führe die Konvertierung aus
./convert_all.sh

# Fertig! Die CSV-Dateien sind jetzt in: converted/
```

**Oder einzelne Datei konvertieren:**

```bash
./convert.sh meine_datei.xls ausgabe.csv
```

## Häufige Fragen

### Werden Dateien mehrfach konvertiert?

Nein! Das System trackt bereits konvertierte Dateien in `.conversion_log.json`. Wenn du `./convert_all.sh` erneut ausführst, werden nur neue oder geänderte Dateien konvertiert.

### Wie konvertiere ich Dateien erneut?

```bash
# Lösche das Log
rm .conversion_log.json

# Führe Konvertierung erneut aus
./convert_all.sh
```

### Wo finde ich die konvertierten Dateien?

Im Ordner `converted/` - jede Datei behält ihren ursprünglichen Namen, nur mit `.csv` Endung.

### Kann ich die Dateien direkt in ZOHO Books importieren?

Ja! Die generierten CSV-Dateien sind vollständig kompatibel mit ZOHO Books:
- Semikolon (;) als Trennzeichen
- Korrekte Spaltenüberschriften
- Beschreibungen auf 99 Zeichen begrenzt
- Unix-Zeilenendungen (LF)

## Tipps

1. **Mehrere Dateien auf einmal:** Kopiere einfach alle XLS-Dateien in `to_convert/` und führe `./convert_all.sh` aus

2. **Workflow:**
   - Neue Statements von CRDB Bank herunterladen
   - In `to_convert/` ablegen
   - `./convert_all.sh` ausführen
   - CSV-Dateien aus `converted/` in ZOHO Books importieren

3. **Archivierung:** Du kannst die XLS-Dateien nach dem Import aus `to_convert/` entfernen oder archivieren. Das Log bleibt erhalten.

## Probleme?

```bash
# Log ansehen
cat .conversion_log.json

# Verzeichnisse überprüfen
ls -l to_convert/ converted/

# Manuelle Einzelkonvertierung testen
./convert.sh example/202601_Statement_TZS.xls test_output.csv
```
