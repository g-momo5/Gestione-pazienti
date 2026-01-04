# 🚀 Guida Rapida - Sistema Gestionale Pazienti TAVI

## Avvio Immediato

### Primo Utilizzo

```bash
# 1. Apri il terminale e vai nella cartella
cd /Users/mohamed/Desktop/database_TAVI

# 2. Avvia l'applicazione
npm run tauri:dev
```

**Tempo di avvio**: ~30 secondi la prima volta (compila Rust), poi ~5 secondi

### Cosa Succede?
1. Si apre automaticamente una finestra dell'applicazione
2. Il database SQLite viene creato automaticamente
3. L'app è pronta all'uso!

## 🎯 Come Usare l'App

### Barra Laterale (NavigationRail)

Ci sono 3 icone sulla sinistra:

- **📋 Elenco** - Visualizza tutte le procedure
- **➕ Nuova** - Inserisci una nuova procedura
- **📊 Statistiche** - Vedi grafici e analisi

### 1. Inserire una Nuova Procedura

1. Click su **➕ Nuova**
2. Compila i 4 sezioni del form:
   - **📋 Dati Paziente**: Nome, cognome, data nascita, altezza, peso
   - **🕐 Dati Temporali**: Data e ora procedura (la durata si calcola automaticamente!)
   - **❤️ Dati Pre-procedurali**: FE, Vmax, Gmax, Gmed, AVA, etc.
   - **🏥 Dati Procedurali**: Tipo valvola, modello, dimensione, dilatazioni
3. Click su **💾 Salva Procedura**
4. Torna automaticamente all'elenco

**💡 Tip**: I campi in rosso sono obbligatori. I campi con errori mostrano un messaggio in rosso sotto.

### 2. Cercare e Filtrare Procedure

1. Click su **📋 Elenco**
2. Usa la **barra di ricerca** per cercare per nome, cognome o modello valvola
3. Usa i **filtri** per:
   - Filtrare per tipo valvola (Balloon/Self Expandable)
   - Filtrare per periodo (ultimo mese, 3 mesi, 6 mesi, anno)
4. Click su **📥 Esporta Excel** per scaricare le procedure filtrate

### 3. Visualizzare Dettagli Procedura

1. Nell'elenco, **click su una card procedura**
2. Si apre un modal con **tutti i dettagli**
3. Da qui puoi:
   - **✏️ Modificare** la procedura
   - **🗑️ Eliminare** la procedura (con conferma)

### 4. Modificare una Procedura

1. Nell'elenco, click sull'icona **✏️** sulla card
2. Oppure: apri i dettagli e click su **✏️ Modifica**
3. Il form si apre pre-compilato
4. Modifica i campi desiderati
5. Click su **💾 Salva Procedura**

### 5. Vedere le Statistiche

1. Click su **📊 Statistiche**
2. Vedi:
   - **4 card** con totale procedure, durata media, percentuali dilatazione
   - **Grafico a torta**: distribuzione tipo valvola
   - **Grafico a barre**: top 5 modelli valvole più usati
   - **Parametri medi**: FE, Vmax, Gmax, Gmed, AVA
3. Puoi filtrare per **periodo** (ultimo mese, 3 mesi, etc.)
4. Click su **📥 Esporta Excel** per scaricare le statistiche

## 📥 Export Excel

### Export Procedure
- **Dove**: Elenco → Pulsante "📥 Esporta Excel"
- **Cosa esporta**: Tutte le procedure visibili (con filtri applicati)
- **Formato**: 26 colonne con header formattato, calcoli automatici inclusi

### Export Statistiche
- **Dove**: Statistiche → Pulsante "📥 Esporta Excel"
- **Cosa esporta**: 3 fogli Excel:
  1. Statistiche Generali
  2. Parametri Emodinamici Medi
  3. Top Modelli Valvole

## ⌨️ Scorciatoie Utili

- **Ricerca**: Scrivi direttamente nella barra di ricerca, filtra in tempo reale
- **Reset Filtri**: Click su "🔄 Reset Filtri" per rimuovere tutti i filtri
- **Chiudi Modal**: Click fuori dal modal o premi ESC
- **Annulla Form**: Click su "Annulla" per tornare all'elenco senza salvare

## 🔧 Funzionalità Automatiche

### Calcoli Automatici
- **Età**: Calcolata automaticamente dalla data di nascita
- **BMI**: Calcolato da altezza e peso (se inseriti)
- **Durata Procedura**: Calcolata da ora inizio e ora fine
- **Contatore Totale**: Aggiornato automaticamente nell'header

### Validazione Real-time
- I campi vengono validati mentre scrivi
- Messaggi di errore immediati se i valori non sono nel range corretto
- Il pulsante "Salva" si attiva solo se il form è valido

### Dropdown Intelligenti
- **Modelli Valvola**: Cambiano automaticamente in base al tipo selezionato
  - Se selezioni "Balloon Expandable" → vedi Edwards SAPIEN 3, Myval, Allegra
  - Se selezioni "Self Expandable" → vedi CoreValve Evolut, ACURATE neo, Portico

### Campi Condizionali
- **Valvola Protesica**: Se spunti la checkbox, appaiono i campi Modello e Dimensione

## 💾 Dove Sono i Dati?

Il database è salvato in:
```
macOS:   ~/Library/Application Support/com.hospital.gestionale-pazienti-tavi/registro_tavi.db
Windows: %APPDATA%\com.hospital.gestionale-pazienti-tavi\registro_tavi.db
Linux:   ~/.local/share/com.hospital.gestionale-pazienti-tavi/registro_tavi.db
```

### Fare Backup

**Metodo semplice**:
1. Chiudi l'applicazione
2. Copia il file `registro_tavi.db` in un posto sicuro (USB, cloud, etc.)
3. Per ripristinare: sostituisci il file con la copia

## 🐛 Risoluzione Problemi

### L'app non si avvia
```bash
# Riprova con:
npm install
npm run tauri:dev
```

### Errore "port 1420 in use"
```bash
# Chiudi altre istanze dell'app o:
pkill -f "vite"
npm run tauri:dev
```

### Build non funziona
```bash
# Pulisci e ricompila:
rm -rf node_modules dist src-tauri/target
npm install
npm run tauri:dev
```

### Database corrotto
Il database è nel file `.db` (vedi sopra). Se corrotto:
1. Chiudi l'app
2. Rinomina il file `registro_tavi.db` in `registro_tavi_old.db`
3. Riavvia l'app → crea un nuovo database vuoto

## 📝 Best Practices

### Inserimento Dati
1. **Inserisci sempre i dati obbligatori** (nome, cognome, data nascita, data procedura, orari, tipo/modello valvola)
2. **Compila più campi possibili** per statistiche più accurate
3. **Controlla gli orari** prima di salvare (ora fine > ora inizio)

### Ricerca
1. **Usa filtri + ricerca insieme** per risultati precisi
2. **Export frequenti** delle procedure per backup

### Statistiche
1. **Filtra per periodo** per analisi temporali
2. **Export statistiche** per report periodici

## 🎯 Casi d'Uso Comuni

### Scenario 1: Inserire Procedura Post-operatoria
1. Vai su ➕ Nuova
2. Compila nome paziente e dati anagrafici
3. Inserisci data/ora della procedura appena effettuata
4. Compila parametri pre-procedurali (dal referto ecocardiografico)
5. Inserisci tipo e modello valvola impiantata
6. Spunta pre/post dilatazione se effettuate
7. Salva

### Scenario 2: Report Mensile
1. Vai su 📊 Statistiche
2. Seleziona "Ultimo mese" dal filtro periodo
3. Visualizza le metriche
4. Export Excel per il report
5. Vai su 📋 Elenco
6. Filtra per "Ultimo mese"
7. Export Excel delle procedure

### Scenario 3: Cercare Paziente Specifico
1. Vai su 📋 Elenco
2. Digita cognome nella ricerca
3. Click sulla card per vedere dettagli completi
4. Modifica se necessario con ✏️

## 🎨 Personalizzazione

### Aggiungere Modelli Valvole

Modifica il file `src/lib/constants.js`:

```javascript
export const BALLOON_EXPANDABLE_MODELS = [
  'Edwards SAPIEN 3',
  'Edwards SAPIEN 3 Ultra',
  'Myval',
  'Allegra',
  'TUO_NUOVO_MODELLO',  // ← Aggiungi qui
];
```

Poi riavvia l'app:
```bash
# Ctrl+C per fermare
npm run tauri:dev
```

### Cambiare Colori

Modifica `tailwind.config.js`:

```javascript
colors: {
  primary: {
    DEFAULT: '#2196F3',  // ← Cambia questo colore
    // ...
  }
}
```

## 📞 Supporto

Per problemi o domande:
1. Controlla questa guida
2. Leggi [README.md](README.md) per info tecniche
3. Leggi [COMPLETAMENTO.md](COMPLETAMENTO.md) per dettagli implementazione

---

**Buon lavoro! 🏥❤️**
