import comuni from '../data/comuni.json';
import stati from '../data/stati.json';

export function loadPlaces() {
  return { comuni, stati };
}

function normalizeItem(item) {
  if (typeof item === 'string') return item;
  if (item && typeof item === 'object') return item.nome || item.name || '';
  return '';
}

export function filterPlaces(query) {
  if (!query || query.trim().length < 2) return [];
  const q = query.trim().toLowerCase();
  const { comuni, stati } = loadPlaces();

  const comuniMatch = comuni
    .map(c => ({ label: normalizeItem(c), codice: c.codice_catastale || null }))
    .filter(c => c.label.toLowerCase().includes(q));

  const statiMatch = stati
    .map(s => ({ label: normalizeItem(s), codice: null }))
    .filter(s => s.label.toLowerCase().includes(q));

  return [...comuniMatch, ...statiMatch].slice(0, 15);
}
