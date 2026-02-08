export const STATUS_OPTIONS = [
  { value: 'Da valutare', label: 'Da valutare' },
  { value: 'In corso di accertamenti', label: 'In corso di accertamenti' },
  { value: 'In attesa di TAVI', label: 'In attesa di TAVI' },
  { value: 'Non candidabile a TAVI', label: 'Non candidabile a TAVI' },
  { value: 'TAVI eseguita', label: 'TAVI eseguita' },
];

export const STATUS_SUMMARY_LABELS = {
  'Non candidabile a TAVI': 'Non candidabili a TAVI',
};

export const PRIORITY_OPTIONS = [
  { value: 'alta', label: 'Alta (entro 1 mese)' },
  { value: 'media', label: 'Media (entro 3 mesi)' },
  { value: 'bassa', label: 'Bassa (oltre 3 mesi)' },
];

export const ACTIVE_PROGRESS_KEYS = new Set([
  'Da valutare',
  'In corso di accertamenti',
  'In attesa di TAVI',
]);

export const ELIGIBLE_TAVI_STATUSES = ['In attesa di TAVI', 'TAVI eseguita'];

export const STATUS_COLORS = {
  'Da valutare': 'bg-blue-100 text-blue-800',
  'In corso di accertamenti': 'bg-yellow-100 text-yellow-800',
  'In attesa di TAVI': 'bg-purple-100 text-purple-800',
  'Non candidabile a TAVI': 'bg-red-100 text-red-800',
  'TAVI eseguita': 'bg-green-100 text-green-800',
};
