// Modelli di Valvole (identici al progetto Flutter)

export const BALLOON_EXPANDABLE_MODELS = [
  'Edwards SAPIEN 3',
  'Edwards SAPIEN 3 Ultra',
  'Myval',
  'Allegra',
];

export const SELF_EXPANDABLE_MODELS = [
  'Medtronic CoreValve Evolut R',
  'Medtronic CoreValve Evolut PRO',
  'Medtronic CoreValve Evolut PRO+',
  'Boston Scientific ACURATE neo',
  'Portico',
];

export const VALVE_TYPES = {
  BALLOON: 'Balloon Expandable',
  SELF: 'Self Expandable',
};

// Fattori di rischio cardiovascolari
export const CV_RISK_FACTORS = [
  'Ipertensione arteriosa',
  'Diabete mellito',
  'Dislipidemia',
  'Fumo di sigaretta',
  'Obesità',
  'Familiarità per cardiopatia',
];

// Range di validazione per parametri medici
export const RANGES = {
  fe: { min: 0, max: 100, unit: '%', label: 'FE' },
  vmax: { min: 0, max: 10, unit: 'm/s', label: 'Vmax' },
  gmax: { min: 0, max: 200, unit: 'mmHg', label: 'Gmax' },
  gmed: { min: 0, max: 150, unit: 'mmHg', label: 'Gmed' },
  ava: { min: 0, max: 5, unit: 'cm²', label: 'AVA' },
  anulus: { min: 15, max: 35, unit: 'mm', label: 'Anulus Aortico' },
  altezza: { min: 100, max: 250, unit: 'cm', label: 'Altezza' },
  peso: { min: 30, max: 200, unit: 'kg', label: 'Peso' },
  dimensioneValvola: { min: 15, max: 35, unit: 'mm', label: 'Dimensione Valvola' },
};

// Periodi per filtri temporali
export const FILTER_PERIODS = [
  { value: 'all', label: 'Tutto il periodo' },
  { value: '1m', label: 'Ultimo mese' },
  { value: '3m', label: 'Ultimi 3 mesi' },
  { value: '6m', label: 'Ultimi 6 mesi' },
  { value: '1y', label: 'Ultimo anno' },
];

// Opzioni tipo valvola per filtri
export const VALVE_TYPE_FILTERS = [
  { value: 'all', label: 'Tutte le valvole' },
  { value: VALVE_TYPES.BALLOON, label: VALVE_TYPES.BALLOON },
  { value: VALVE_TYPES.SELF, label: VALVE_TYPES.SELF },
];

// Messaggi di errore
export const ERROR_MESSAGES = {
  required: 'Questo campo è obbligatorio',
  invalidNumber: 'Inserire un numero valido',
  outOfRange: (min, max, unit) => `Valore deve essere tra ${min} e ${max} ${unit}`,
  invalidDate: 'Data non valida',
  futureDate: 'La data non può essere nel futuro',
  invalidTime: 'Orario non valido',
  endBeforeStart: 'L\'orario di fine deve essere successivo all\'inizio',
  deleteConfirm: 'Sei sicuro di voler eliminare questa procedura?',
  saveSuccess: 'Procedura salvata con successo',
  saveError: 'Errore durante il salvataggio',
  deleteSuccess: 'Procedura eliminata con successo',
  deleteError: 'Errore durante l\'eliminazione',
};
