# 🔧 Troubleshooting - Sistema Gestionale Pazienti TAVI

## ❌ Errore: "Errore durante il salvataggio" quando si salva una procedura

### Causa Probabile
Il problema è nella serializzazione dei dati tra JavaScript (frontend) e Rust (backend).

### Soluzione 1: Controlla la Console del Browser

1. **Apri l'applicazione**: `npm run tauri:dev`
2. **Apri DevTools**:
   - **macOS**: `Cmd + Option + I`
   - **Windows/Linux**: `F12` o `Ctrl + Shift + I`
3. **Vai alla tab "Console"**
4. **Prova a salvare** una procedura
5. **Guarda l'errore** completo che appare in console

### Soluzione 2: Compila Solo i Campi Obbligatori

Per testare, prova a compilare **SOLO** questi campi obbligatori:

- Nome: `Mario`
- Cognome: `Rossi`
- Data Nascita: `1950-01-01`
- Data Procedura: (oggi)
- Ora Inizio: `09:00`
- Ora Fine: `10:30`
- Tipo Valvola: `Balloon Expandable`
- Modello Valvola: `Edwards SAPIEN 3`

**Lascia vuoti tutti gli altri campi** (altezza, peso, FE, Vmax, etc.)

### Soluzione 3: Controlla i Log di Tauri

Nel terminale dove hai eseguito `npm run tauri:dev`, cerca messaggi di errore dopo aver cliccato "Salva".

### Possibili Errori e Soluzioni

#### Errore: "failed to deserialize"
**Causa**: I dati JavaScript non matchano il formato Rust
**Soluzione**: Assicurati di:
- Non lasciare campi stringa vuoti per valvola protesica se non spuntata
- I campi numerici vuoti devono essere `null` o `undefined`, non stringhe vuote

#### Errore: "CHECK constraint failed"
**Causa**: Il tipo valvola non è uno dei due valori accettati
**Soluzione**: Seleziona esattamente:
- `Balloon Expandable` OPPURE
- `Self Expandable`

#### Errore: "NOT NULL constraint failed"
**Causa**: Un campo obbligatorio è vuoto
**Soluzione**: Compila tutti i campi obbligatori (quelli con asterisco rosso `*`)

### Debug Avanzato

Se l'errore persiste, esegui questo test:

1. **Apri la console del browser** (DevTools → Console)

2. **Esegui questo comando** per testare direttamente il backend:

```javascript
// Test minimo
const testProcedure = {
  nome: "Mario",
  cognome: "Rossi",
  data_nascita: "1950-01-01",
  altezza: undefined,
  peso: undefined,
  fe: undefined,
  vmax: undefined,
  gmax: undefined,
  gmed: undefined,
  ava: undefined,
  anulus_aortico: undefined,
  valvola_protesica: false,
  protesica_modello: null,
  protesica_dimensione: null,
  data_procedura: "2024-11-30",
  ora_inizio: "09:00",
  ora_fine: "10:30",
  tipo_valvola: "Balloon Expandable",
  modello_valvola: "Edwards SAPIEN 3",
  dimensione_valvola: undefined,
  pre_dilatazione: false,
  post_dilatazione: false
};

// Chiama il backend
const { invoke } = window.__TAURI__.tauri;
invoke('create_procedure', { procedure: testProcedure })
  .then(id => console.log('✅ Successo! ID:', id))
  .catch(err => console.error('❌ Errore:', err));
```

3. **Guarda il risultato** in console

### Soluzione 4: Verifica Database

Il database potrebbe non essere stato creato correttamente.

1. **Trova il database**:
   - macOS: `~/Library/Application Support/com.hospital.gestionale-pazienti-tavi/`
   - Windows: `%APPDATA%\com.hospital.gestionale-pazienti-tavi\`
   - Linux: `~/.local/share/com.hospital.gestionale-pazienti-tavi/`

2. **Verifica che esista** il file `registro_tavi.db`

3. **Se non esiste**, l'app lo creerà al primo avvio

4. **Se esiste ma corrotto**, eliminalo e riavvia l'app

### Soluzione 5: Ricompila Tutto

```bash
# Ferma l'app (Ctrl+C nel terminale)

# Pulisci cache
rm -rf node_modules dist src-tauri/target

# Reinstalla dipendenze
npm install

# Riavvia
npm run tauri:dev
```

### Soluzione 6: Controlla Permessi File

**macOS/Linux**:
```bash
# Verifica permessi cartella Application Support
ls -la ~/Library/Application\ Support/com.hospital.gestionale-pazienti-tavi/

# Se non hai permessi, crea la cartella manualmente
mkdir -p ~/Library/Application\ Support/com.hospital.gestionale-pazienti-tavi
chmod 755 ~/Library/Application\ Support/com.hospital.gestionale-pazienti-tavi
```

**Windows**:
Verifica che l'app abbia permessi di scrittura in `%APPDATA%`

## 🐛 Altri Problemi Comuni

### App non si avvia

**Errore: "Port 1420 already in use"**
```bash
# Chiudi processi esistenti
pkill -f "tauri dev"
pkill -f "vite"

# Riavvia
npm run tauri:dev
```

### Grafici non appaiono

**Causa**: ECharts non caricato
**Soluzione**:
1. Vai su Statistiche
2. Apri DevTools → Console
3. Cerca errori JavaScript
4. Se vedi "echarts is not defined", reinstalla:
   ```bash
   npm install echarts
   npm run tauri:dev
   ```

### Export Excel non funziona

**Errore**: Dialog salvataggio non appare
**Soluzione**:
1. Controlla permessi file system
2. Verifica che `tauri.conf.json` abbia:
   ```json
   "dialog": {
     "save": true
   },
   "fs": {
     "all": true
   }
   ```

### Filtri non funzionano

**Sintomo**: Ricerca o filtri non cambiano i risultati
**Soluzione**:
1. Clicca "🔄 Reset Filtri"
2. Ricarica la lista (cambia tab e torna)
3. Se persiste, riavvia l'app

## 📞 Debug Checklist

Prima di segnalare un problema, verifica:

- [ ] Ho provato con SOLO campi obbligatori compilati?
- [ ] Ho controllato la Console del browser (DevTools)?
- [ ] Ho controllato il terminale dove gira `tauri dev`?
- [ ] Ho provato a riavviare l'app?
- [ ] Ho pulito e reinstallato dipendenze?
- [ ] Il database esiste nella cartella corretta?
- [ ] Ho permessi di scrittura sulla cartella dati?

## 🔍 Log Utili

### Vedere log dettagliati Tauri

Avvia con variabile d'ambiente:
```bash
RUST_LOG=debug npm run tauri:dev
```

### Vedere query SQL

Nel terminale, cerca righe che iniziano con:
- `[SQLite]` - Query database
- `[Tauri]` - Operazioni Tauri
- `[Error]` - Errori

### JSON Inviato al Backend

Nella console del browser, il `console.error` nel form mostra l'errore completo.

## 🆘 Se Nulla Funziona

1. **Chiudi completamente l'app**
2. **Elimina il database**:
   ```bash
   # macOS
   rm -rf ~/Library/Application\ Support/com.hospital.gestionale-pazienti-tavi/
   ```
3. **Pulisci tutto**:
   ```bash
   rm -rf node_modules dist src-tauri/target
   npm install
   ```
4. **Riavvia**:
   ```bash
   npm run tauri:dev
   ```
5. **Riprova il salvataggio**

Se il problema persiste dopo tutti questi step, potrebbe essere un bug nel codice che richiede ulteriore debug specifico.

---

**Ultima modifica**: 30 Novembre 2024
