# Gestionale Pazienti TAVI

Sistema desktop per la registrazione e gestione delle procedure TAVI (Transcatheter Aortic Valve Implantation).

## Stack Tecnologico

- **Tauri 1.5+** - Framework desktop (Rust + WebView)
- **Svelte 4** - Frontend framework
- **Tailwind CSS 3** - Styling
- **Apache ECharts 5** - Grafici e statistiche
- **SQLite** - Database locale
- **SheetJS (xlsx)** - Export Excel

## Prerequisiti

- Node.js 18+ e npm
- Rust 1.70+ e Cargo
- Xcode Command Line Tools (macOS) / Visual Studio (Windows) / GCC (Linux)

## Installazione

```bash
# 1. Clona il repository
cd database_TAVI

# 2. Installa dipendenze npm
npm install

# 3. Installa dipendenze Rust (automatico al primo build)
```

## Sviluppo

```bash
# Run in development mode con hot reload
npm run tauri:dev
```

## Build

```bash
# Build per produzione
npm run tauri:build
```

Il file .dmg (macOS) / .msi (Windows) / .AppImage (Linux) sarà generato in `src-tauri/target/release/bundle/`.

## Funzionalità

- ✅ Registrazione completa procedure TAVI
- ✅ Database SQLite locale
- ✅ Ricerca e filtri multi-campo
- ✅ Statistiche aggregate con grafici interattivi
- ✅ Export Excel procedure e statistiche
- ✅ Validazione dati medici
- ✅ Calcoli automatici (età, BMI, durata)
- ✅ UI moderna e minimal (Material Design 3 inspired)

## Struttura Progetto

```
database_TAVI/
├── src/                  # Frontend Svelte
│   ├── lib/
│   │   ├── components/   # Componenti riutilizzabili
│   │   ├── stores/       # State management
│   │   ├── utils/        # Utilities
│   │   └── views/        # Schermate principali
│   ├── App.svelte        # Layout principale
│   └── main.js           # Entry point
│
├── src-tauri/            # Backend Rust
│   └── src/
│       ├── main.rs       # Entry point Tauri
│       ├── database.rs   # Layer database SQLite
│       ├── models.rs     # Modelli dati
│       └── commands.rs   # Tauri commands (API)
│
└── public/               # Assets statici
```

## Licenza

Proprietario - © 2024 Dr. Mohamed. Tutti i diritti riservati.
Uso riservato al personale ospedaliero autorizzato.
