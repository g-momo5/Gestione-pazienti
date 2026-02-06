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
  import { tick } from 'svelte';
  import { calculateBMI, calculateBSA, categorizeBMI, formatNumberIT, calculateEgfrCkdEpi } from '../utils/statistics.js';
  import { changePatientStatus, loadPatient, updatePatient, deletePatient } from '../stores/patientStore.js';
  import { showToast } from '../stores/toastStore.js';
  import { loadPlaces } from '../utils/placeSuggestions.js';
  import { invoke } from '@tauri-apps/api/tauri';
  import { readBinaryFile } from '@tauri-apps/api/fs';
  import { open as openExternal } from '@tauri-apps/api/shell';
  import { renderAsync } from 'docx-preview';
  import * as pdfjsLib from 'pdfjs-dist/legacy/build/pdf';
  import pdfWorker from 'pdfjs-dist/legacy/build/pdf.worker?url';

  pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorker;
  const todayIso = new Date().toISOString().split('T')[0];

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
  const ELIGIBLE_TAVI_STATUSES = ['In attesa di TAVI', 'TAVI eseguita'];
  const PRIORITY_OPTIONS = [
    { value: 'alta', label: 'Alta (entro 1 mese)' },
    { value: 'media', label: 'Media (entro 3 mesi)' },
    { value: 'bassa', label: 'Bassa (oltre 3 mesi)' },
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
    { value: 'edwards_sapien_3_ultra', label: 'Edwards Sapien 3 Ultra', sizes: ['20', '23', '26'] },
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
  ];

  let statusSelection = '';
  let savingStatus = false;
  let checklistState = {};
  let lastLoadedChecklistId = null;
  let patientAge = null;
  let taviDate = '';
  let savingTaviDate = false;
  let anagraficaForm = {
    nome: '',
    cognome: '',
    data_nascita: '',
    sesso: 'M',
    priority: 'media',
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
    dataVisita: '',
    anamnesi: '',
    apr: '',
    visita: '',
    conclusioni: '',
    medicoTitolo: 'Dott.',
    medicoNome: 'Pierluigi Merella',
    specializzandoTitolo: 'Dott.',
    specializzandoNome: '',
  };
  let savingAmbulatorio = false;
  let schedaProceduraleForm = {
    peso: '',
    altezza: '',
    allergiaMdc: '',
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
    pacemaker: '',
    pacemakerNote: '',
    accessoPrincipaleFem: '',
    accessoPrincipaleAltro: '',
  accessoProtezione: '',
  accessoProtezioneNote: '',
  altriAccessi: '',
  diametroPalloneFemorale: '',
  guidaSafari: '',
  protezioneOsti: '',
  valvuloplastica: '',
  valvuloplasticaNote: '',
  bioprotesiModello: '',
  bioprotesiDimensione: '',
};
  let savingSchedaProcedurale = false;
  let generatingSchedaReferto = false;
  let generatingReferto = false;
  let generatingConsenso = false;
  let generatingEsami = false;
  let showPrintPreview = false;
  let silentPrintMode = false;
  let printMode = 'html';
  let printTitle = '';
  let printHtml = '';
  let printDocxPath = '';
  let printPdfPath = '';
  let printDocxContainer = null;
  let printPdfContainer = null;
  let renderingPdf = false;
  let autoPrintPending = false;
  let deletingPatient = false;
  let showDeleteConfirm = false;
  let showAmbulatorioStatusConfirm = false;
  let changingStatusAfterReferto = false;
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
      ambulatorioForm = {
        fattori: [],
        dataVisita: '',
        anamnesi: '',
        apr: '',
        visita: '',
        conclusioni: '',
        medicoTitolo: 'Dott.',
        medicoNome: 'Pierluigi Merella',
        specializzandoTitolo: 'Dott.',
        specializzandoNome: '',
      };
      return;
    }

    ambulatorioForm = {
      fattori: patient.patient.ambulatorio_fattori || [],
      dataVisita: patient.patient.ambulatorio_data_visita || '',
      anamnesi: stripFormatting(patient.patient.anamnesi_cardiologica || ''),
      apr: stripFormatting(patient.patient.apr || ''),
      visita: stripFormatting(patient.patient.visita_odierna || ''),
      conclusioni: stripFormatting(patient.patient.conclusioni || ''),
      medicoTitolo: patient.patient.medico_titolo || 'Dott.',
      medicoNome: patient.patient.medico_nome || 'Pierluigi Merella',
      specializzandoTitolo: patient.patient.medico_specializzando_titolo || 'Dott.',
      specializzandoNome: patient.patient.medico_specializzando_nome || '',
    };
  }

  function loadSchedaProcedurale() {
    if (!patient?.patient) {
    schedaProceduraleForm = {
      peso: '',
      altezza: '',
      allergiaMdc: '',
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
        pacemaker: '',
        pacemakerNote: '',
        accessoPrincipaleFem: '',
        accessoPrincipaleAltro: '',
      accessoProtezione: '',
      accessoProtezioneNote: '',
      altriAccessi: '',
      diametroPalloneFemorale: '',
      guidaSafari: '',
      protezioneOsti: '',
      valvuloplastica: '',
      valvuloplasticaNote: '',
      bioprotesiModello: '',
      bioprotesiDimensione: '',
    };
      return;
    }

    schedaProceduraleForm = {
      peso: patient.patient.peso ?? '',
      altezza: patient.patient.altezza ?? '',
      allergiaMdc: patient.patient.procedurale_allergia_mdc || '',
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
      pacemaker: patient.patient.procedurale_pacemaker || '',
      pacemakerNote: patient.patient.procedurale_pacemaker_note || '',
      accessoPrincipaleFem: patient.patient.procedurale_accesso_principale_fem || '',
      accessoPrincipaleAltro: patient.patient.procedurale_accesso_principale_altro || '',
      accessoProtezione: patient.patient.procedurale_accesso_protezione || '',
      accessoProtezioneNote: patient.patient.procedurale_accesso_protezione_note || '',
      altriAccessi: patient.patient.procedurale_altri_accessi || '',
      diametroPalloneFemorale: patient.patient.procedurale_diametro_pallone_femorale || '',
      guidaSafari: patient.patient.procedurale_guida_safari || '',
      protezioneOsti: patient.patient.procedurale_protezione_osti || '',
      valvuloplastica: patient.patient.procedurale_valvuloplastica || '',
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

  const getPlaceLabel = (item) => {
    if (typeof item === 'string') return item;
    return item?.nome || item?.name || item?.label || '';
  };

  const getPlaceCode = (item) => {
    if (typeof item !== 'object' || item === null) return null;
    return item.codice_catastale || item.codice || null;
  };

  const findPlaceCode = (value) => {
    if (!value || !value.trim()) return '';
    const q = value.trim().toLowerCase();
    const base = [...(placeData.comuni || []), ...(placeData.stati || [])];
    const match = base.find((item) => getPlaceLabel(item).trim().toLowerCase() === q);
    return match ? getPlaceCode(match) || '' : '';
  };

  const filterPlaces = (value) => {
    if (!value || value.trim().length < 2) return [];
    const q = value.trim().toLowerCase();
    const base = [...(placeData.comuni || []), ...(placeData.stati || [])];
    return base
      .map(item => {
        const label = getPlaceLabel(item);
        const codice = getPlaceCode(item);
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
  $: eligibleForTavi = ELIGIBLE_TAVI_STATUSES.includes(statusSelection || patient?.status || '');

  $: {
    const creat = normalizeNumber(schedaProceduraleForm.creatinina);
    const sex = anagraficaForm.sesso || patient?.patient?.sesso;
    const egfrCalc = calculateEgfrCkdEpi(creat, patientAge, sex);
    if (egfrCalc !== null && egfrCalc !== undefined) {
      const next = String(egfrCalc);
      if (schedaProceduraleForm.egfr !== next) {
        schedaProceduraleForm.egfr = next;
      }
    } else if (!schedaProceduraleForm.creatinina) {
      if (schedaProceduraleForm.egfr !== '') {
        schedaProceduraleForm.egfr = '';
      }
    }
  }

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

  const isValidISODate = (value) => /^\d{4}-\d{2}-\d{2}$/.test(value);

  async function saveTaviDate() {
    if (!patient?.patient?.id) return;
    if (!eligibleForTavi) {
      showToast('La data TAVI è disponibile solo per pazienti eleggibili', 'warning');
      return;
    }
    if (taviDate && !isValidISODate(taviDate)) {
      showToast('Inserisci una data TAVI valida (YYYY-MM-DD)', 'warning');
      return;
    }

    savingTaviDate = true;
    const payload = {
      ...patient.patient,
      data_tavi: taviDate || null,
    };

    try {
      await updatePatient(payload);
      await loadPatient(patient.patient.id);
      showToast('Data TAVI aggiornata', 'success');
    } catch (e) {
      console.error(e);
      showToast('Errore durante il salvataggio della data TAVI', 'error');
    } finally {
      savingTaviDate = false;
    }
  }

  const normalizeText = (value) => {
    if (value === undefined || value === null) return null;
    const v = String(value).trim();
    return v === '' ? null : v;
  };

  const stripFormatting = (value) => {
    if (value === undefined || value === null) return '';
    let v = String(value);
    v = v.replace(/<\/?[a-z][^>]*>/gi, '');
    v = v.replace(/\*\*(.*?)\*\*/g, '$1');
    v = v.replace(/__(.*?)__/g, '$1');
    v = v.replace(/\*(.*?)\*/g, '$1');
    return v;
  };

  const normalizePlainText = (value) => {
    return normalizeText(stripFormatting(value));
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

  function formatDisplayFromIso(value) {
    if (!value) return '';
    const clean = String(value).split('T')[0];
    const parts = clean.split('-');
    if (parts.length === 3 && parts[0].length === 4) {
      const [year, month, day] = parts;
      return `${day.padStart(2, '0')}/${month.padStart(2, '0')}/${year}`;
    }
    const digits = clean.replace(/\D/g, '');
    if (digits.length === 8 && Number(digits.slice(0, 4)) > 31) {
      return `${digits.slice(6, 8)}/${digits.slice(4, 6)}/${digits.slice(0, 4)}`;
    }
    return formatDisplayFromDigits(digits);
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
    anagraficaForm.luogo_nascita_codice = findPlaceCode(anagraficaForm.luogo_nascita);
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
      priority: normalizeText(anagraficaForm.priority),
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

  async function confirmAmbulatorioStatusChange() {
    if (!patient?.patient?.id) return;
    changingStatusAfterReferto = true;
    try {
      await changePatientStatus(patient.patient.id, 'In corso di accertamenti');
      await loadPatient(patient.patient.id);
      statusSelection = 'In corso di accertamenti';
      showToast('Stato paziente aggiornato', 'success');
    } catch (e) {
      console.error(e);
      showToast('Errore durante l\'aggiornamento dello stato', 'error');
    } finally {
      changingStatusAfterReferto = false;
      showAmbulatorioStatusConfirm = false;
    }
  }

  function cancelAmbulatorioStatusChange() {
    showAmbulatorioStatusConfirm = false;
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
      ambulatorio_data_visita: normalizeText(ambulatorioForm.dataVisita),
      anamnesi_cardiologica: normalizePlainText(ambulatorioForm.anamnesi),
      apr: normalizePlainText(ambulatorioForm.apr),
      visita_odierna: normalizePlainText(ambulatorioForm.visita),
      conclusioni: normalizePlainText(ambulatorioForm.conclusioni),
      medico_titolo: normalizeText(ambulatorioForm.medicoTitolo),
      medico_nome: normalizeText(ambulatorioForm.medicoNome),
      medico_specializzando_titolo: normalizeText(ambulatorioForm.specializzandoTitolo),
      medico_specializzando_nome: normalizeText(ambulatorioForm.specializzandoNome),
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
      peso: normalizeNumber(antropometricForm.peso),
      altezza: normalizeNumber(antropometricForm.altezza),
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
      if (patient.status !== 'In corso di accertamenti') {
        showAmbulatorioStatusConfirm = true;
      }
    } catch (e) {
      console.error('Errore referto', e);
      const msg = typeof e === 'string' ? e : e?.message || 'Errore durante la generazione del referto';
      showToast(msg, 'error');
    } finally {
      generatingReferto = false;
    }
  }

  async function generateConsensoInformato() {
    if (!patient?.patient?.id) return;
    generatingConsenso = true;
    try {
      const docxResult = await invoke('generate_consenso_informato', {
        patientId: patient.patient.id
      });
      const docxPath = typeof docxResult === 'string' ? docxResult : '';

      if (docxPath) {
        showToast('Invio il consenso informato alla stampante...', 'info');
        try {
          await invoke('print_file', { path: docxPath });
          showToast('Stampa del consenso avviata', 'success');
        } catch (printErr) {
          console.error('Errore stampa diretta consenso', printErr);
          showToast('Stampa diretta non riuscita, apro anteprima...', 'warning');
          await openPrintPreview({
            title: 'Consenso informato',
            docxPath
          });
        }
      } else {
        showToast('Consenso informato generato', 'success');
      }
    } catch (e) {
      console.error('Errore consenso informato', e);
      const msg = typeof e === 'string' ? e : e?.message || 'Errore durante la generazione del consenso informato';
      showToast(msg, 'error');
    } finally {
      generatingConsenso = false;
    }
  }

  async function generateEsamiEmatochimici() {
    if (!patient?.patient?.id) return;
    generatingEsami = true;
    try {
      const result = await invoke('generate_esami_ematochimici', {
        patientId: patient.patient.id
      });
      const path = typeof result === 'string' ? result : '';
      if (path) {
        showToast('Invio la lista esami alla stampante...', 'info');
        try {
          await invoke('print_file', { path });
          showToast('Stampa della lista esami avviata', 'success');
        } catch (printErr) {
          console.error('Errore stampa diretta lista esami', printErr);
          showToast('Stampa diretta non riuscita, apro anteprima...', 'warning');
          await openPrintPreview({
            title: 'Lista esami',
            pdfPath: path
          });
        }
      } else {
        showToast('Lista esami generata', 'success');
      }
    } catch (e) {
      console.error('Errore esami ematochimici', e);
      const msg = typeof e === 'string' ? e : e?.message || 'Errore durante la generazione della lista esami';
      showToast(msg, 'error');
    } finally {
      generatingEsami = false;
    }
  }

  async function openPrintPreview({ title, html, pdfPath, docxPath, skipPreview = false }) {
    printTitle = title || 'Anteprima stampa';
    silentPrintMode = skipPreview;
    showPrintPreview = !skipPreview;
    if (docxPath) {
      printMode = 'docx';
      printDocxPath = docxPath;
      printPdfPath = '';
      printHtml = '';
      renderingPdf = false;
    } else if (html) {
      printMode = 'html';
      printHtml = html;
      printDocxPath = '';
      printPdfPath = '';
      renderingPdf = false;
    } else if (pdfPath) {
      printMode = 'pdf';
      printPdfPath = pdfPath;
      printDocxPath = '';
      printHtml = '';
    }
    autoPrintPending = true;
    await tick();
    if (printMode === 'docx') {
      await renderDocxPreview();
    } else if (printMode === 'pdf') {
      await renderPdfPreview();
    } else if (printMode === 'html') {
      await triggerPrint();
      autoPrintPending = false;
    }

    if (autoPrintPending) {
      autoPrintPending = false;
      await triggerPrint();
    }

    if (skipPreview) {
      closePrintPreview();
    }
  }

  function closePrintPreview() {
    silentPrintMode = false;
    showPrintPreview = false;
    printHtml = '';
    printDocxPath = '';
    printPdfPath = '';
    autoPrintPending = false;
    renderingPdf = false;
    if (printDocxContainer) {
      printDocxContainer.innerHTML = '';
    }
    if (printPdfContainer) {
      printPdfContainer.innerHTML = '';
    }
  }

  async function triggerPrint() {
    try {
      await invoke('print_window');
      return;
    } catch (e) {
      console.error('Errore stampa', e);
      // fallback to window.print
    }
    window.print();
  }

  async function renderDocxPreview() {
    if (!printDocxPath || !printDocxContainer) return;
    try {
      const buffer = await readBinaryFile(printDocxPath);
      printDocxContainer.innerHTML = '';
      await renderAsync(buffer, printDocxContainer, printDocxContainer, {
        inWrapper: true,
        className: 'docx',
        useBase64URL: true,
      });
    } catch (e) {
      console.error('Errore anteprima consenso', e);
      showToast('Errore durante la preparazione del consenso informato', 'error');
    }
  }

  async function renderPdfPreview() {
    if (!printPdfPath || !printPdfContainer) return;
    try {
      renderingPdf = true;
      printPdfContainer.innerHTML = '';
      const buffer = await readBinaryFile(printPdfPath);
      const pdf = await pdfjsLib.getDocument({ data: buffer }).promise;
      const containerWidth = printPdfContainer.clientWidth || 840;
      const dpr = window.devicePixelRatio || 1;

      for (let pageNum = 1; pageNum <= pdf.numPages; pageNum += 1) {
        const page = await pdf.getPage(pageNum);
        const baseViewport = page.getViewport({ scale: 1 });
        const scale = containerWidth / baseViewport.width;
        const viewport = page.getViewport({ scale: scale * dpr });
        const canvas = document.createElement('canvas');
        const context = canvas.getContext('2d');
        canvas.width = viewport.width;
        canvas.height = viewport.height;
        canvas.style.width = `${baseViewport.width * scale}px`;
        canvas.style.height = `${baseViewport.height * scale}px`;
        if (context) {
          await page.render({ canvasContext: context, viewport }).promise;
        }
        printPdfContainer.appendChild(canvas);
      }
    } catch (e) {
      console.error('Errore anteprima PDF', e);
      showToast('Errore durante la visualizzazione del PDF', 'error');
    } finally {
      renderingPdf = false;
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

  async function openRefertoFile(path) {
    if (!path) return;
    const fileUrl = path.startsWith('file://') ? path : `file://${encodeURI(path)}`;
    try {
      await openExternal(path);
    } catch (e) {
      try {
        await openExternal(fileUrl);
      } catch (e2) {
        console.error('Errore apertura referto', e, e2);
        const msg = e2?.message || e?.message || 'Errore apertura referto';
        showToast(msg, 'error');
      }
    }
  }

  async function maybeOpenReferto(path) {
    if (!path) return;
    const shouldOpen = await shouldAutoOpenReferto();
    if (!shouldOpen) return;
    await openRefertoFile(path);
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
      priority: patient.patient.priority || 'media',
      luogo_nascita: patient.patient.luogo_nascita || '',
      luogo_nascita_codice: findPlaceCode(patient.patient.luogo_nascita || ''),
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
      dataVisita: patient.patient.ambulatorio_data_visita || '',
      anamnesi: stripFormatting(patient.patient.anamnesi_cardiologica || ''),
      apr: stripFormatting(patient.patient.apr || ''),
      visita: stripFormatting(patient.patient.visita_odierna || ''),
      conclusioni: stripFormatting(patient.patient.conclusioni || ''),
      medicoTitolo: patient.patient.medico_titolo || 'Dott.',
      medicoNome: patient.patient.medico_nome || 'Pierluigi Merella',
      specializzandoTitolo: patient.patient.medico_specializzando_titolo || 'Dott.',
      specializzandoNome: patient.patient.medico_specializzando_nome || '',
    };
    taviDate = patient.patient.data_tavi || '';
    anagraficaDateDisplay = formatDisplayFromIso(patient.patient.data_nascita || '');
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
        <button
          type="button"
          on:click={onBack}
          class="w-9 h-9 rounded-full border border-gray-200 bg-surface text-textPrimary hover:bg-surface-stronger transition-colors flex items-center justify-center shrink-0 -ml-1"
          aria-label="Torna alla home"
        >
          ←
        </button>
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

      <div class="flex flex-col sm:flex-row lg:flex-col gap-2 w-full lg:w-auto">
        <Button
          variant="secondary"
          size="sm"
          disabled={generatingConsenso}
          on:click={generateConsensoInformato}
        >
          {generatingConsenso ? 'Generazione...' : 'Stampa consenso informato'}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          disabled={generatingEsami}
          on:click={generateEsamiEmatochimici}
        >
          {generatingEsami ? 'Generazione...' : 'Stampa lista esami'}
        </Button>
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
        {#if eligibleForTavi}
          <Input
            type="date"
            label="Data TAVI programmata"
            bind:value={taviDate}
            max={todayIso}
          />
          <Button
            variant="secondary"
            fullWidth
            size="sm"
            disabled={savingTaviDate}
            on:click={saveTaviDate}
          >
            {savingTaviDate ? 'Salvataggio...' : 'Salva data TAVI'}
          </Button>
        {/if}
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
                        ></div>
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
                <label class="block text-sm font-medium text-textPrimary mb-1" for="anagraficaDataNascita">
                  Data di nascita <span class="text-error">*</span>
                </label>
                <Input
                  id="anagraficaDataNascita"
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
                  <div class="absolute z-30 mt-1 w-full bg-surface border border-gray-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
                    {#each filterPlaces(anagraficaForm.luogo_nascita) as suggestion}
                      <button
                        type="button"
                        class="w-full text-left px-3 py-2 text-sm hover:bg-surface-stronger"
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
              <Select
                label="Priorità"
                options={PRIORITY_OPTIONS}
                bind:value={anagraficaForm.priority}
              />
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
                  <div class="absolute z-30 mt-1 w-full bg-surface border border-gray-200 rounded-lg shadow-lg max-h-48 overflow-y-auto">
                    {#each filterPlaces(anagraficaForm.provenienza) as suggestion}
                      <button
                        type="button"
                        class="w-full text-left px-3 py-2 text-sm hover:bg-surface-stronger"
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
              <div class="sm:col-span-2 flex items-center gap-3 p-3 rounded-lg bg-surface-strong text-sm">
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
                <label class="block text-sm font-medium text-textPrimary mb-1" for="ambulatorioMedicoTitolo">Titolo medico</label>
                <select
                  id="ambulatorioMedicoTitolo"
                  class="w-full px-3 py-2 border rounded-lg border-gray-200 focus:ring-primary/20 focus:border-primary"
                  bind:value={ambulatorioForm.medicoTitolo}
                >
                  <option value="Dott.">Dott.</option>
                  <option value="Dott.ssa">Dott.ssa</option>
                </select>
              </div>
              <Input
                label="Cardiologo refertante"
                placeholder="Nome e cognome"
                bind:value={ambulatorioForm.medicoNome}
              />
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-textPrimary mb-1" for="ambulatorioSpecializzandoTitolo">Titolo specializzando</label>
                <select
                  id="ambulatorioSpecializzandoTitolo"
                  class="w-full px-3 py-2 border rounded-lg border-gray-200 focus:ring-primary/20 focus:border-primary"
                  bind:value={ambulatorioForm.specializzandoTitolo}
                >
                  <option value="Dott.">Dott.</option>
                  <option value="Dott.ssa">Dott.ssa</option>
                </select>
              </div>
              <Input
                label="Medico specializzando"
                placeholder="Nome e cognome"
                bind:value={ambulatorioForm.specializzandoNome}
              />
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <Input
                label="Data visita ambulatorio"
                type="date"
                bind:value={ambulatorioForm.dataVisita}
              />
            </div>
            <CVRiskFactors label="Fattori di rischio CV" bind:selectedFactors={ambulatorioForm.fattori} />
            <div class="space-y-2">
              <label class="block text-sm font-semibold text-textPrimary">Anamnesi Patologica Remota</label>
              <textarea
                class="w-full min-h-[140px] px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary bg-surface text-textPrimary"
                placeholder="Inserisci l'anamnesi patologica remota..."
                bind:value={ambulatorioForm.anamnesi}
              ></textarea>
            </div>
            <div class="space-y-2">
              <label class="block text-sm font-semibold text-textPrimary">Terapia domiciliare</label>
              <textarea
                class="w-full min-h-[140px] px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary bg-surface text-textPrimary"
                placeholder="Inserisci la terapia domiciliare..."
                bind:value={ambulatorioForm.apr}
              ></textarea>
            </div>
            <div class="space-y-2">
              <label class="block text-sm font-semibold text-textPrimary">Visita odierna</label>
              <textarea
                class="w-full min-h-[140px] px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary bg-surface text-textPrimary"
                placeholder="Esame obiettivo, sintomi, parametri rilevati..."
                bind:value={ambulatorioForm.visita}
              ></textarea>
            </div>
            <div class="space-y-2">
              <label class="block text-sm font-semibold text-textPrimary">Conclusioni</label>
              <textarea
                class="w-full min-h-[140px] px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary bg-surface text-textPrimary"
                placeholder="Decisioni, piano terapeutico, follow-up..."
                bind:value={ambulatorioForm.conclusioni}
              ></textarea>
            </div>
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
              <Input label="Peso (Kg)" type="number" min="0" bind:value={antropometricForm.peso} />
              <Input label="Altezza (cm)" type="number" min="0" bind:value={antropometricForm.altezza} />
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
                <p class="block text-sm font-semibold text-textPrimary mb-1">Pacemaker definitivo</p>
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
                <p class="block text-sm font-semibold text-textPrimary mb-1">Accesso arterioso di protezione</p>
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
                <p class="block text-sm font-semibold text-textPrimary mb-1">Necessità di protezione osti coronarici</p>
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

    {#if showPrintPreview || silentPrintMode}
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div
        class={`fixed inset-0 z-50 flex items-center justify-center p-4 print-overlay ${silentPrintMode ? 'silent-print bg-transparent' : 'bg-black/50'}`}
        on:click={silentPrintMode ? undefined : closePrintPreview}
      >
        <div
          class="bg-surface rounded-2xl shadow-xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col print-container"
          on:click|stopPropagation
        >
          {#if !silentPrintMode}
            <div class="px-5 py-4 border-b border-gray-200 flex items-center justify-between gap-4 no-print">
              <div>
                <h3 class="text-lg font-semibold text-textPrimary">{printTitle}</h3>
                <p class="text-xs text-textSecondary">Anteprima del documento pronta per la stampa</p>
              </div>
              <Button variant="text" size="sm" on:click={closePrintPreview}>Chiudi</Button>
            </div>
          {/if}

          <div class="flex-1 overflow-auto bg-surface-muted p-6">
            <div class="print-surface bg-surface shadow-sm mx-auto w-full max-w-[840px]">
              {#if printMode === 'docx'}
                <div class="print-docx docx p-8" bind:this={printDocxContainer}></div>
              {:else if printMode === 'pdf'}
                <div class="print-pdf p-6" bind:this={printPdfContainer}>
                  {#if renderingPdf}
                    <p class="text-sm text-textSecondary">Caricamento PDF...</p>
                  {/if}
                </div>
              {:else}
                <div class="print-content p-8">
                  {@html printHtml}
                </div>
              {/if}
            </div>
          </div>

          {#if !silentPrintMode}
            <div class="px-5 py-4 border-t border-gray-200 flex justify-end gap-2 no-print">
              <Button variant="text" size="sm" on:click={closePrintPreview}>Chiudi</Button>
              <Button variant="primary" size="sm" on:click={triggerPrint}>Stampa</Button>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    {#if showDeleteConfirm}
      <div
        class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4"
        on:click={cancelDelete}
        role="button"
        tabindex="0"
        on:keydown={(e) =>
          (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') && cancelDelete()}
      >
        <div
          class="max-w-md w-full"
          on:click|stopPropagation
          on:keydown|stopPropagation
          role="dialog"
          aria-modal="true"
          tabindex="-1"
        >
          <Card padding="lg" class="bg-surface">
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

    {#if showAmbulatorioStatusConfirm}
      <div
        class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4"
        on:click={cancelAmbulatorioStatusChange}
        role="button"
        tabindex="0"
        on:keydown={(e) =>
          (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') && cancelAmbulatorioStatusChange()}
      >
        <div
          class="max-w-md w-full"
          on:click|stopPropagation
          on:keydown|stopPropagation
          role="dialog"
          aria-modal="true"
          tabindex="-1"
        >
          <Card padding="lg" class="bg-surface">
            <div class="flex items-start gap-3">
              <IconBadge icon="info" tone="secondary" />
              <div>
                <h3 class="text-lg font-semibold text-textPrimary">Cambiare stato paziente?</h3>
                <p class="text-sm text-textSecondary mt-1">
                  Vuoi cambiare lo stato del paziente
                  <span class="font-semibold text-textPrimary">
                    {patient.patient.nome} {patient.patient.cognome}
                  </span>
                  a &quot;In corso di accertamenti&quot;?
                </p>
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-6">
              <Button variant="text" size="sm" on:click={cancelAmbulatorioStatusChange}>Non ora</Button>
              <Button variant="primary" size="sm" on:click={confirmAmbulatorioStatusChange} disabled={changingStatusAfterReferto}>
                {changingStatusAfterReferto ? 'Aggiornamento...' : 'Sì, cambia stato'}
              </Button>
            </div>
          </Card>
        </div>
      </div>
    {/if}

  </div>
{/if}

<style>
  .print-content {
    font-family: "Times New Roman", serif;
    font-size: 12pt;
    line-height: 1.4;
    color: #111;
  }

  .print-content p {
    margin: 0 0 12px;
  }

  .print-pdf canvas {
    display: block;
    width: 100%;
    height: auto;
    margin: 0 auto 16px;
  }

  @page {
    size: A4;
    margin: 15mm;
  }

  .silent-print {
    opacity: 0;
    pointer-events: none;
  }

  @media print {
    :global(body *) {
      visibility: hidden;
    }

    :global(.print-surface),
    :global(.print-surface *) {
      visibility: visible;
    }

    :global(.print-container) {
      max-height: none !important;
      overflow: visible !important;
      height: auto !important;
    }

    :global(.print-overlay) {
      align-items: flex-start !important;
    }

    :global(.silent-print) {
      opacity: 1 !important;
      pointer-events: auto !important;
    }

    :global(.print-overlay) {
      background: transparent;
    }

    :global(.print-surface) {
      position: fixed;
      inset: 0;
      margin: 0;
      max-width: none;
      box-shadow: none;
    }

    :global(.no-print) {
      display: none !important;
    }
  }
</style>
