<script>
  import { onMount } from 'svelte';
  import ToastContainer from './lib/components/ui/ToastContainer.svelte';
  import Button from './lib/components/ui/Button.svelte';
  import Card from './lib/components/ui/Card.svelte';
  import Input from './lib/components/ui/Input.svelte';
  import Checkbox from './lib/components/ui/Checkbox.svelte';
  import IconBadge from './lib/components/ui/IconBadge.svelte';
  import PatientDetail from './lib/views/PatientDetail.svelte';
  import { showToast } from './lib/stores/toastStore.js';
  import {
    patients,
    patientsByStatus,
    statusCounts,
    selectedPatient,
    loadPatients,
    loadStatusCounts,
    loadPatient,
    createPatient
  } from './lib/stores/patientStore.js';
  import { formatDateIT, calculateAge } from './lib/utils/dateUtils.js';
  import { loadPlaces } from './lib/utils/placeSuggestions.js';
  import { open as openDialog } from '@tauri-apps/api/dialog';
  import { writeTextFile, removeFile, createDir } from '@tauri-apps/api/fs';
  import { join, dirname, appDataDir, documentDir, homeDir } from '@tauri-apps/api/path';
  import { invoke } from '@tauri-apps/api/tauri';

  const STATUS_CONFIG = [
    { key: 'Da valutare', label: 'Da valutare' },
    { key: 'In corso di accertamenti', label: 'In corso di accertamenti' },
    { key: 'In attesa di TAVI', label: 'In attesa di TAVI' },
    { key: 'TAVI eseguita', label: 'TAVI eseguita' },
    { key: 'Non candidabile a TAVI', label: 'Non candidabili a TAVI' },
  ];
  const ACTIVE_PROGRESS_KEYS = new Set([
    'Da valutare',
    'In corso di accertamenti',
    'In attesa di TAVI',
  ]);

  let currentView = 'home';
  let selectedPatientId = null;
  let loadingDetail = false;
  const PLACE_DATA = loadPlaces();
  let showNewPatientModal = false;
  let savingPatient = false;
  let placeData = PLACE_DATA;
  let showProvSuggestions = false;
  let showLuogoSuggestions = false;
  let newPatientDateDisplay = '';
  let newPatientDateError = '';
  let newPatientForm = {
    nome: '',
    cognome: '',
    data_nascita: '',
    sesso: 'M',
    luogo_nascita: '',
    luogo_nascita_codice: '',
    codice_fiscale: '',
    telefono: '',
    email: '',
    provenienza: '',
  };
  let formErrors = {};
  let statusCountsValue = [];
  $: statusCountsValue = $statusCounts;
  // Impostazioni (solo UI locale per ora)
  const DEFAULT_REFERTI_PROC_NAMING = 'Scheda procedurale - {cognome} {nome}.docx';
  const DEFAULT_REFERTI_AMB_NAMING = 'Referto ambulatorio - {cognome} {nome}.docx';
  const DEFAULT_REFERTI_HELPER = 'Placeholders: {nome} {cognome} {dn} ecc.';
  let settings = {
    dbPath: '',
    backupPath: '',
    refertiAmbPath: '',
    refertiProcPath: '',
    namingAmb: DEFAULT_REFERTI_AMB_NAMING,
    namingProc: DEFAULT_REFERTI_PROC_NAMING,
    autoOpenReferti: true,
  };
  let settingsErrors = {};
  let showInitialSetup = false;

  const pickDefined = (obj = {}) =>
    Object.fromEntries(
      Object.entries(obj).filter(
        ([, v]) => v !== undefined && v !== null && v !== ''
      )
    );

  const isWindows = () => typeof navigator !== 'undefined' && navigator.userAgent?.includes('Windows');
  const pathSep = () => (isWindows() ? '\\' : '/');

  const safeJoin = async (base, ...parts) => {
    try {
      return await join(base, ...parts);
    } catch (e) {
      const clean = [base, ...parts]
        .filter(Boolean)
        .map((p, idx) => {
          if (idx === 0) return p.replace(/[\\/]+$/, '');
          return p.replace(/^[\\/]+/, '');
        })
        .join(pathSep());
      return clean;
    }
  };

  const normalizeRelative = async (value, base) => {
    if (!value) return value;
    // Rimuovi eventuale prefisso "./" o ".\"
    value = value.replace(/^\.[/\\]/, '');
    const isAbsolute = value.startsWith('/') || /^[A-Za-z]:[\\/]/.test(value);
    if (isAbsolute) return value;
    if (!base) return value;
    try {
      return await safeJoin(base, value);
    } catch (e) {
      return value;
    }
  };

  const safeDirname = async (p) => {
    if (!p?.trim()) return null;
    try {
      return await dirname(p);
    } catch (e) {
      if (p.includes('/')) return p.slice(0, p.lastIndexOf('/'));
      if (p.match(/^[A-Za-z]:[\\/]/) && p.lastIndexOf('\\') > 2) {
        return p.slice(0, p.lastIndexOf('\\'));
      }
      return null;
    }
  };

  async function browsePath(target, opts = { directory: true, key: '', defaultFile: '' }) {
    try {
      const result = await openDialog({ directory: opts.directory, multiple: false });
      if (typeof result === 'string' && result) {
        let path = result;
        if (opts.directory && opts.defaultFile) {
          path = await safeJoin(result, opts.defaultFile);
        }
        settings = { ...settings, [opts.key]: path };
      }
    } catch (e) {
      console.error(e);
      showToast('Errore durante la selezione del percorso', 'error');
    }
  }

  async function loadPersistedSettings() {
    let next = { ...settings };
    let hadPersisted = false;
    let initialSnapshot = { ...next };
    // Prefer backend config file; fallback a local cache se in browser.
    try {
      const loaded = await invoke('load_settings');
      if (loaded && typeof loaded === 'object') {
        const picked = pickDefined(loaded);
        if (Object.keys(picked).length > 0) hadPersisted = true;
        next = { ...next, ...picked };
      }
    } catch (e) {
      console.error('Errore caricamento impostazioni da backend', e);
    }

    try {
      const stored = localStorage.getItem('tavi_settings');
      if (stored) {
        const parsed = JSON.parse(stored);
        const picked = pickDefined(parsed);
        if (Object.keys(picked).length > 0) hadPersisted = true;
        next = { ...next, ...picked };
      }
    } catch (e) {
      console.error('Errore caricamento impostazioni locale', e);
    }

    // Defaults calcolati sul filesystem corrente o fallback statici
    const defaults = {};
    try {
      const dataDir = await appDataDir().catch(() => '');
      const docsDir = await documentDir().catch(() => '');
      const home = await homeDir().catch(() => '');
      const fallbackBase = home || dataDir || docsDir || (isWindows() ? 'C:\\\\Users\\\\Public' : '/Users/Shared');
      const baseData = dataDir || home || docsDir || fallbackBase;
      const baseDocs = docsDir || home || dataDir || fallbackBase;
      const fallback = baseDocs;

      // Normalizza percorsi eventualmente relativi già presenti
      next.dbPath = await normalizeRelative(next.dbPath, baseData || fallback);
      next.backupPath = await normalizeRelative(next.backupPath, baseData || fallback);
      next.refertiAmbPath = await normalizeRelative(next.refertiAmbPath, baseDocs || fallback);
      next.refertiProcPath = await normalizeRelative(next.refertiProcPath, baseDocs || fallback);

      if (!next.dbPath) {
        defaults.dbPath = await safeJoin(baseData, 'pazienti_tavi.db');
      }
      if (!next.backupPath) {
        defaults.backupPath = await safeJoin(baseData, 'backup');
      }
      if (!next.refertiAmbPath) {
        defaults.refertiAmbPath = await safeJoin(baseDocs, 'Referti TAVI', 'Ambulatorio strutturale');
      }
      if (!next.refertiProcPath) {
        defaults.refertiProcPath = await safeJoin(baseDocs, 'Referti TAVI', 'Schede procedurali');
      }
    } catch (e) {
      console.error('Errore nel calcolo dei percorsi di default', e);
    }

    const finalSettings = { ...next, ...pickDefined(defaults) };

    const changed =
      JSON.stringify(pickDefined(finalSettings)) !== JSON.stringify(pickDefined(initialSnapshot));

    settings = finalSettings;

    // Se abbiamo normalizzato o aggiunto default, persistiamo subito solo se avevamo già delle impostazioni salvate.
    if (hadPersisted && changed) {
      try {
        await invoke('save_settings', { settings: finalSettings });
        localStorage.setItem('tavi_settings', JSON.stringify(finalSettings));
      } catch (e) {
        console.error('Errore nel salvataggio automatico impostazioni normalizzate', e);
      }
    }

    // Se non c'era nulla salvato, chiedi all'utente al primo avvio.
    showInitialSetup = !hadPersisted;
  }

  async function checkWritableDir(dir) {
    if (!dir) return 'Percorso obbligatorio';
    // Normalizza eventuali percorsi relativi verso una base sicura (home/doc/appData).
    let targetDir = dir;
    if (!dir.startsWith('/') && !dir.match(/^[A-Za-z]:[\\/]/)) {
      try {
        const base =
          (await documentDir().catch(() => '')) ||
          (await homeDir().catch(() => '')) ||
          (await appDataDir().catch(() => '')) ||
          (isWindows() ? 'C:\\\\Users\\\\Public' : '/Users/Shared');
        if (base) {
          targetDir = await join(base, dir);
        }
      } catch (e) {
        // fallback: usa così com'è
      }
    }
    try {
      await createDir(targetDir, { recursive: true });
      const testFile = await join(targetDir, '__codex_test__');
      await writeTextFile(testFile, 'ok');
      await removeFile(testFile).catch(() => {});
      return '';
    } catch (e) {
      // Se il test di scrittura fallisce, non blocchiamo il salvataggio ma logghiamo per debug.
      console.warn('write check failed (non bloccante)', targetDir, e);
      return '';
    }
  }

  async function validateAndSaveSettings() {
    // Normalizza percorsi prima di validare
    try {
      const dataDir = await appDataDir().catch(() => '');
      const docsDir = await documentDir().catch(() => '');
      const home = await homeDir().catch(() => '');
      const baseData = dataDir || home || docsDir || (isWindows() ? 'C:\\\\Users\\\\Public' : '/Users/Shared');
      const baseDocs = docsDir || home || dataDir || (isWindows() ? 'C:\\\\Users\\\\Public' : '/Users/Shared');

      settings = {
        ...settings,
        dbPath: await normalizeRelative(settings.dbPath, baseData),
        backupPath: await normalizeRelative(settings.backupPath, baseData),
        refertiAmbPath: await normalizeRelative(settings.refertiAmbPath, baseDocs),
        refertiProcPath: await normalizeRelative(settings.refertiProcPath, baseDocs),
      };
    } catch (e) {
      console.warn('Normalizzazione percorsi in salvataggio impostazioni fallita', e);
    }

    const errors = {};
    if (!settings.dbPath?.trim()) errors.dbPath = 'Percorso database obbligatorio';
    if (!settings.backupPath?.trim()) errors.backupPath = 'Cartella backup obbligatoria';
    if (!settings.refertiAmbPath?.trim()) errors.refertiAmbPath = 'Cartella referti ambulatorio obbligatoria';
    if (!settings.refertiProcPath?.trim()) errors.refertiProcPath = 'Cartella referti procedurale obbligatoria';
    if (!settings.namingAmb?.trim()) errors.namingAmb = 'Naming obbligatorio';
    if (!settings.namingProc?.trim()) errors.namingProc = 'Naming obbligatorio';

    if (!errors.backupPath) {
      const msg = await checkWritableDir(settings.backupPath);
      if (msg) errors.backupPath = msg;
    }
    if (!errors.refertiAmbPath) {
      const msg = await checkWritableDir(settings.refertiAmbPath);
      if (msg) errors.refertiAmbPath = msg;
    }
    if (!errors.refertiProcPath) {
      const msg = await checkWritableDir(settings.refertiProcPath);
      if (msg) errors.refertiProcPath = msg;
    }
    if (!errors.dbPath) {
      const dir = await safeDirname(settings.dbPath);
      if (!dir) {
        errors.dbPath = 'Percorso database non valido';
      } else {
        const msg = await checkWritableDir(dir);
        if (msg) errors.dbPath = msg;
      }
    }

    settingsErrors = errors;
    if (Object.keys(errors).length > 0) {
      showToast('Correggi gli errori nei campi impostazioni', 'error');
      return false;
    }

    try {
      await invoke('save_settings', { settings });
      localStorage.setItem('tavi_settings', JSON.stringify(settings));
      showToast('Impostazioni salvate', 'success');
      return true;
    } catch (e) {
      console.error(e);
      showToast('Errore nel salvataggio impostazioni', 'error');
      return false;
    }
  }

  onMount(() => {
    loadPersistedSettings();
  });
  $: statusSummaries = STATUS_CONFIG.map(({ key, label }) => {
    const grouped = ($patientsByStatus[key] || []).length;
    const backend = statusCountsValue.find((c) => c.status === key)?.count ?? null;
    const card = getCount(key);
    return { key, label, grouped, backend, card };
  });
  $: activeProgressTotal = statusSummaries
    .filter((s) => ACTIVE_PROGRESS_KEYS.has(s.key))
    .reduce((sum, s) => sum + (s.card || 0), 0);
  $: summaryMap = Object.fromEntries(statusSummaries.map((s) => [s.key, s]));
  $: statusCardGroups = [
    ...statusSummaries.slice(0, 3),
    {
      key: 'combo_tavi_non',
      combo: true,
      entries: [
        summaryMap['TAVI eseguita'],
        summaryMap['Non candidabile a TAVI'],
      ].filter(Boolean),
    },
  ];

  onMount(async () => {
    await refreshData();
  });

  async function refreshData() {
    await Promise.all([loadPatients(), loadStatusCounts()]);
  }

  async function openPatient(patient) {
    selectedPatientId = patient?.patient?.id ?? patient?.id ?? null;
    if (!selectedPatientId) return;

    loadingDetail = true;
    try {
      await loadPatient(selectedPatientId);
      currentView = 'patient-detail';
    } finally {
      loadingDetail = false;
    }
  }

  async function backToHome() {
    currentView = 'home';
    selectedPatientId = null;
    await refreshData();
  }

  function openSettings() {
    currentView = 'settings';
  }

  // Conta per stato: prima dai conteggi back-end, fallback a derived locale.
  function getCount(status) {
    const derivedCount = ($patientsByStatus[status] || []).length;
    const fromCounts = statusCountsValue.find((c) => c.status === status)?.count;
    if (fromCounts !== undefined) {
      // Se il backend restituisce 0 ma i dati locali hanno elementi, usa il fallback locale.
      return fromCounts === 0 ? derivedCount : fromCounts;
    }
    return derivedCount;
  }

  function getStatusColor(status) {
    const colors = {
      'Da valutare': 'bg-blue-100 text-blue-800',
      'In corso di accertamenti': 'bg-yellow-100 text-yellow-800',
      'In attesa di TAVI': 'bg-purple-100 text-purple-800',
      'Non candidabile a TAVI': 'bg-red-100 text-red-800',
      'TAVI eseguita': 'bg-green-100 text-green-800',
    };
    return colors[status] || 'bg-gray-100 text-gray-800';
  }

  $: totalPatients =
    statusCountsValue.length > 0
      ? statusCountsValue.reduce((sum, item) => sum + (item.count || 0), 0)
      : $patients.length;
  const MIN_BAR_WIDTH = 8; // percent to keep bar visible when count > 0

  function getProgress(statusKey, count) {
    if (!ACTIVE_PROGRESS_KEYS.has(statusKey) || activeProgressTotal === 0) return '0%';
    const width = Math.max(
      (count / activeProgressTotal) * 100,
      count > 0 ? MIN_BAR_WIDTH : 0
    );
    return `${width}%`;
  }

  const filterPlaces = (value) => {
    if (!value || value.trim().length < 2) return [];
    const q = value.trim().toLowerCase();
    const base = [...(placeData.comuni || []), ...(placeData.stati || [])];
    return base
      .map(item => {
        const label =
          typeof item === 'string'
            ? item
            : item.nome || item.name || item.label || '';
        const codice =
          typeof item === 'object'
            ? item.codice_catastale || item.codice || null
            : null;
        return { label, codice };
      })
      .filter(item => item.label.toLowerCase().includes(q))
      .slice(0, 12);
  };

  function capitalizeWordsStrict(value) {
    if (!value) return '';
    return value
      .toLowerCase()
      .split(' ')
      .filter(Boolean)
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  }

  function formatDateInput(value) {
    const digits = (value || '').replace(/\D/g, '').slice(0, 8);
    const day = digits.slice(0, 2);
    const month = digits.slice(2, 4);
    const year = digits.slice(4, 8);
    let display = '';
    if (day) display += day;
    if (month) display += (display ? '/' : '') + month;
    if (year) display += (display ? '/' : '') + year;
    let iso = '';
    let error = '';
    if (day.length === 2 && month.length === 2 && year.length === 4) {
      const dNum = Number(day);
      const mNum = Number(month);
      const yNum = Number(year);
      const today = new Date();
      const currentYear = today.getFullYear();
      if (dNum < 1 || dNum > 31) {
        error = 'Giorno non valido';
      } else if (mNum < 1 || mNum > 12) {
        error = 'Mese non valido';
      } else if (yNum < 1900 || yNum > currentYear) {
        error = 'Anno non valido';
      } else {
        const candidate = new Date(`${year}-${month}-${day}T00:00:00`);
        if (isNaN(candidate.getTime())) {
          error = 'Data non valida';
        } else if (candidate > today) {
          error = 'La data non può essere futura';
        } else {
          iso = `${year}-${month}-${day}`;
        }
      }
    }
    return { display, iso, error };
  }

  function formatDisplayFromDigits(digits) {
    const d = digits.replace(/\D/g, '').slice(0, 8);
    let out = '';
    for (let i = 0; i < d.length; i++) {
      out += d[i];
      if (i === 1 || i === 3) out += '/';
    }
    return out;
  }

  function handleNewDateInput(value) {
    const cleaned = (value || '').replace(/\D/g, '').slice(0, 8);
    const display = formatDisplayFromDigits(cleaned);
    const { iso, error } = formatDateInput(cleaned);
    newPatientDateDisplay = display;
    newPatientForm.data_nascita = iso;
    newPatientDateError = error;
    updateNewPatientCF();
  }

  function calcCodiceFiscale({ nome, cognome, data_nascita, sesso, luogo_nascita_codice }) {
    if (!nome || !cognome || !data_nascita || !sesso || !luogo_nascita_codice) return '';
    const oddMap = {
      '0': 1, '1': 0, '2': 5, '3': 7, '4': 9, '5': 13, '6': 15, '7': 17, '8': 19, '9': 21,
      A: 1, B: 0, C: 5, D: 7, E: 9, F: 13, G: 15, H: 17, I: 19, J: 21,
      K: 2, L: 4, M: 18, N: 20, O: 11, P: 3, Q: 6, R: 8, S: 12, T: 14,
      U: 16, V: 10, W: 22, X: 25, Y: 24, Z: 23
    };
    const evenMap = {
      '0': 0, '1': 1, '2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7, '8': 8, '9': 9,
      A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9,
      K: 10, L: 11, M: 12, N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19,
      U: 20, V: 21, W: 22, X: 23, Y: 24, Z: 25
    };

    const vowels = 'AEIOU';
    const consonantsOf = str => str.replace(/[^A-Z]/gi, '').toUpperCase().split('').filter(c => !vowels.includes(c));
    const vowelsOf = str => str.replace(/[^A-Z]/gi, '').toUpperCase().split('').filter(c => vowels.includes(c));

    const pad = arr => {
      while (arr.length < 3) arr.push('X');
      return arr;
    };

    const surnameCons = consonantsOf(cognome);
    const surnameVowels = vowelsOf(cognome);
    const surnameCode = pad([...surnameCons, ...surnameVowels]).slice(0, 3).join('');

    const nameCons = consonantsOf(nome);
    let nameCode;
    if (nameCons.length >= 4) {
      nameCode = [nameCons[0], nameCons[2], nameCons[3]];
    } else {
      nameCode = [...nameCons, ...vowelsOf(nome)];
      nameCode = pad(nameCode).slice(0, 3);
    }
    nameCode = nameCode.join('');

    const date = new Date(data_nascita);
    if (isNaN(date.getTime())) return '';
    const year = String(date.getFullYear()).slice(-2);
    const monthCodes = 'ABCDEHLMPRST';
    const month = monthCodes[date.getMonth()];
    const day = date.getDate() + (sesso === 'F' ? 40 : 0);
    const dayCode = String(day).padStart(2, '0');

    const partial = `${surnameCode}${nameCode}${year}${month}${dayCode}${luogo_nascita_codice}`.toUpperCase();
    if (partial.length !== 15) return '';

    let sum = 0;
    for (let i = 0; i < partial.length; i++) {
      const ch = partial[i];
      const map = (i + 1) % 2 === 0 ? evenMap : oddMap; // posizioni dispari/pari 1-based
      sum += map[ch] ?? 0;
    }
    const checkChar = String.fromCharCode('A'.charCodeAt(0) + (sum % 26));
    return `${partial}${checkChar}`;
  }

  function updateNewPatientCF() {
    const cf = calcCodiceFiscale(newPatientForm);
    if (cf) {
      newPatientForm.codice_fiscale = cf;
    }
  }

  async function openNewPatientModal() {
    formErrors = {};
    newPatientForm = {
      nome: '',
      cognome: '',
      data_nascita: '',
      sesso: 'M',
      luogo_nascita: '',
      luogo_nascita_codice: '',
      codice_fiscale: '',
      telefono: '',
      email: '',
      provenienza: '',
    };
    newPatientDateDisplay = '';
    newPatientDateError = '';
    showNewPatientModal = true;
  }

  async function closeNewPatientModal() {
    showNewPatientModal = false;
    await refreshData();
  }

  async function handleInitialSetupSave() {
    const ok = await validateAndSaveSettings();
    if (ok) {
      showInitialSetup = false;
    }
  }

  function validateNewPatient() {
    const errors = {};
    if (!newPatientForm.nome.trim()) errors.nome = 'Nome obbligatorio';
    if (!newPatientForm.cognome.trim()) errors.cognome = 'Cognome obbligatorio';
    if (!newPatientForm.data_nascita || newPatientDateError) {
      errors.data_nascita = newPatientDateError || 'Data di nascita obbligatoria';
    }
    return errors;
  }

  const normalize = (value) => {
    if (value === undefined || value === null) return null;
    const trimmed = value.trim();
    return trimmed === '' ? null : trimmed;
  };

  async function saveNewPatient() {
    formErrors = validateNewPatient();
    if (Object.keys(formErrors).length > 0) {
      showToast('Compila i campi obbligatori', 'warning');
      return;
    }

    const payload = {
      nome: capitalizeWordsStrict(newPatientForm.nome),
      cognome: capitalizeWordsStrict(newPatientForm.cognome),
      data_nascita: newPatientForm.data_nascita,
      luogo_nascita: normalize(newPatientForm.luogo_nascita),
      codice_fiscale: normalize(newPatientForm.codice_fiscale),
      telefono: normalize(newPatientForm.telefono),
      email: normalize(newPatientForm.email),
      provenienza: normalize(newPatientForm.provenienza),
      sesso: newPatientForm.sesso || null,
    };

    savingPatient = true;
    try {
      await createPatient(payload);
      // Assicurati che i dati siano aggiornati subito dopo l'inserimento
      await refreshData();
      showToast('Paziente creato e inserito in "Da valutare"', 'success');
      await closeNewPatientModal();
    } catch (err) {
      console.error(err);
      showToast('Errore durante la creazione del paziente', 'error');
    } finally {
      savingPatient = false;
    }
  }
</script>

<main class="h-screen bg-background text-textPrimary flex flex-col">
  <!-- Header -->
  <header class="h-16 bg-white border-b border-gray-200 px-6 flex items-center justify-between flex-shrink-0">
    <div class="flex items-center gap-4">
      <img src="/icons/icon.png" alt="Logo TAVI" class="h-10 w-10 rounded-lg" />
      <h1 class="text-xl font-semibold text-textPrimary">Gestionale Pazienti TAVI</h1>
    </div>
    {#if currentView === 'patient-detail' || currentView === 'settings'}
      <button
        on:click={backToHome}
        class="px-4 py-2 rounded-lg bg-secondary text-white hover:bg-secondary/90 transition-colors"
      >
        ← Torna alla Home
      </button>
    {/if}
  </header>

  <!-- Main content area -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if currentView === 'home'}
      <div class="max-w-7xl mx-auto space-y-8">
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div>
            <h2 class="text-2xl font-bold text-textPrimary">Pazienti TAVI</h2>
            <p class="text-textSecondary mt-1">Gestisci i pazienti per stato e checklist</p>
          </div>
          <div class="flex items-center gap-3">
            <Button
              variant="primary"
              class="flex flex-col items-center gap-1 min-w-[120px]"
              on:click={openNewPatientModal}
            >
              <IconBadge icon="userPlus" size="lg" tone="primary" class="mb-1" />
              <span class="text-xs font-semibold">Nuovo paziente</span>
            </Button>
            <Button
              variant="secondary"
              class="flex flex-col items-center gap-1 min-w-[120px]"
              on:click={openSettings}
            >
              <IconBadge icon="settings" size="lg" tone="neutral" class="mb-1" />
              <span class="text-xs font-semibold">Impostazioni</span>
            </Button>
          </div>
        </div>

        <!-- Status cards grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {#each statusCardGroups as summary (summary.key)}
            {#if !summary.combo}
              <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                <div class="flex items-center justify-between mb-2">
                  <h3 class="font-semibold text-textPrimary">{summary.label}</h3>
                  <span class="text-2xl font-bold text-primary">{summary.card}</span>
                </div>
                {#if ACTIVE_PROGRESS_KEYS.has(summary.key)}
                  <div class="h-2 bg-gray-100 rounded-full overflow-hidden">
                    <div
                      class="h-full bg-primary transition-all duration-300"
                      style={`width: ${getProgress(summary.key, summary.card)}`}
                    />
                  </div>
                {/if}
              </div>
            {:else}
              <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 space-y-3">
                {#each summary.entries as entry (entry.key)}
                  <div class="flex items-center justify-between">
                    <h3 class="font-semibold text-textPrimary">{entry.label}</h3>
                    <span class="text-2xl font-bold text-primary">{entry.card}</span>
                  </div>
                {/each}
              </div>
            {/if}
          {/each}
        </div>

        <!-- Patient tables grouped by status -->
        {#each statusSummaries as summary (summary.key)}
          {@const statusPatients = $patientsByStatus[summary.key] || []}

          <div class="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
            <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
              <h2 class="text-lg font-semibold text-textPrimary">{summary.label}</h2>
              <span class="px-3 py-1 rounded-full text-sm font-medium {getStatusColor(summary.key)}">
                {statusPatients.length} pazient{statusPatients.length !== 1 ? 'i' : 'e'}
              </span>
            </div>

            {#if statusPatients.length === 0}
              <div class="px-6 py-12 text-center text-textSecondary">
                Nessun paziente in questo stato
              </div>
            {:else}
              <div class="overflow-x-auto">
                <table class="w-full">
                  <thead class="bg-gray-50 border-b border-gray-200">
                    <tr>
                      <th class="px-6 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                        Paziente
                      </th>
                      <th class="px-6 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                        Età
                      </th>
                      <th class="px-6 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                        Data Nascita
                      </th>
                      <th class="px-6 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                        Provenienza
                      </th>
                      <th class="px-6 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                        Inserito il
                      </th>
                      <th class="px-6 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                        Telefono
                      </th>
                    </tr>
                  </thead>
                  <tbody class="bg-white divide-y divide-gray-200">
                    {#each statusPatients as patient}
                      <tr
                        on:click={() => openPatient(patient)}
                        class="hover:bg-gray-50 cursor-pointer transition-colors"
                      >
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="font-medium text-textPrimary">
                            {patient.patient?.nome} {patient.patient?.cognome}
                          </div>
                          {#if patient.patient?.codice_fiscale}
                            <div class="text-sm text-textSecondary">
                              {patient.patient?.codice_fiscale}
                            </div>
                          {/if}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-textSecondary">
                          {#if patient.patient?.data_nascita}
                            {calculateAge(patient.patient?.data_nascita)} anni
                          {:else}
                            -
                          {/if}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-textSecondary">
                          {formatDateIT(patient.patient?.data_nascita)}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-textSecondary">
                          {patient.patient?.provenienza || '-'}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-textSecondary">
                          {formatDateIT(patient.status_created_at?.split(' ')[0])}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-textSecondary">
                          {patient.patient?.telefono || '-'}
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {:else if currentView === 'settings'}
      <div class="max-w-7xl mx-auto space-y-8">
        <div class="space-y-2">
          <h2 class="text-2xl font-bold text-textPrimary">Impostazioni</h2>
          <p class="text-textSecondary">Configura salvataggio dati e referti.</p>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card padding="lg" class="border border-gray-200 space-y-4">
            <div>
              <h3 class="text-lg font-semibold text-textPrimary flex items-center gap-2">
                <IconBadge icon="database" tone="neutral" /> Database
              </h3>
              <p class="text-sm text-textSecondary">Percorso salvataggio e gestione backup.</p>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
              <Input
                label="Percorso database"
                bind:value={settings.dbPath}
                error={settingsErrors.dbPath}
              />
              <Button
                variant="secondary"
                size="sm"
                class="self-start sm:mt-6"
                on:click={() =>
                  browsePath('database', {
                    directory: true,
                    key: 'dbPath',
                    defaultFile: 'pazienti_tavi.db',
                  })}
              >
                Sfoglia
              </Button>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
              <Input
                label="Cartella backup"
                bind:value={settings.backupPath}
                error={settingsErrors.backupPath}
              />
              <Button
                variant="secondary"
                size="sm"
                class="self-start sm:mt-6"
                on:click={() => browsePath('backup', { directory: true, key: 'backupPath' })}
              >
                Sfoglia
              </Button>
            </div>
          </Card>

          <Card padding="lg" class="border border-gray-200 space-y-4">
            <div>
              <h3 class="text-lg font-semibold text-textPrimary flex items-center gap-2">
                <IconBadge icon="file" tone="neutral" /> Referti
              </h3>
              <p class="text-sm text-textSecondary">Percorso salvataggio, naming e apertura automatica.</p>
            </div>
            <div class="space-y-2">
              <p class="text-sm font-semibold text-textPrimary">Ambulatorio strutturale</p>
              <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
                <Input
                  label="Cartella referti ambulatorio"
                  bind:value={settings.refertiAmbPath}
                  error={settingsErrors.refertiAmbPath}
                />
                <Button variant="secondary" size="sm" class="self-start sm:mt-6" on:click={() => browsePath('referti ambulatorio', { directory: true, key: 'refertiAmbPath' })}>
                  Sfoglia
                </Button>
              </div>
              <Input
                label="Naming file ambulatorio"
                bind:value={settings.namingAmb}
                helperText={DEFAULT_REFERTI_HELPER}
                error={settingsErrors.namingAmb}
              />
            </div>

            <div class="space-y-2">
              <p class="text-sm font-semibold text-textPrimary">Scheda procedurale</p>
              <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
                <Input
                  label="Cartella referti procedurale"
                  bind:value={settings.refertiProcPath}
                  error={settingsErrors.refertiProcPath}
                />
                <Button variant="secondary" size="sm" class="self-start sm:mt-6" on:click={() => browsePath('referti procedurale', { directory: true, key: 'refertiProcPath' })}>
                  Sfoglia
                </Button>
              </div>
              <Input
                label="Naming file procedurale"
                bind:value={settings.namingProc}
                helperText={DEFAULT_REFERTI_HELPER}
                error={settingsErrors.namingProc}
              />
            </div>
            <div class="flex items-center gap-2">
              <Checkbox
                checked={settings.autoOpenReferti}
                on:change={() => (settings.autoOpenReferti = !settings.autoOpenReferti)}
              />
              <span class="text-sm text-textPrimary">Apri automaticamente dopo la generazione</span>
            </div>
          </Card>
        </div>

        <div class="flex justify-end">
          <Button variant="primary" on:click={validateAndSaveSettings}>Salva impostazioni</Button>
        </div>
      </div>
    {:else if currentView === 'patient-detail'}
      <div class="max-w-6xl mx-auto">
        <PatientDetail
          patient={$selectedPatient}
          loading={loadingDetail}
          onBack={backToHome}
        />
      </div>
    {/if}
  </div>
</main>

{#if showNewPatientModal}
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center p-4 z-40"
    on:click={closeNewPatientModal}
  >
    <Card
      padding="lg"
      class="max-w-3xl w-full bg-white shadow-xl"
      on:click={(e) => e.stopPropagation()}
    >
      <div class="flex items-start justify-between gap-4 mb-6">
        <div>
          <p class="text-sm text-textSecondary">Nuovo paziente</p>
          <h3 class="text-2xl font-bold text-textPrimary">Aggiungi paziente</h3>
          <p class="text-sm text-textSecondary mt-1">
            Inserisci i dati minimi, potrai completare le informazioni e la checklist dopo la creazione.
          </p>
        </div>
        <Button variant="text" size="sm" on:click={closeNewPatientModal}>
          <IconBadge icon="close" tone="neutral" />
        </Button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Input
          label="Nome"
          required
          placeholder="Mario"
          bind:value={newPatientForm.nome}
          on:blur={() => {
            newPatientForm.nome = capitalizeWordsStrict(newPatientForm.nome);
            updateNewPatientCF();
          }}
          on:input={updateNewPatientCF}
          error={formErrors.nome}
        />
        <Input
          label="Cognome"
          required
          placeholder="Rossi"
          bind:value={newPatientForm.cognome}
          on:blur={() => {
            newPatientForm.cognome = capitalizeWordsStrict(newPatientForm.cognome);
            updateNewPatientCF();
          }}
          on:input={updateNewPatientCF}
          error={formErrors.cognome}
        />
        <Input
          label="Data di nascita"
          placeholder="gg/mm/aaaa"
          required
          bind:value={newPatientDateDisplay}
          on:input={(e) => handleNewDateInput(e.detail?.target?.value || '')}
          error={formErrors.data_nascita || newPatientDateError}
        />
        <div class="relative">
          <Input
            label="Luogo di nascita"
            placeholder="Roma"
            bind:value={newPatientForm.luogo_nascita}
            on:focus={() => (showLuogoSuggestions = true)}
            on:input={() => (showLuogoSuggestions = true)}
            on:blur={() => setTimeout(() => (showLuogoSuggestions = false), 120)}
          />
          {#if showLuogoSuggestions && filterPlaces(newPatientForm.luogo_nascita).length}
            <div class="absolute z-30 mt-1 w-full bg-white border border-gray-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
              {#each filterPlaces(newPatientForm.luogo_nascita) as suggestion}
                <button
                  type="button"
                  class="w-full text-left px-3 py-2 text-sm hover:bg-gray-50"
                  on:click={() => {
                    newPatientForm.luogo_nascita = suggestion.label;
                    newPatientForm.luogo_nascita_codice = suggestion.codice || '';
                    updateNewPatientCF();
                    showLuogoSuggestions = false;
                  }}
                >
                  {suggestion.label}
                </button>
              {/each}
            </div>
          {/if}
        </div>
        <div>
          <p class="text-sm font-medium text-textPrimary mb-1">Sesso</p>
          <div class="flex items-center gap-6 mt-1 min-h-[40px]">
            <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
              <Checkbox
                checked={newPatientForm.sesso === 'M'}
                on:change={() => {
                  newPatientForm.sesso = 'M';
                  updateNewPatientCF();
                }}
              />
              Maschio
            </label>
            <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
              <Checkbox
                checked={newPatientForm.sesso === 'F'}
                on:change={() => {
                  newPatientForm.sesso = 'F';
                  updateNewPatientCF();
                }}
              />
              Femmina
            </label>
          </div>
        </div>
        <Input
          label="Codice fiscale"
          placeholder="MRARSS80A01H501U"
          bind:value={newPatientForm.codice_fiscale}
        />
        <Input
          label="Telefono"
          type="tel"
          placeholder="3331234567"
          bind:value={newPatientForm.telefono}
        />
        <Input
          label="Email"
          type="email"
          placeholder="nome@ospedale.it"
          bind:value={newPatientForm.email}
        />
        <div class="relative">
          <Input
            label="Provenienza (cardiologo/ospedale)"
            placeholder="Cardiologo o ospedale di provenienza"
            bind:value={newPatientForm.provenienza}
            on:focus={() => (showProvSuggestions = true)}
            on:input={() => (showProvSuggestions = true)}
            on:blur={() => setTimeout(() => (showProvSuggestions = false), 120)}
          />
          {#if showProvSuggestions && filterPlaces(newPatientForm.provenienza).length}
            <div class="absolute z-30 mt-1 w-full bg-white border border-gray-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
              {#each filterPlaces(newPatientForm.provenienza) as suggestion}
                <button
                  type="button"
                  class="w-full text-left px-3 py-2 text-sm hover:bg-gray-50"
                  on:click={() => {
                    newPatientForm.provenienza = suggestion.label;
                    showProvSuggestions = false;
                  }}
                >
                  {suggestion.label}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <div class="flex flex-wrap items-center justify-between gap-3 mt-6">
        <p class="text-sm text-textSecondary">
          Lo stato iniziale sarà <span class="font-semibold text-textPrimary">"Da valutare"</span>.
        </p>
        <div class="flex gap-2">
          <Button variant="text" on:click={closeNewPatientModal}>
            Annulla
          </Button>
          <Button variant="primary" on:click={saveNewPatient} disabled={savingPatient}>
            {savingPatient ? 'Salvataggio...' : 'Salva paziente'}
          </Button>
        </div>
      </div>
    </Card>
  </div>
{/if}

{#if showInitialSetup}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50">
    <Card padding="lg" class="max-w-3xl w-full bg-white shadow-2xl">
      <div class="mb-4">
        <h3 class="text-xl font-bold text-textPrimary">Configura percorsi</h3>
        <p class="text-sm text-textSecondary mt-1">Seleziona dove salvare database e referti.</p>
      </div>
      <div class="space-y-4">
        <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
          <Input
            label="Percorso database"
            bind:value={settings.dbPath}
            error={settingsErrors.dbPath}
          />
          <Button
            variant="secondary"
            size="sm"
            class="self-start sm:mt-6"
            on:click={() =>
              browsePath('database', {
                directory: true,
                key: 'dbPath',
                defaultFile: 'pazienti_tavi.db',
              })}
          >
            Sfoglia
          </Button>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
          <Input
            label="Cartella backup"
            bind:value={settings.backupPath}
            error={settingsErrors.backupPath}
          />
          <Button
            variant="secondary"
            size="sm"
            class="self-start sm:mt-6"
            on:click={() => browsePath('backup', { directory: true, key: 'backupPath' })}
          >
            Sfoglia
          </Button>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
          <Input
            label="Cartella referti ambulatorio"
            bind:value={settings.refertiAmbPath}
            error={settingsErrors.refertiAmbPath}
          />
          <Button
            variant="secondary"
            size="sm"
            class="self-start sm:mt-6"
            on:click={() => browsePath('referti ambulatorio', { directory: true, key: 'refertiAmbPath' })}
          >
            Sfoglia
          </Button>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-[1fr_auto] gap-2 items-start">
          <Input
            label="Cartella referti procedurale"
            bind:value={settings.refertiProcPath}
            error={settingsErrors.refertiProcPath}
          />
          <Button
            variant="secondary"
            size="sm"
            class="self-start sm:mt-6"
            on:click={() => browsePath('referti procedurale', { directory: true, key: 'refertiProcPath' })}
          >
            Sfoglia
          </Button>
        </div>
      </div>
      <div class="flex justify-end gap-2 mt-6">
        <Button variant="secondary" on:click={() => (showInitialSetup = false)}>Annulla</Button>
        <Button variant="primary" on:click={handleInitialSetupSave}>Salva e continua</Button>
      </div>
    </Card>
  </div>
{/if}

<ToastContainer />
