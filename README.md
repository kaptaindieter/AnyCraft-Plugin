# AnyCraft-Plugin — Craft Suite Plugin for AnyCraft (Craft Anything)

Eigenstaendiges Craft-Suite-Plugin fuer Vanilla Craft Anything (AnyCraft): Integrierter BYOK AI-Agent mit App-Scaffolding- & Analyse-Commands, dynamischer Cloud- & Auto-Save-Sync (frei waehlbarer Ordner) und Aurora/Dark/Light Farbschema-System.

> **Wichtiger Hinweis zur Architektur:**
> Dieses Repository enthaelt **ausschliesslich das eigenstaendige Plugin** (die neuen Rust-Module, AI-Icons, den Hook-Patch und den Installer) und **nicht** den gesamten Vanilla-Quellcode von AnyCraft (Craft Anything). Jeder Nutzer laedt sich einfach die originale Vanilla-Version von [https://github.com/storytold/craft-anything](https://github.com/storytold/craft-anything) herunter und installiert anschliessend dieses Plugin darueber.

---

## Features des Plugins

### 1. BYOK AI-Agent & Command-Bridge (`artcraft_agent.rs`)
- **Bring Your Own Key (BYOK)**: Jeder Nutzer kann in der AI-Sidebar ueber **`+ API`** bzw. **`Manage APIs`** seine eigenen API-Keys und Endpunkte hinterlegen (OpenAI / GPT, Google Gemini, DeepSeek, Mistral, Qwen, lokales Ollama oder jeder OpenAI-kompatible Server).
- **Keine hartcodierten API-Keys**: Alle Zugangsdaten werden ausschliesslich lokal im jeweiligen Benutzerprofil (`~/.craft-suite/shared-ai-config.json` bzw. `%APPDATA%\CraftSuite\shared-ai-config.json`) gespeichert.
- **Direkte App-Steuerung ueber Commands**:
  - Unterstuetzte Commands: `anycraft.scaffoldApp`, `anycraft.inspectProject`, `anycraft.configureBuilder`, `save_project`

### 2. Dynamisches Cloud- & Auto-Save-Sync (`artcraft_drive.rs`)
- **Frei waehlbarer Sync-Ordner fuer jeden Endnutzer**: Ueber einen Klick auf das Cloud-Status-Badge in der oberen rechten Aktionsleiste oeffnet sich das Konfigurationsfenster **`☁️ Cloud & Auto-Save Sync`**.
- **Anbieter & Ordner selbst waehlen**: Unterstuetzt Google Drive, OneDrive, Nextcloud, Dropbox oder jeden beliebigen lokalen Ordner ueber den nativen Ordner-Auswahldialog (**`📂 Choose Folder`**).
- **Lokale Persistenz**: Die gewaehlte Pfad-Konfiguration wird lokal unter `~/.craft-suite/cloud-config.json` gespeichert (keine fremden Nutzerpfade im Code).

### 3. Farbschema-System (`Aurora`, `Dark`, `Light`) & UI-Design (`artcraft_theme.rs`)
- **Neues Farbschema-System**: Erweitert AnyCraft (Craft Anything) um den kosmischen **Aurora**-Modus (inkl. Regenbogen-Akzentlinie) zusaetzlich zu **Dark** und **Light**.
- **Einheitliche Aktionsleiste & Theme-Switcher**: Standardisierte 28px Top-Right Action Bar (`✓ Auto-Save`, `● Cloud-Status`, `✨ AI Agent`) sowie Theme-Umschalter in der Fusszeile.

---

## Kompatibilitaet mit der Vanilla-Version

- **Originales Upstream-Repository**: [https://github.com/storytold/craft-anything](https://github.com/storytold/craft-anything)
- **Getesteter Vanilla-Basis-Commit**: `801f060368c3b92540e2956e871fb6967f76c5f5` (*"Refuse pictures over 20 MB when they are added (#720)"*)

---

## Installation (Schritt fuer Schritt)

### Voraussetzungen
- **Git** und **Python 3.8+** (oder PowerShell unter Windows)
- **Rust Toolchain** (`cargo`, Rust 1.80+)

### 1. Originale Vanilla-Version herunterladen
```bash
git clone https://github.com/storytold/craft-anything.git
cd craft-anything
# Optional: Exakten getesteten Vanilla-Stand auschecken
git checkout 801f060368c3b92540e2956e871fb6967f76c5f5
cd ..
```

### 2. Dieses Plugin herunterladen & installieren
```bash
git clone https://github.com/kaptaindieter/AnyCraft-Plugin.git
cd AnyCraft-Plugin

# Plugin auf die Vanilla-Version anwenden (und optional direkt kompilieren):
python install_plugin.py ../craft-anything --build
```

Oder unter Windows per PowerShell:
```powershell
.\install_plugin.ps1 -TargetDir ..\craft-anything -Build
```

Die fertige ausfuehrbare Datei liegt anschliessend unter:
`../craft-anything/target/release/craft-anything.exe` (bzw. `target/release/craft-anything` unter Linux/macOS).

---

## Deinstallation (Vanilla-Zustand wiederherstellen)

Falls du das Vanilla-Repository wieder in den 100% originalen Zustand zurueckversetzen moechtest:

```bash
python uninstall_plugin.py ../craft-anything
```

Oder per PowerShell:
```powershell
.\uninstall_plugin.ps1 -TargetDir ..\craft-anything
```

---

## Paketstruktur des Plugins

- `plugin/src/artcraft_agent.rs`
- `plugin/src/artcraft_mcp.rs`
- `plugin/src/artcraft_drive.rs`
- `plugin/src/artcraft_theme.rs`
- `plugin/src/craft_family_scaffold.rs`
- `plugin/assets/ai_icons/` — Icons fuer die KI-Modell-Anbieter (OpenAI, Gemini, DeepSeek, Llama, Mistral, Qwen, AI)
- `patches/vanilla-hooks.patch` — Minimaler Integrations-Patch fuer die Vanilla-Dateien:
- `apps/craft-anything/build.rs`
- `crates/ui-egui/Cargo.toml`
- `crates/ui-egui/src/builders.rs`
- `crates/ui-egui/src/lib.rs`
- `crates/ui-egui/src/make.rs`
- `crates/ui-egui/src/settings_ui.rs`
- `crates/ui-egui/src/theme.rs`
- `crates/ui-egui/src/titlebar.rs`
- `crates/ui-egui/src/workbench.rs`
- `config/cloud-config.example.json` — Beispiel-Vorlage fuer die Endnutzer-Cloud-Konfiguration
- `config/shared-ai-config.example.json` — Beispiel-Vorlage fuer die eigene BYOK-API-Konfiguration
- `config/UPSTREAM_VERSION.json` — Versions- & Commit-Metadaten der Vanilla-Basis
- `install_plugin.py` / `install_plugin.ps1` — Automatischer Plugin-Installer
- `uninstall_plugin.py` / `uninstall_plugin.ps1` — Automatischer Plugin-Uninstaller

---

## Lizenz

MIT License — siehe [LICENSE](LICENSE).
