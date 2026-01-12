<script>
  import Card from '../components/ui/Card.svelte';
  import Button from '../components/ui/Button.svelte';
  import Select from '../components/ui/Select.svelte';
  import Checkbox from '../components/ui/Checkbox.svelte';
  import Input from '../components/ui/Input.svelte';
  import RichTextArea from '../components/ui/RichTextArea.svelte';
  import CVRiskFactors from '../components/ui/CVRiskFactors.svelte';
  import IconBadge from '../components/ui/IconBadge.svelte';
  import SectionPanel from '../components/ui/SectionPanel.svelte';
  import { formatDateIT, calculateAge } from '../utils/dateUtils.js';
  import { calculateBMI, calculateBSA, categorizeBMI, formatNumberIT } from '../utils/statistics.js';
  import { changePatientStatus, loadPatient, updatePatient, deletePatient } from '../stores/patientStore.js';
  import { showToast } from '../stores/toastStore.js';
  import { loadPlaces } from '../utils/placeSuggestions.js';
  import { invoke } from '@tauri-apps/api/tauri';
  import { open as openExternal } from '@tauri-apps/api/shell';

  export let patient = null;
  export let loading = false;
  export let onBack = () => {};

  const STATUS_OPTIONS = [
    { value: 'Da valutare', label: 'Da valutare' },
    { value: 'In corso di accertamenti', label: 'In corso di accertamenti' },
    { value: 'In attesa di TAVI', label: 'In attesa di TAVI' },
    { value: 'Non candidabile a TAVI', label: 'Non candidabile a TAVI' },
    { value: 'TAVI eseguita', label: 'TAVI eseguita' },
  ];

  const ECG_OPTIONS = [
    { id: 'ecgRitmoSinusale', label: 'Ritmo sinusale' },
    { id: 'ecgFA', label: 'FA' },
    { id: 'ecgBBS', label: 'BBS' },
    { id: 'ecgBBD', label: 'BBD' },
    { id: 'ecgEAS', label: 'EAS' },
    { id: 'ecgBavPrimo', label: 'BAV di primo' },
    { id: 'ecgRitmoStimolato', label: 'Ritmo stimolato' },
  ];

  const ANESTESIA_OPTIONS = ['Locale', 'Sedazione', 'Generale'];
  const CORONAROGRAFIA_OPTIONS = [
    { value: 'ricovero', label: 'Da eseguire durante ricovero per TAVI' },
    { value: 'gia_eseguita', label: 'Già eseguita' },
  ];

  const VALVOLE_TAVI = [
    { value: 'edwards_sapien_3', label: 'Edwards Sapien 3', sizes: ['20', '23', '26', '29'] },
    { value: 'edwards_sapien_3_ultra', label: 'Edwards Sapien 3 Ultra', sizes: ['20', '23', '26', '29'] },
    { value: 'medtronic_evolut_pro', label: 'Medtronic Evolut PRO / PRO+', sizes: ['23', '26', '29', '34'] },
    { value: 'medtronic_evolut_r', label: 'Medtronic Evolut R', sizes: ['23', '26', '29', '34'] },
    { value: 'boston_accurate_neo2', label: 'Boston Acurate neo2', sizes: ['S (23)', 'M (25)', 'L (27)'] },
    { value: 'abbott_portico_navitor', label: 'Abbott Portico / Navitor', sizes: ['23', '25', '27', '29'] },
    { value: 'meril_myval', label: 'Meril Myval', sizes: ['20', '21.5', '23', '24.5', '26', '27.5', '29', '30.5'] },
  ];

  const getValveSizes = (model) => {
    const found = VALVOLE_TAVI.find(v => v.value === model);
    return found ? found.sizes : [];
  };

  const handleValveModelChange = (model) => {
    schedaProceduraleForm.bioprotesiModello = model;
    const sizes = getValveSizes(model);
    if (!sizes.includes(schedaProceduraleForm.bioprotesiDimensione)) {
      schedaProceduraleForm.bioprotesiDimensione = '';
    }
  };
  const setAccessoPrincipale = (value) => {
    schedaProceduraleForm.accessoPrincipaleFem =
      schedaProceduraleForm.accessoPrincipaleFem === value ? '' : value;
    if (schedaProceduraleForm.accessoPrincipaleFem !== 'altro') {
      schedaProceduraleForm.accessoPrincipaleAltro = '';
    }
  };

  const CHECKLIST_SECTIONS = [
    {
      id: 'ematochimici',
      title: 'Esami ematochimici',
      icon: 'clipboard',
      items: [{ id: 'esami_ematochimici', label: 'Esami ematochimici completati' }],
    },
    {
      id: 'radiologia',
      title: 'Esami Radiologici Necessari alla Procedura',
      icon: 'activity',
      items: [
        { id: 'rx_torace', label: 'Radiografia del torace' },
        { id: 'angio_tc', label: 'Angio-TC aorta toraco-addominale e assi iliaci' },
        { id: 'coronarografia_pre_tavi', label: 'Coronarografia/rivascolarizzazione pre-TAVI completata' },
      ],
    },
    {
      id: 'altre_valutazioni',
      title: 'Altre valutazioni',
      icon: 'stethoscope',
      items: [
        { id: 'heart_team', label: 'Valutazione Heart Team o consulenza cardiochirurgica eseguita' },
        { id: 'esami_heart_team', label: "Altri esami richiesti dall'Heart Team o comunque ritenuti necessari alla procedura" },
        { id: 'valutazione_anestesiologica', label: 'Valutazione anestesiologica' },
        { id: 'accertamenti_anestesista', label: "Accertamenti/prescrizioni richiesti dall'anestesista eseguiti" },
      ],
    },
    {
      id: 'preparazione_procedura',
      title: 'Preparazione del paziente in vista della procedura',
      icon: 'hospital',
      items: [
        { id: 'criticita_valutazioni', label: 'Criticità rilevate nella valutazione clinica e/o nella valutazione infermieristica' },
        { id: 'emoderivati', label: 'Richiesta/emogruppo ed emoderivati (se necessari)' },
        { id: 'consenso_informato', label: 'Consenso informato firmato' },
        { id: 'idratazione_mdc', label: 'Idratazione per somministrazione di mdc (per pazienti con eGFR<60 ml/min/m2)' },
        { id: 'profilassi_allergie_mdc', label: 'Profilassi per le allergie da mdc (ove indicato)' },
        { id: 'sospensione_anticoagulante', label: 'Sospensione adeguata terapia anticoagulante' },
        { id: 'profilassi_antibiotica', label: 'Prescrizione profilassi antibiotica' },
      ],
    },
  ];

  let statusSelection = '';
  let savingStatus = false;
  let checklistState = {};
  let lastLoadedChecklistId = null;
  let patientAge = null;
  let anagraficaForm = {
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
  let antropometricForm = {
    altezza: '',
    peso: '',
  };
  let savingAnagrafica = false;
  let savingAntropometrici = false;
  let ambulatorioForm = {
    fattori: [],
    anamnesi: '',
    apr: '',
    visita: '',
    conclusioni: '',
    medicoTitolo: 'Dott.',
    medicoNome: '',
  };
  let savingAmbulatorio = false;
  let schedaProceduraleForm = {
    peso: '',
    altezza: '',
    allergiaMdc: 'no',
    creatinina: '',
    egfr: '',
    hb: '',
    altro: '',
    ecgRitmoSinusale: false,
    ecgFA: false,
    ecgBBS: false,
    ecgBBD: false,
    ecgEAS: false,
    ecgBavPrimo: false,
    ecgRitmoStimolato: false,
    anestesia: '',
    coronarografia: '',
    coronarografiaNote: '',
    pacemaker: 'no',
    pacemakerNote: '',
    accessoPrincipaleFem: '',
    accessoPrincipaleAltro: '',
  accessoProtezione: 'no',
  accessoProtezioneNote: '',
  altriAccessi: '',
  diametroPalloneFemorale: '',
  guidaSafari: '',
  protezioneOsti: 'no',
  valvuloplastica: 'no',
  valvuloplasticaNote: '',
  bioprotesiModello: '',
  bioprotesiDimensione: '',
};
  let savingSchedaProcedurale = false;
  let generatingSchedaReferto = false;
  let generatingReferto = false;
  let deletingPatient = false;
  let showDeleteConfirm = false;
  const PLACE_DATA = loadPlaces();
  let placeData = PLACE_DATA;
  let showProvSuggestions = false;
  let showLuogoSuggestions = false;
  let anagraficaDateDisplay = '';
  let anagraficaDateError = '';

  function loadChecklistFromStorage(id) {
    const saved = localStorage.getItem(`patient-checklist-${id}`);
    if (!saved) {
      checklistState = {};
      return;
    }

    try {
      checklistState = JSON.parse(saved);
    } catch (e) {
      console.warn('Impossibile leggere checklist salvata', e);
      checklistState = {};
    }
  }

  function persistChecklist() {
    if (!patient?.patient?.id) return;
    localStorage.setItem(`patient-checklist-${patient.patient.id}`, JSON.stringify(checklistState));
  }

  function loadAmbulatorio() {
    if (!patient?.patient) {
      ambulatorioForm = { fattori: [], anamnesi: '', apr: '', visita: '', conclusioni: '', medicoTitolo: 'Dott.', medicoNome: '' };
      return;
    }

    ambulatorioForm = {
      fattori: patient.patient.ambulatorio_fattori || [],
      anamnesi: patient.patient.anamnesi_cardiologica || '',
      apr: patient.patient.apr || '',
      visita: patient.patient.visita_odierna || '',
      conclusioni: patient.patient.conclusioni || '',
      medicoTitolo: patient.patient.medico_titolo || 'Dott.',
      medicoNome: patient.patient.medico_nome || '',
    };
  }

  function loadSchedaProcedurale() {
    if (!patient?.patient) {
    schedaProceduraleForm = {
      peso: '',
      altezza: '',
      allergiaMdc: 'no',
      creatinina: '',
      egfr: '',
      hb: '',
      altro: '',
        ecgRitmoSinusale: false,
        ecgFA: false,
        ecgBBS: false,
        ecgBBD: false,
        ecgEAS: false,
        ecgBavPrimo: false,
        ecgRitmoStimolato: false,
        anestesia: '',
        coronarografia: '',
        coronarografiaNote: '',
        pacemaker: 'no',
        pacemakerNote: '',
        accessoPrincipaleFem: '',
        accessoPrincipaleAltro: '',
      accessoProtezione: 'no',
      accessoProtezioneNote: '',
      altriAccessi: '',
      diametroPalloneFemorale: '',
      guidaSafari: '',
      protezioneOsti: 'no',
      valvuloplastica: 'no',
      valvuloplasticaNote: '',
      bioprotesiModello: '',
      bioprotesiDimensione: '',
    };
      return;
    }

    schedaProceduraleForm = {
      peso: patient.patient.peso ?? '',
      altezza: patient.patient.altezza ?? '',
      allergiaMdc: patient.patient.procedurale_allergia_mdc || 'no',
      creatinina: patient.patient.procedurale_creatinina || '',
      egfr: patient.patient.procedurale_egfr || '',
      hb: patient.patient.procedurale_hb || '',
      altro: patient.patient.procedurale_altro || '',
      ecgRitmoSinusale: Boolean(patient.patient.procedurale_ecg_ritmo_sinusale),
      ecgFA: Boolean(patient.patient.procedurale_ecg_fa),
      ecgBBS: Boolean(patient.patient.procedurale_ecg_bbs),
      ecgBBD: Boolean(patient.patient.procedurale_ecg_bbd),
      ecgEAS: Boolean(patient.patient.procedurale_ecg_eas),
      ecgBavPrimo: Boolean(patient.patient.procedurale_ecg_bav_primo),
      ecgRitmoStimolato: Boolean(patient.patient.procedurale_ecg_ritmo_stimolato),
      anestesia: patient.patient.procedurale_anestesia || '',
      coronarografia: patient.patient.procedurale_coronarografia || '',
      coronarografiaNote: patient.patient.procedurale_coronarografia_note || '',
      pacemaker: patient.patient.procedurale_pacemaker || 'no',
      pacemakerNote: patient.patient.procedurale_pacemaker_note || '',
      accessoPrincipaleFem: patient.patient.procedurale_accesso_principale_fem || '',
      accessoPrincipaleAltro: patient.patient.procedurale_accesso_principale_altro || '',
      accessoProtezione: patient.patient.procedurale_accesso_protezione || 'no',
      accessoProtezioneNote: patient.patient.procedurale_accesso_protezione_note || '',
      altriAccessi: patient.patient.procedurale_altri_accessi || '',
      diametroPalloneFemorale: patient.patient.procedurale_diametro_pallone_femorale || '',
      guidaSafari: patient.patient.procedurale_guida_safari || '',
      protezioneOsti: patient.patient.procedurale_protezione_osti || 'no',
      valvuloplastica: patient.patient.procedurale_valvuloplastica || 'no',
      valvuloplasticaNote: patient.patient.procedurale_valvuloplastica_note || '',
      bioprotesiModello: patient.patient.procedurale_bioprotesi_modello || '',
      bioprotesiDimensione: patient.patient.procedurale_bioprotesi_dimensione || '',
    };
    handleValveModelChange(schedaProceduraleForm.bioprotesiModello);
  }

  function toggleItem(sectionId, itemId) {
    const sectionState = checklistState[sectionId] || {};
    checklistState = {
      ...checklistState,
      [sectionId]: { ...sectionState, [itemId]: !sectionState[itemId] },
    };
    persistChecklist();
  }

  function isChecked(sectionId, itemId) {
    return Boolean(checklistState[sectionId]?.[itemId]);
  }

  function sectionProgress(section) {
    const total = section.items.length;
    const done = section.items.filter(item => isChecked(section.id, item.id)).length;
    const percent = total === 0 ? 0 : Math.round((done / total) * 100);
    return { done, total, percent };
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

  $: totalChecklistItems = CHECKLIST_SECTIONS.reduce((acc, sec) => acc + sec.items.length, 0);
  $: totalChecklistDone = CHECKLIST_SECTIONS.reduce((acc, sec) => acc + sectionProgress(sec).done, 0);
  $: overallChecklistPercent =
    totalChecklistItems === 0 ? 0 : Math.round((totalChecklistDone / totalChecklistItems) * 100);
  $: patientAge = patient?.patient?.data_nascita ? calculateAge(patient.patient.data_nascita) : null;
  $: bmiValue = calculateBMI(
    normalizeNumber(antropometricForm.peso),
    normalizeNumber(antropometricForm.altezza)
  );
  $: bsaValue = calculateBSA(
    normalizeNumber(antropometricForm.peso),
    normalizeNumber(antropometricForm.altezza)
  );
  $: bmiCategory = categorizeBMI(bmiValue);
  $: bioprotesiSizeOptions = getValveSizes(schedaProceduraleForm.bioprotesiModello);

  async function handleStatusChange() {
    if (!patient?.patient?.id) return;
    if (statusSelection === patient.status) return;

    savingStatus = true;
    try {
      await changePatientStatus(patient.patient.id, statusSelection);
      await loadPatient(patient.patient.id);
      showToast('Stato paziente aggiornato', 'success');
    } catch (e) {
      console.error(e);
      showToast('Errore durante l\'aggiornamento dello stato', 'error');
    } finally {
      savingStatus = false;
    }
  }

  const normalizeText = (value) => {
    if (value === undefined || value === null) return null;
    const v = String(value).trim();
    return v === '' ? null : v;
  };

  const normalizeNumber = (value) => {
    if (value === undefined || value === null || value === '') return null;
    const num = Number(value);
    return Number.isFinite(num) ? num : null;
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

  function digitsToIsoAndError(digits) {
    const day = `${digits[0] || ''}${digits[1] || ''}`;
    const month = `${digits[2] || ''}${digits[3] || ''}`;
    const year = `${digits[4] || ''}${digits[5] || ''}${digits[6] || ''}${digits[7] || ''}`;
    if (day.length < 2 || month.length < 2 || year.length < 4) {
      return { iso: '', error: '' };
    }
    const numeric = formatDateInput(`${day}${month}${year}`);
    return { iso: numeric.iso, error: numeric.error };
  }

  function handleDetailDateInput(value) {
    const cleaned = (value || '').replace(/\D/g, '').slice(0, 8);
    const display = formatDisplayFromDigits(cleaned);
    const { iso, error } = formatDateInput(cleaned);
    anagraficaDateDisplay = display;
    anagraficaForm.data_nascita = iso;
    anagraficaDateError = error;
    updatePatientCF();
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
      const map = (i + 1) % 2 === 0 ? evenMap : oddMap;
      sum += map[ch] ?? 0;
    }
    const checkChar = String.fromCharCode('A'.charCodeAt(0) + (sum % 26));
    return `${partial}${checkChar}`;
  }

  function updatePatientCF() {
    const cf = calcCodiceFiscale(anagraficaForm);
    if (cf) {
      anagraficaForm.codice_fiscale = cf;
    }
  }

  // Load place data once (sync import)
  placeData = loadPlaces();

  async function saveAnagrafica() {
    if (!patient?.patient?.id) return;
    savingAnagrafica = true;

    if (!anagraficaForm.data_nascita || anagraficaDateError) {
      showToast('Data di nascita non valida', 'warning');
      savingAnagrafica = false;
      return;
    }

    const payload = {
      ...patient.patient,
      nome: capitalizeWordsStrict(anagraficaForm.nome),
      cognome: capitalizeWordsStrict(anagraficaForm.cognome),
      data_nascita: anagraficaForm.data_nascita,
      luogo_nascita: normalizeText(anagraficaForm.luogo_nascita),
      codice_fiscale: normalizeText(anagraficaForm.codice_fiscale),
      telefono: normalizeText(anagraficaForm.telefono),
      email: normalizeText(anagraficaForm.email),
      provenienza: normalizeText(anagraficaForm.provenienza),
      sesso: normalizeText(anagraficaForm.sesso),
      altezza: normalizeNumber(antropometricForm.altezza),
      peso: normalizeNumber(antropometricForm.peso),
    };

    try {
      await updatePatient(payload);
      await loadPatient(patient.patient.id);
      showToast('Anagrafica aggiornata', 'success');
    } catch (e) {
      console.error(e);
      showToast('Errore durante il salvataggio', 'error');
    } finally {
      savingAnagrafica = false;
    }
  }

  function openDeleteConfirm() {
    if (!patient?.patient?.id) return;
    showDeleteConfirm = true;
  }

  async function confirmDelete() {
    if (!patient?.patient?.id) return;
    deletingPatient = true;
    try {
      await deletePatient(patient.patient.id);
      showToast('Paziente eliminato', 'success');
      onBack();
    } catch (e) {
      console.error(e);
      showToast('Errore durante l\'eliminazione', 'error');
    } finally {
      deletingPatient = false;
      showDeleteConfirm = false;
    }
  }

  function cancelDelete() {
    showDeleteConfirm = false;
  }

  async function saveAntropometrici() {
    if (!patient?.patient?.id) return;
    savingAntropometrici = true;

    const payload = {
      ...patient.patient,
      altezza: normalizeNumber(antropometricForm.altezza),
      peso: normalizeNumber(antropometricForm.peso),
    };

    try {
      await updatePatient(payload);
      await loadPatient(patient.patient.id);
      showToast('Dati antropometrici aggiornati', 'success');
    } catch (e) {
      console.error(e);
      showToast('Errore durante il salvataggio', 'error');
    } finally {
      savingAntropometrici = false;
    }
  }

  async function saveAmbulatorio() {
    if (!patient?.patient?.id) return false;
    savingAmbulatorio = true;

    const payload = {
      ...patient.patient,
      ambulatorio_fattori: ambulatorioForm.fattori,
      anamnesi_cardiologica: normalizeText(ambulatorioForm.anamnesi),
      apr: normalizeText(ambulatorioForm.apr),
      visita_odierna: normalizeText(ambulatorioForm.visita),
      conclusioni: normalizeText(ambulatorioForm.conclusioni),
      medico_titolo: normalizeText(ambulatorioForm.medicoTitolo),
      medico_nome: normalizeText(ambulatorioForm.medicoNome),
    };

    try {
      await updatePatient(payload);
      await loadPatient(patient.patient.id);
      showToast('Ambulatorio strutturale salvato', 'success');
      return true;
    } catch (e) {
      console.error(e);
      showToast('Errore durante il salvataggio', 'error');
      return false;
    } finally {
      savingAmbulatorio = false;
    }
  }

  async function saveSchedaProcedurale() {
    if (!patient?.patient?.id) return;
    savingSchedaProcedurale = true;

    const payload = {
      ...patient.patient,
      peso: normalizeNumber(schedaProceduraleForm.peso),
      altezza: normalizeNumber(schedaProceduraleForm.altezza),
      procedurale_allergia_mdc: schedaProceduraleForm.allergiaMdc,
      procedurale_creatinina: normalizeText(schedaProceduraleForm.creatinina),
      procedurale_egfr: normalizeText(schedaProceduraleForm.egfr),
      procedurale_hb: normalizeText(schedaProceduraleForm.hb),
      procedurale_altro: normalizeText(schedaProceduraleForm.altro),
      procedurale_ecg_ritmo_sinusale: schedaProceduraleForm.ecgRitmoSinusale,
      procedurale_ecg_fa: schedaProceduraleForm.ecgFA,
      procedurale_ecg_bbs: schedaProceduraleForm.ecgBBS,
      procedurale_ecg_bbd: schedaProceduraleForm.ecgBBD,
      procedurale_ecg_eas: schedaProceduraleForm.ecgEAS,
      procedurale_ecg_bav_primo: schedaProceduraleForm.ecgBavPrimo,
      procedurale_ecg_ritmo_stimolato: schedaProceduraleForm.ecgRitmoStimolato,
      procedurale_anestesia: schedaProceduraleForm.anestesia,
      procedurale_coronarografia: schedaProceduraleForm.coronarografia,
      procedurale_coronarografia_note: normalizeText(schedaProceduraleForm.coronarografiaNote),
      procedurale_pacemaker: schedaProceduraleForm.pacemaker,
      procedurale_pacemaker_note: normalizeText(schedaProceduraleForm.pacemakerNote),
      procedurale_accesso_principale_fem: normalizeText(schedaProceduraleForm.accessoPrincipaleFem),
      procedurale_accesso_principale_altro:
        schedaProceduraleForm.accessoPrincipaleFem === 'altro'
          ? normalizeText(schedaProceduraleForm.accessoPrincipaleAltro)
          : null,
      procedurale_accesso_protezione: schedaProceduraleForm.accessoProtezione,
      procedurale_accesso_protezione_note: normalizeText(schedaProceduraleForm.accessoProtezioneNote),
      procedurale_altri_accessi: normalizeText(schedaProceduraleForm.altriAccessi),
      procedurale_diametro_pallone_femorale: normalizeText(schedaProceduraleForm.diametroPalloneFemorale),
      procedurale_guida_safari: normalizeText(schedaProceduraleForm.guidaSafari),
      procedurale_protezione_osti: normalizeText(schedaProceduraleForm.protezioneOsti),
      procedurale_valvuloplastica: schedaProceduraleForm.valvuloplastica,
      procedurale_valvuloplastica_note: normalizeText(schedaProceduraleForm.valvuloplasticaNote),
      procedurale_bioprotesi_modello: normalizeText(schedaProceduraleForm.bioprotesiModello),
      procedurale_bioprotesi_dimensione: normalizeText(schedaProceduraleForm.bioprotesiDimensione),
    };

    try {
      await updatePatient(payload);
      await loadPatient(patient.patient.id);
      showToast('Scheda procedurale salvata', 'success');
    } catch (e) {
      console.error(e);
      showToast('Errore durante il salvataggio della scheda procedurale', 'error');
    } finally {
      savingSchedaProcedurale = false;
    }
  }

  async function generateSchedaProceduraleReferto() {
    if (!patient?.patient?.id) return;
    generatingSchedaReferto = true;
    try {
      // Salva prima la scheda procedurale per avere dati aggiornati
      await saveSchedaProcedurale();
      const result = await invoke('generate_scheda_procedurale_referto', {
        patientId: patient.patient.id
      });
      const path = typeof result === 'string' ? result : '';
      if (path) {
        showToast(`Referto generato: ${path}`, 'success');
        await maybeOpenReferto(path);
      } else {
        showToast('Referto generato', 'success');
      }
    } catch (e) {
      console.error('Errore referto procedurale', e);
      const msg = typeof e === 'string' ? e : e?.message || 'Errore durante la generazione del referto';
      showToast(msg, 'error');
    } finally {
      generatingSchedaReferto = false;
    }
  }

  async function generateReferto() {
    if (!patient?.patient?.id) return;
    generatingReferto = true;
    try {
      // Save latest ambulatorio data before generating
      const saved = await saveAmbulatorio();
      if (!saved) {
        generatingReferto = false;
        return;
      }
      const result = await invoke('generate_ambulatorio_referto', {
        patientId: patient.patient.id
      });
      const path = typeof result === 'string' ? result : '';
      if (path) {
        showToast(`Referto generato: ${path}`, 'success');
        await maybeOpenReferto(path);
      } else {
        showToast('Referto generato', 'success');
      }
    } catch (e) {
      console.error('Errore referto', e);
      const msg = typeof e === 'string' ? e : e?.message || 'Errore durante la generazione del referto';
      showToast(msg, 'error');
    } finally {
      generatingReferto = false;
    }
  }

  async function shouldAutoOpenReferto() {
    try {
      const stored = localStorage.getItem('tavi_settings');
      if (stored) {
        const parsed = JSON.parse(stored);
        if (typeof parsed?.autoOpenReferti === 'boolean') {
          return parsed.autoOpenReferti;
        }
      }
    } catch (e) {
      // ignore
    }

    try {
      const loaded = await invoke('load_settings');
      if (loaded && typeof loaded === 'object' && typeof loaded.autoOpenReferti === 'boolean') {
        return loaded.autoOpenReferti;
      }
    } catch (e) {
      // ignore
    }

    return true; // default: apri il referto
  }

  async function maybeOpenReferto(path) {
    if (!path) return;
    const shouldOpen = await shouldAutoOpenReferto();
    if (!shouldOpen) return;
    try {
      await openExternal(path);
    } catch (e) {
      console.error('Errore apertura referto', e);
      showToast('Errore apertura referto', 'error');
    }
  }

  $: if (patient?.patient?.id && patient.patient.id !== lastLoadedChecklistId) {
    statusSelection = patient.status;
    loadChecklistFromStorage(patient.patient.id);
    loadAmbulatorio();
    loadSchedaProcedurale();
    anagraficaForm = {
      nome: capitalizeWordsStrict(patient.patient.nome || ''),
      cognome: capitalizeWordsStrict(patient.patient.cognome || ''),
      data_nascita: patient.patient.data_nascita || '',
      sesso: patient.patient.sesso || 'M',
      luogo_nascita: patient.patient.luogo_nascita || '',
      luogo_nascita_codice: '',
      codice_fiscale: patient.patient.codice_fiscale || '',
      telefono: patient.patient.telefono || '',
      email: patient.patient.email || '',
      provenienza: patient.patient.provenienza || '',
    };
    antropometricForm = {
      altezza: patient.patient.altezza ?? '',
      peso: patient.patient.peso ?? '',
    };
    ambulatorioForm = {
      fattori: patient.patient.ambulatorio_fattori || [],
      anamnesi: patient.patient.anamnesi_cardiologica || '',
      apr: patient.patient.apr || '',
      visita: patient.patient.visita_odierna || '',
      conclusioni: patient.patient.conclusioni || '',
      medicoTitolo: patient.patient.medico_titolo || 'Dott.',
      medicoNome: patient.patient.medico_nome || '',
    };
    anagraficaDateDisplay = formatDisplayFromDigits((patient.patient.data_nascita || '').replace(/-/g, ''));
    anagraficaDateError = '';
    lastLoadedChecklistId = patient.patient.id;
  }
</script>

{#if loading || !patient}
  <Card padding="lg">
    <div class="flex items-center gap-3">
      <div class="inline-block animate-spin rounded-full h-10 w-10 border-b-2 border-primary"></div>
      <div>
        <p class="text-textPrimary font-semibold">Caricamento paziente...</p>
        <p class="text-textSecondary text-sm">Recupero dati e checklist</p>
      </div>
    </div>
  </Card>
{:else if !patient}
  <Card padding="lg">
    <div class="flex items-center gap-3 text-textSecondary">
      <IconBadge icon="info" tone="secondary" />
      <div>
        <p class="font-semibold text-textPrimary">Nessun paziente selezionato</p>
        <p class="text-sm">Torna alla lista e scegli un paziente per visualizzare la checklist.</p>
      </div>
    </div>
  </Card>
{:else}
  <div class="space-y-6">
    <div class="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
      <div class="flex items-start gap-3">
        <IconBadge icon="user" size="lg" tone="primary" class="mt-1" />
        <div>
          <p class="text-sm text-textSecondary">Paziente</p>
          <h2 class="text-3xl font-bold text-textPrimary">
            {patient.patient.nome} {patient.patient.cognome}
          </h2>
          <p class="text-textSecondary mt-1">
            {patientAge ?? '-'} anni • Nato il {formatDateIT(patient.patient.data_nascita)}
          </p>
          <p class="text-xs text-textSecondary mt-1">
            Inserito il {formatDateIT(patient.patient.created_at?.split(' ')[0])}
          </p>
          <div class="mt-2 inline-flex items-center gap-2 px-3 py-1 rounded-full bg-primary/10 text-primary text-sm">
            <span class="font-medium">{patient.status}</span>
          </div>
        </div>
      </div>

      <div class="w-full max-w-xs space-y-2">
        <Select
          label="Aggiorna stato"
          bind:value={statusSelection}
          options={STATUS_OPTIONS}
          placeholder="Seleziona stato"
        />
        <Button
          variant="primary"
          fullWidth
          disabled={savingStatus || statusSelection === patient.status}
          on:click={handleStatusChange}
        >
          {savingStatus ? 'Aggiornamento...' : 'Salva nuovo stato'}
        </Button>
        <p class="text-xs text-textSecondary">
          Ultimo aggiornamento: {formatDateIT(patient.status_created_at?.split(' ')[0])}
        </p>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <div class="space-y-4 lg:sticky lg:top-4 lg:max-h-[calc(100vh-140px)] lg:overflow-y-auto lg:pr-2 overscroll-contain">
        <Card padding="lg">
          <div class="flex items-center justify-between gap-3 mb-4 flex-wrap">
            <div class="flex items-center gap-2">
              <IconBadge icon="check" tone="primary" />
              <h3 class="text-lg font-semibold text-textPrimary">Checklist paziente</h3>
            </div>
          </div>

          <div class="grid grid-cols-1 gap-4">
            {#each CHECKLIST_SECTIONS as section}
              {@const progress = sectionProgress(section)}
              {#key section.id}
                <div class="p-4 border rounded-lg border-gray-200">
                  <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center gap-2">
                      <IconBadge icon={section.icon} tone="secondary" />
                      <div>
                        <h4 class="text-base font-semibold text-textPrimary">{section.title}</h4>
                      </div>
                    </div>
                    <div class="text-right">
                      <p class="text-xs text-textSecondary">
                        {progress.done}/{progress.total} completati
                      </p>
                      <div class="h-1.5 w-24 bg-gray-100 rounded-full overflow-hidden mt-1">
                        <div
                          class="h-full bg-primary"
                          style={`width: ${progress.percent}%`}
                        />
                      </div>
                    </div>
                  </div>

                  <div class="space-y-2">
                    {#each section.items as item}
                      <div class="flex items-start gap-2 py-1">
                        <Checkbox
                          checked={isChecked(section.id, item.id)}
                          on:change={() => toggleItem(section.id, item.id)}
                        />
                        <div>
                          <p class="text-sm text-textPrimary">{item.label}</p>
                          {#if patient.status === 'TAVI eseguita' && section.id === 'post_tavi' && item.id === 'followup'}
                            <p class="text-xs text-textSecondary">Assicurarsi che il follow-up sia pianificato prima delle dimissioni.</p>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/key}
            {/each}
          </div>
        </Card>
      </div>

      <div class="lg:col-span-2 space-y-4">
        <SectionPanel title="Dati anagrafici" icon="clipboard" collapsed={false}>
          <div class="space-y-4">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <Input
                label="Nome"
                required
                bind:value={anagraficaForm.nome}
                on:input={updatePatientCF}
                on:blur={() => {
                  anagraficaForm.nome = capitalizeWordsStrict(anagraficaForm.nome);
                  updatePatientCF();
                }}
              />
              <Input
                label="Cognome"
                required
                bind:value={anagraficaForm.cognome}
                on:input={updatePatientCF}
                on:blur={() => {
                  anagraficaForm.cognome = capitalizeWordsStrict(anagraficaForm.cognome);
                  updatePatientCF();
                }}
              />
              <div>
                <label class="block text-sm font-medium text-textPrimary mb-1">
                  Data di nascita <span class="text-error">*</span>
                </label>
                <Input
                  placeholder="gg/mm/aaaa"
                  bind:value={anagraficaDateDisplay}
                  on:input={(e) => handleDetailDateInput(e.detail?.target?.value || '')}
                  error={anagraficaDateError}
                />
              </div>
              <div class="relative">
                <Input
                  label="Luogo di nascita"
                  bind:value={anagraficaForm.luogo_nascita}
                  on:focus={() => (showLuogoSuggestions = true)}
                  on:input={() => (showLuogoSuggestions = true)}
                  on:blur={() => {
                    setTimeout(() => (showLuogoSuggestions = false), 120);
                    updatePatientCF();
                  }}
                />
                {#if showLuogoSuggestions && filterPlaces(anagraficaForm.luogo_nascita).length}
                  <div class="absolute z-30 mt-1 w-full bg-white border border-gray-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
                    {#each filterPlaces(anagraficaForm.luogo_nascita) as suggestion}
                      <button
                        type="button"
                        class="w-full text-left px-3 py-2 text-sm hover:bg-gray-50"
                        on:click={() => {
                          anagraficaForm.luogo_nascita = suggestion.label;
                          anagraficaForm.luogo_nascita_codice = suggestion.codice || '';
                          showLuogoSuggestions = false;
                          updatePatientCF();
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
                      checked={anagraficaForm.sesso === 'M'}
                      on:change={() => {
                        anagraficaForm.sesso = 'M';
                        updatePatientCF();
                      }}
                    />
                    Maschio
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={anagraficaForm.sesso === 'F'}
                      on:change={() => {
                        anagraficaForm.sesso = 'F';
                        updatePatientCF();
                      }}
                    />
                    Femmina
                  </label>
                </div>
              </div>
              <Input label="Codice fiscale" bind:value={anagraficaForm.codice_fiscale} />
              <Input label="Telefono" type="tel" bind:value={anagraficaForm.telefono} />
              <Input label="Email" type="email" bind:value={anagraficaForm.email} />
              <div class="relative">
                <Input
                  label="Provenienza (cardiologo/ospedale)"
                  placeholder="Cardiologo o ospedale di provenienza"
                  bind:value={anagraficaForm.provenienza}
                  on:focus={() => (showProvSuggestions = true)}
                  on:input={() => (showProvSuggestions = true)}
                  on:blur={() => setTimeout(() => (showProvSuggestions = false), 120)}
                />
                {#if showProvSuggestions && filterPlaces(anagraficaForm.provenienza).length}
                  <div class="absolute z-30 mt-1 w-full bg-white border border-gray-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
                    {#each filterPlaces(anagraficaForm.provenienza) as suggestion}
                      <button
                        type="button"
                        class="w-full text-left px-3 py-2 text-sm hover:bg-gray-50"
                        on:click={() => {
                          anagraficaForm.provenienza = suggestion.label;
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
            <div class="flex justify-end gap-2">
              <Button variant="danger" size="sm" on:click={openDeleteConfirm} disabled={deletingPatient}>
                {deletingPatient ? 'Eliminazione...' : 'Elimina paziente'}
              </Button>
              <Button variant="primary" size="sm" on:click={saveAnagrafica} disabled={savingAnagrafica}>
                {savingAnagrafica ? 'Salvataggio...' : 'Salva modifiche'}
              </Button>
            </div>
          </div>
        </SectionPanel>

        <SectionPanel title="Dati antropometrici" icon="activity" collapsed={true}>
          <div class="space-y-4">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <Input
                label="Altezza (cm)"
                type="number"
                min="0"
                bind:value={antropometricForm.altezza}
              />
              <Input
                label="Peso (kg)"
                type="number"
                min="0"
                bind:value={antropometricForm.peso}
              />
              <div class="sm:col-span-2 flex items-center gap-3 p-3 rounded-lg bg-gray-50 text-sm">
                <IconBadge icon="heart" tone="secondary" />
                <div class="flex flex-col gap-1">
                  <div class="flex items-center gap-2">
                    <p class="text-textSecondary">BMI</p>
                    <span class="font-semibold text-textPrimary">{bmiValue ? formatNumberIT(bmiValue, 1) : '-'}</span>
                    <span class="px-2 py-0.5 rounded-full text-xs bg-gray-200 text-textPrimary">
                      {bmiCategory}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <p class="text-textSecondary">BSA (Mosteller)</p>
                    <span class="font-semibold text-textPrimary">{bsaValue ? `${bsaValue} m²` : '-'}</span>
                  </div>
                </div>
              </div>
            </div>
            <div class="flex justify-end">
              <Button variant="primary" size="sm" on:click={saveAntropometrici} disabled={savingAntropometrici}>
                {savingAntropometrici ? 'Salvataggio...' : 'Salva modifiche'}
              </Button>
            </div>
          </div>
        </SectionPanel>

        <SectionPanel title="Ambulatorio strutturale" icon="clipboard" collapsed={true}>
          <div class="space-y-4">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-textPrimary mb-1">Titolo medico</label>
                <select
                  class="w-full px-3 py-2 border rounded-lg border-gray-200 focus:ring-primary/20 focus:border-primary"
                  bind:value={ambulatorioForm.medicoTitolo}
                >
                  <option value="Dott.">Dott.</option>
                  <option value="Dott.ssa">Dott.ssa</option>
                </select>
              </div>
              <Input
                label="Cardiologo referente"
                placeholder="Nome e cognome"
                bind:value={ambulatorioForm.medicoNome}
              />
            </div>
            <CVRiskFactors label="Fattori di rischio CV" bind:selectedFactors={ambulatorioForm.fattori} />
            <RichTextArea
              label="Anamnesi cardiologica"
              placeholder="Inserisci l'anamnesi cardiologica..."
              bind:value={ambulatorioForm.anamnesi}
            />
            <RichTextArea
              label="APR"
              placeholder="Note APR..."
              bind:value={ambulatorioForm.apr}
            />
            <RichTextArea
              label="Visita odierna"
              placeholder="Esame obiettivo, sintomi, parametri rilevati..."
              bind:value={ambulatorioForm.visita}
            />
            <RichTextArea
              label="Conclusioni"
              placeholder="Decisioni, piano terapeutico, follow-up..."
              bind:value={ambulatorioForm.conclusioni}
            />
            <div class="flex flex-wrap gap-3 justify-end">
              <Button variant="primary" size="sm" on:click={saveAmbulatorio} disabled={savingAmbulatorio || generatingReferto}>
                {savingAmbulatorio ? 'Salvataggio...' : 'Salva sezione'}
              </Button>
              <Button variant="secondary" size="sm" on:click={generateReferto} disabled={savingAmbulatorio || generatingReferto}>
                {generatingReferto ? 'Generazione...' : 'Salva e genera referto'}
              </Button>
            </div>
          </div>
        </SectionPanel>

        <SectionPanel title="Scheda procedurale" icon="activity" collapsed={true}>
          <div class="space-y-4">
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <Input label="Peso (Kg)" type="number" min="0" bind:value={schedaProceduraleForm.peso} />
              <Input label="Altezza (cm)" type="number" min="0" bind:value={schedaProceduraleForm.altezza} />
              <div class="flex flex-col justify-center">
                <p class="text-sm font-semibold text-textPrimary mb-2">Allergia al mdc</p>
                <div class="flex items-center gap-4">
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.allergiaMdc === 'si'}
                      on:change={() => (schedaProceduraleForm.allergiaMdc = schedaProceduraleForm.allergiaMdc === 'si' ? '' : 'si')}
                    />
                    Sì
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.allergiaMdc === 'no'}
                      on:change={() => (schedaProceduraleForm.allergiaMdc = schedaProceduraleForm.allergiaMdc === 'no' ? '' : 'no')}
                    />
                    No
                  </label>
                </div>
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <Input label="Creatinina (mg/dL)" bind:value={schedaProceduraleForm.creatinina} />
              <Input label="eGFR (ml/min)" bind:value={schedaProceduraleForm.egfr} />
              <Input label="Hb (g/dL)" bind:value={schedaProceduraleForm.hb} />
            </div>

            <RichTextArea
              label="Altro"
              placeholder="Note aggiuntive..."
              bind:value={schedaProceduraleForm.altro}
              minHeight="90px"
            />

            <div class="space-y-2">
              <p class="text-sm font-semibold text-textPrimary">ECG basale</p>
              <div class="grid grid-cols-1 sm:grid-cols-3 md:grid-cols-4 gap-3">
                {#each ECG_OPTIONS as opt}
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm[opt.id]}
                      on:change={() => (schedaProceduraleForm[opt.id] = !schedaProceduraleForm[opt.id])}
                    />
                    {opt.label}
                  </label>
                {/each}
              </div>
            </div>

            <div class="space-y-2">
              <p class="text-sm font-semibold text-textPrimary">Anestesia</p>
              <div class="flex flex-wrap items-center gap-4">
                {#each ANESTESIA_OPTIONS as opt}
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.anestesia === opt}
                      on:change={() => (schedaProceduraleForm.anestesia = schedaProceduraleForm.anestesia === opt ? '' : opt)}
                    />
                    {opt}
                  </label>
                {/each}
              </div>
            </div>

            <div class="space-y-2">
              <p class="text-sm font-semibold text-textPrimary">Coronarografia</p>
                <div class="flex flex-col gap-2">
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.coronarografia === 'ricovero'}
                      on:change={() => (schedaProceduraleForm.coronarografia = schedaProceduraleForm.coronarografia === 'ricovero' ? '' : 'ricovero')}
                  />
                  Da eseguire durante ricovero per TAVI
                </label>
                <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.coronarografia === 'gia_eseguita'}
                    on:change={() => (schedaProceduraleForm.coronarografia = schedaProceduraleForm.coronarografia === 'gia_eseguita' ? '' : 'gia_eseguita')}
                  />
                  Già eseguita
                </label>
              </div>
              {#if schedaProceduraleForm.coronarografia === 'gia_eseguita'}
                <Input
                  label="Dettagli coronarografia"
                  placeholder="Note / data"
                  bind:value={schedaProceduraleForm.coronarografiaNote}
                />
              {/if}
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-semibold text-textPrimary mb-1">Pacemaker definitivo</label>
                <div class="flex flex-col gap-2">
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.pacemaker === 'si'}
                      on:change={() => (schedaProceduraleForm.pacemaker = schedaProceduraleForm.pacemaker === 'si' ? '' : 'si')}
                    />
                    Sì
                    {#if schedaProceduraleForm.pacemaker === 'si'}
                      <input
                        class="ml-2 px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary"
                        placeholder="Dispositivo / note"
                        bind:value={schedaProceduraleForm.pacemakerNote}
                      />
                    {/if}
                  </label>
                <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.pacemaker === 'no'}
                    on:change={() => {
                      schedaProceduraleForm.pacemaker =
                        schedaProceduraleForm.pacemaker === 'no' ? '' : 'no';
                      schedaProceduraleForm.pacemakerNote = '';
                    }}
                  />
                  No
                </label>
                </div>
              </div>
            </div>

            <div class={`grid grid-cols-1 gap-4 ${schedaProceduraleForm.accessoPrincipaleFem === 'altro' ? 'sm:grid-cols-2' : 'sm:grid-cols-1'}`}>
              <div>
                <p class="text-sm font-semibold text-textPrimary mb-2">Accesso principale</p>
                <div class="grid grid-cols-2 gap-3">
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.accessoPrincipaleFem === 'percutaneo_dx'}
                      on:change={() => setAccessoPrincipale('percutaneo_dx')}
                    />
                    Percutaneo femorale destro
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.accessoPrincipaleFem === 'percutaneo_sn'}
                      on:change={() => setAccessoPrincipale('percutaneo_sn')}
                    />
                    Percutaneo femorale sinistro
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.accessoPrincipaleFem === 'chirurgico_dx'}
                      on:change={() => setAccessoPrincipale('chirurgico_dx')}
                    />
                    Chirurgico femorale destro
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.accessoPrincipaleFem === 'chirurgico_sn'}
                      on:change={() => setAccessoPrincipale('chirurgico_sn')}
                    />
                    Chirurgico femorale sinistro
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                    <Checkbox
                      checked={schedaProceduraleForm.accessoPrincipaleFem === 'altro'}
                      on:change={() => setAccessoPrincipale('altro')}
                    />
                    Altro
                  </label>
                </div>
              </div>
              {#if schedaProceduraleForm.accessoPrincipaleFem === 'altro'}
                <RichTextArea
                  label="Note accesso principale"
                  placeholder="Ulteriori dettagli di accesso"
                  bind:value={schedaProceduraleForm.accessoPrincipaleAltro}
                />
              {/if}
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-semibold text-textPrimary mb-1">Accesso arterioso di protezione</label>
                <div class="flex flex-col gap-2">
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.accessoProtezione === 'si'}
                    on:change={() => (schedaProceduraleForm.accessoProtezione = schedaProceduraleForm.accessoProtezione === 'si' ? '' : 'si')}
                  />
                  Sì
                  {#if schedaProceduraleForm.accessoProtezione === 'si'}
                    <input
                      class="ml-2 px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary"
                        placeholder="Specificare sede e Fr"
                        bind:value={schedaProceduraleForm.accessoProtezioneNote}
                      />
                    {/if}
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.accessoProtezione === 'no'}
                    on:change={() => {
                      schedaProceduraleForm.accessoProtezione =
                        schedaProceduraleForm.accessoProtezione === 'no' ? '' : 'no';
                      schedaProceduraleForm.accessoProtezioneNote = '';
                    }}
                  />
                  No
                </label>
                </div>
              </div>
              <div>
                <label class="block text-sm font-semibold text-textPrimary mb-1">Necessità di protezione osti coronarici</label>
                <div class="flex flex-col gap-2">
                  <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.protezioneOsti === 'si'}
                    on:change={() => (schedaProceduraleForm.protezioneOsti = schedaProceduraleForm.protezioneOsti === 'si' ? '' : 'si')}
                  />
                  Sì
                </label>
                <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.protezioneOsti === 'no'}
                    on:change={() => (schedaProceduraleForm.protezioneOsti = schedaProceduraleForm.protezioneOsti === 'no' ? '' : 'no')}
                  />
                  No
                </label>
              </div>
            </div>
            </div>

            <RichTextArea
              label="Altri accessi necessari"
              placeholder="Descrivi altri accessi"
              bind:value={schedaProceduraleForm.altriAccessi}
            />

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <Input
                label="Diametro pallone per femorale"
                placeholder="Es. 12 Fr"
                bind:value={schedaProceduraleForm.diametroPalloneFemorale}
              />
              <Input
                label="Guida Safari (small/extrasmall)"
                placeholder="Specificare"
                bind:value={schedaProceduraleForm.guidaSafari}
              />
            </div>

            <div class="space-y-2">
              <p class="text-sm font-semibold text-textPrimary mb-2">Valvuloplastica pre-impianto</p>
              <div class="flex flex-col gap-2">
                <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.valvuloplastica === 'si'}
                    on:change={() => (schedaProceduraleForm.valvuloplastica = schedaProceduraleForm.valvuloplastica === 'si' ? '' : 'si')}
                  />
                  <span>Sì</span>
                  {#if schedaProceduraleForm.valvuloplastica === 'si'}
                    <input
                      class="ml-2 px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary"
                      placeholder="Diametro pallone"
                      bind:value={schedaProceduraleForm.valvuloplasticaNote}
                    />
                  {/if}
                </label>
                <label class="inline-flex items-center gap-2 text-sm text-textPrimary">
                  <Checkbox
                    checked={schedaProceduraleForm.valvuloplastica === 'no'}
                    on:change={() => {
                      const next = schedaProceduraleForm.valvuloplastica === 'no' ? '' : 'no';
                      schedaProceduraleForm.valvuloplastica = next;
                      if (next !== 'si') schedaProceduraleForm.valvuloplasticaNote = '';
                    }}
                  />
                  No
                </label>
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <Select
                label="Bioprotesi - modello valvola"
                bind:value={schedaProceduraleForm.bioprotesiModello}
                on:change={(e) => handleValveModelChange(e.detail?.value ?? e.target?.value)}
                options={[{ value: '', label: 'Seleziona modello' }, ...VALVOLE_TAVI]}
              />
              <Select
                label="Bioprotesi - dimensione"
                bind:value={schedaProceduraleForm.bioprotesiDimensione}
                options={[{ value: '', label: 'Seleziona dimensione' }, ...bioprotesiSizeOptions.map(size => ({ value: size, label: size }))]}
                disabled={!schedaProceduraleForm.bioprotesiModello}
              />
            </div>

            <div class="flex flex-wrap gap-3 justify-end">
              <Button variant="primary" size="sm" on:click={saveSchedaProcedurale} disabled={savingSchedaProcedurale || generatingSchedaReferto}>
                {savingSchedaProcedurale ? 'Salvataggio...' : 'Salva scheda procedurale'}
              </Button>
              <Button variant="secondary" size="sm" on:click={generateSchedaProceduraleReferto} disabled={savingSchedaProcedurale || generatingSchedaReferto}>
                {generatingSchedaReferto ? 'Generazione...' : 'Salva e genera referto'}
              </Button>
            </div>
          </div>
        </SectionPanel>
      </div>
    </div>

    {#if showDeleteConfirm}
      <div class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4" on:click={cancelDelete}>
        <div class="max-w-md w-full" on:click|stopPropagation>
          <Card padding="lg" class="bg-white">
            <div class="flex items-start gap-3">
              <IconBadge icon="alert" tone="error" />
              <div>
                <h3 class="text-lg font-semibold text-textPrimary">Eliminare il paziente?</h3>
                <p class="text-sm text-textSecondary mt-1">
                  Questa azione rimuoverà il paziente e i suoi dati. Confermi l'eliminazione?
                </p>
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-6">
              <Button variant="text" size="sm" on:click={cancelDelete}>Annulla</Button>
              <Button variant="danger" size="sm" on:click={confirmDelete} disabled={deletingPatient}>
                {deletingPatient ? 'Eliminazione...' : 'Elimina'}
              </Button>
            </div>
          </Card>
        </div>
      </div>
    {/if}

  </div>
{/if}
