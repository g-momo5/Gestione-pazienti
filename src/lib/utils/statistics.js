/**
 * Calcola il BMI (Body Mass Index)
 */
export function calculateBMI(peso, altezza) {
  if (!peso || !altezza || altezza <= 0) return null;

  const altezzaM = altezza / 100;
  const bmi = peso / (altezzaM * altezzaM);

  return Math.round(bmi * 10) / 10;
}

/**
 * Calcola la BSA (Body Surface Area) con formula di Mosteller
 */
export function calculateBSA(peso, altezza) {
  if (!peso || !altezza || altezza <= 0) return null;

  const bsa = Math.sqrt((altezza * peso) / 3600);
  return Math.round(bsa * 100) / 100; // m² con 2 decimali
}

/**
 * Restituisce una categoria BMI testuale
 */
export function categorizeBMI(bmi) {
  if (bmi === null || bmi === undefined || isNaN(bmi)) return '-';

  if (bmi < 18.5) return 'Sottopeso';
  if (bmi < 25) return 'Normopeso';
  if (bmi < 30) return 'Sovrappeso';
  if (bmi < 35) return 'Obeso classe I';
  if (bmi < 40) return 'Obeso classe II';
  return 'Obeso classe III';
}

/**
 * Formatta un numero decimale italiano (virgola invece di punto)
 */
export function formatNumberIT(num, decimals = 1) {
  if (num === null || num === undefined || isNaN(num)) return '-';

  return num.toFixed(decimals).replace('.', ',');
}

/**
 * Formatta una percentuale
 */
export function formatPercentage(value) {
  if (value === null || value === undefined || isNaN(value)) return '-';

  return `${Math.round(value)}%`;
}

/**
 * Arrotonda un numero a N decimali
 */
export function roundTo(num, decimals = 1) {
  if (num === null || num === undefined || isNaN(num)) return null;

  const factor = Math.pow(10, decimals);
  return Math.round(num * factor) / factor;
}

/**
 * Calcola la media di un array di numeri
 */
export function calculateAverage(numbers) {
  const validNumbers = numbers.filter(n => n !== null && n !== undefined && !isNaN(n));

  if (validNumbers.length === 0) return null;

  const sum = validNumbers.reduce((acc, val) => acc + val, 0);
  return sum / validNumbers.length;
}

/**
 * Calcola la media di un campo specifico da un array di oggetti
 */
export function calculateFieldAverage(items, fieldName) {
  const values = items.map(item => item[fieldName]).filter(v => v !== null && v !== undefined);
  return calculateAverage(values);
}

/**
 * Conta occorrenze di valori in un array
 */
export function countOccurrences(items, fieldName) {
  const counts = {};

  items.forEach(item => {
    const value = item[fieldName];
    if (value !== null && value !== undefined) {
      counts[value] = (counts[value] || 0) + 1;
    }
  });

  return counts;
}

/**
 * Ottieni top N elementi per occorrenze
 */
export function getTopN(items, fieldName, n = 5) {
  const counts = countOccurrences(items, fieldName);

  const sorted = Object.entries(counts)
    .sort((a, b) => b[1] - a[1])
    .slice(0, n);

  return sorted.map(([name, count]) => ({ name, count }));
}

/**
 * Calcola percentuale di un valore booleano true
 */
export function calculateBooleanPercentage(items, fieldName) {
  if (items.length === 0) return 0;

  const trueCount = items.filter(item => item[fieldName] === true).length;
  return (trueCount / items.length) * 100;
}

/**
 * Raggruppa items per un campo
 */
export function groupBy(items, fieldName) {
  return items.reduce((groups, item) => {
    const key = item[fieldName];
    if (!groups[key]) {
      groups[key] = [];
    }
    groups[key].push(item);
    return groups;
  }, {});
}

/**
 * Prepara dati per Pie Chart (ECharts)
 */
export function preparePieChartData(items, fieldName) {
  const counts = countOccurrences(items, fieldName);

  return Object.entries(counts).map(([name, value]) => ({
    name,
    value,
  }));
}

/**
 * Prepara dati per Bar Chart (ECharts)
 */
export function prepareBarChartData(items, fieldName, topN = 5) {
  const topItems = getTopN(items, fieldName, topN);

  return {
    labels: topItems.map(item => item.name),
    values: topItems.map(item => item.count),
  };
}

/**
 * Filtra procedures per periodo
 */
export function filterByPeriod(procedures, period) {
  if (!period || period === 'all') return procedures;

  const now = new Date();
  const cutoffDate = new Date();

  switch (period) {
    case '1m':
      cutoffDate.setMonth(now.getMonth() - 1);
      break;
    case '3m':
      cutoffDate.setMonth(now.getMonth() - 3);
      break;
    case '6m':
      cutoffDate.setMonth(now.getMonth() - 6);
      break;
    case '1y':
      cutoffDate.setFullYear(now.getFullYear() - 1);
      break;
    default:
      return procedures;
  }

  return procedures.filter(proc => {
    const procDate = new Date(proc.data_procedura);
    return procDate >= cutoffDate;
  });
}

/**
 * Filtra procedures per tipo valvola
 */
export function filterByValveType(procedures, valveType) {
  if (!valveType || valveType === 'all') return procedures;

  return procedures.filter(proc => proc.tipo_valvola === valveType);
}

/**
 * Filtra procedures per query di ricerca
 */
export function filterBySearchQuery(procedures, query) {
  if (!query || query.trim() === '') return procedures;

  const lowerQuery = query.toLowerCase();

  return procedures.filter(proc => {
    const nome = (proc.nome || '').toLowerCase();
    const cognome = (proc.cognome || '').toLowerCase();
    const modello = (proc.modello_valvola || '').toLowerCase();

    return nome.includes(lowerQuery) ||
           cognome.includes(lowerQuery) ||
           modello.includes(lowerQuery);
  });
}

/**
 * Applica tutti i filtri alle procedures
 */
export function applyFilters(procedures, filters) {
  let filtered = [...procedures];

  if (filters.search_query) {
    filtered = filterBySearchQuery(filtered, filters.search_query);
  }

  if (filters.tipo_valvola) {
    filtered = filterByValveType(filtered, filters.tipo_valvola);
  }

  if (filters.period) {
    filtered = filterByPeriod(filtered, filters.period);
  }

  return filtered;
}
