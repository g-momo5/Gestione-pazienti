# Piano di rifattorizzazione: ridurre ripetizioni nel codice

## Obiettivo
Ridurre duplicazioni di markup, logica e costanti, mantenendo invariato il comportamento. Il piano è incrementale e conforme a `REGOLE.md` (riuso componenti, stili centralizzati, no hardcode locale).

---

## 1) Componentizzare il pulsante “Torna alla home”
**Razionale**: lo stesso bottone (cerchio con freccia) è ripetuto in più viste. Centralizzarlo evita divergenze di stile e semplifica modifiche future.

**Cosa fare**
- Creare `src/lib/components/ui/BackCircleButton.svelte` con prop:
  - `onClick` (function)
  - `ariaLabel` (default: “Torna alla home”)
  - `size` (default: 22)
  - `className` opzionale
- Implementare l’icona con `Icon name="arrowLeft"` e la classe condivisa `.back-circle`.

**Sostituzioni**
- `src/App.svelte` (ambulatorio list + impostazioni)
- `src/lib/views/PatientDetail.svelte` (scheda paziente)

**Risultato atteso**
- Un solo punto per stile/struttura del back button.

---

## 2) Unificare tutti i campi data/ora sui componenti mascherati
**Razionale**: ridurre parsing manuale sparso e garantire UX uniforme.

**Cosa fare**
- Usare sempre `MaskedDateInput` e `MaskedTimeInput`.
- Rimuovere eventuali helper locali duplicati (parser/formattatori) dove non servono più.
- Applicare `maxDate` **solo** dove ha senso (es. data di nascita sì, data TAVI no).

**Checklist tecnica**
- Verifica con `rg 'type="date"|type="time"|gg/mm/aaaa'` che non restino input raw.

---

## 3) Helper condivisi per validazioni ripetute
**Razionale**: le stesse validazioni con toast sono ripetute in più funzioni.

**Cosa fare**
- Creare `src/lib/utils/validationHelpers.js` con funzioni:
  - `requireValue(value, message)`
  - `requireValid(isInvalid, message)`
  - `requireAll([...])` (opzionale)
- Sostituire i blocchi ripetuti in:
  - `saveAmbulatorioAppointment`
  - `saveAnagrafica`
  - `saveNewPatient`
  - `saveTaviDate`

**Risultato atteso**
- Meno righe e messaggi coerenti.

---

## 4) Centralizzare stati, label e colori
**Razionale**: status options/label/color sono duplicati tra App e PatientDetail.

**Cosa fare**
- Nuovo file `src/lib/constants/status.js`:
  - `STATUS_OPTIONS`
  - `PRIORITY_OPTIONS`
  - `ACTIVE_PROGRESS_KEYS`
  - `ELIGIBLE_TAVI_STATUSES`
  - `STATUS_COLORS`
- Importare ovunque questi valori.

**Risultato atteso**
- Un solo punto per cambiare stati e label.

---

## 5) Helper per toast e gestione errori
**Razionale**: `showToast` e `console.error` sono ripetuti.

**Cosa fare**
- Nuovo file `src/lib/utils/notify.js` con:
  - `notifySuccess(message)`
  - `notifyError(err, fallback)`
- Sostituire blocchi ripetitivi in `App.svelte`, `PatientDetail.svelte`, `ProcedureForm.svelte`.

**Risultato atteso**
- Logging coerente e meno boilerplate.

---

## 6) Classi layout condivise in `app.css`
**Razionale**: classi grid/padding sono ripetute nei form.

**Cosa fare**
- Aggiungere utility in `src/app.css`:
  - `.form-grid-2`, `.form-grid-3`
  - `.section-card`
  - `.section-header-row`
- Sostituire le classi ripetute nei file principali.

**Risultato atteso**
- Layout uniforme e modifiche centralizzate.

---

## 7) Micro‑componenti per sezioni ripetitive
**Razionale**: grandi sezioni form con struttura simile causano duplicazione.

**Cosa fare**
- Creare componenti leggeri:
  - `SectionCard.svelte` (titolo + contenuto)
  - `FormGrid.svelte` (wrapper grid con prop `cols`)
- Applicare inizialmente a:
  - `ProcedureForm.svelte`
  - Sezioni ambulatorio in `PatientDetail.svelte`

**Risultato atteso**
- Riduzione markup senza cambi logici.

---

## Ordine di implementazione consigliato
1. BackCircleButton
2. Centralizzazione stati/label
3. Helper toast/validazioni
4. Utility layout
5. Micro‑componenti form

---

## Test e verifiche
- Navigazione: il back button funziona in tutte le viste.
- Form: input data/ora validi e coerenti.
- Toast: messaggi corretti e non duplicati.
- Stati: label e badge coerenti ovunque.
- Layout: nessun cambio visivo indesiderato.

---

## Note
Piano compatibile con `REGOLE.md`: riuso componenti, stili condivisi, nessun hardcode locale.
