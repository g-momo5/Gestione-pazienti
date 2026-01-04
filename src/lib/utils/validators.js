import { RANGES, ERROR_MESSAGES } from '../constants.js';

/**
 * Valida un campo obbligatorio
 */
export function validateRequired(value) {
  if (value === null || value === undefined || value === '') {
    return ERROR_MESSAGES.required;
  }
  return null;
}

/**
 * Valida un numero con range opzionale
 */
export function validateNumber(value, min = null, max = null, unit = '') {
  if (value === null || value === undefined || value === '') {
    return null; // Non obbligatorio se vuoto
  }

  const num = parseFloat(value);
  if (isNaN(num)) {
    return ERROR_MESSAGES.invalidNumber;
  }

  if (min !== null && num < min) {
    return ERROR_MESSAGES.outOfRange(min, max || '∞', unit);
  }

  if (max !== null && num > max) {
    return ERROR_MESSAGES.outOfRange(min || '-∞', max, unit);
  }

  return null;
}

/**
 * Valida un campo numerico con range predefinito
 */
export function validateRange(field, value) {
  const range = RANGES[field];
  if (!range) return null;

  return validateNumber(value, range.min, range.max, range.unit);
}

/**
 * Valida una data
 */
export function validateDate(value, allowFuture = false) {
  if (!value) {
    return ERROR_MESSAGES.required;
  }

  const date = new Date(value);
  if (isNaN(date.getTime())) {
    return ERROR_MESSAGES.invalidDate;
  }

  if (!allowFuture && date > new Date()) {
    return ERROR_MESSAGES.futureDate;
  }

  return null;
}

/**
 * Valida un orario (formato HH:MM)
 */
export function validateTime(value) {
  if (!value) {
    return ERROR_MESSAGES.required;
  }

  const timeRegex = /^([0-1]?[0-9]|2[0-3]):[0-5][0-9]$/;
  if (!timeRegex.test(value)) {
    return ERROR_MESSAGES.invalidTime;
  }

  return null;
}

/**
 * Valida che ora_fine sia dopo ora_inizio
 */
export function validateTimeRange(startTime, endTime) {
  if (!startTime || !endTime) return null;

  const [startH, startM] = startTime.split(':').map(Number);
  const [endH, endM] = endTime.split(':').map(Number);

  const startMinutes = startH * 60 + startM;
  const endMinutes = endH * 60 + endM;

  if (endMinutes <= startMinutes) {
    return ERROR_MESSAGES.endBeforeStart;
  }

  return null;
}

/**
 * Valida un form completo di procedura
 */
export function validateProcedureForm(data) {
  const errors = {};

  // Dati Paziente
  if (!data.nome?.trim()) errors.nome = ERROR_MESSAGES.required;
  if (!data.cognome?.trim()) errors.cognome = ERROR_MESSAGES.required;

  const dataNascitaError = validateDate(data.data_nascita, false);
  if (dataNascitaError) errors.data_nascita = dataNascitaError;

  const altezzaError = validateRange('altezza', data.altezza);
  if (altezzaError) errors.altezza = altezzaError;

  const pesoError = validateRange('peso', data.peso);
  if (pesoError) errors.peso = pesoError;

  // Dati Pre-procedurali
  const feError = validateRange('fe', data.fe);
  if (feError) errors.fe = feError;

  const vmaxError = validateRange('vmax', data.vmax);
  if (vmaxError) errors.vmax = vmaxError;

  const gmaxError = validateRange('gmax', data.gmax);
  if (gmaxError) errors.gmax = gmaxError;

  const gmedError = validateRange('gmed', data.gmed);
  if (gmedError) errors.gmed = gmedError;

  const avaError = validateRange('ava', data.ava);
  if (avaError) errors.ava = avaError;

  const anulusError = validateRange('anulus', data.anulus_aortico);
  if (anulusError) errors.anulus_aortico = anulusError;

  // Se valvola protesica, modello e dimensione sono obbligatori
  if (data.valvola_protesica) {
    if (!data.protesica_modello?.trim()) {
      errors.protesica_modello = ERROR_MESSAGES.required;
    }
    if (!data.protesica_dimensione?.trim()) {
      errors.protesica_dimensione = ERROR_MESSAGES.required;
    }
  }

  // Dati Procedurali
  const dataProceError = validateDate(data.data_procedura, false);
  if (dataProceError) errors.data_procedura = dataProceError;

  const oraInizioError = validateTime(data.ora_inizio);
  if (oraInizioError) errors.ora_inizio = oraInizioError;

  const oraFineError = validateTime(data.ora_fine);
  if (oraFineError) errors.ora_fine = oraFineError;

  const timeRangeError = validateTimeRange(data.ora_inizio, data.ora_fine);
  if (timeRangeError) errors.ora_fine = timeRangeError;

  if (!data.tipo_valvola) errors.tipo_valvola = ERROR_MESSAGES.required;
  if (!data.modello_valvola?.trim()) errors.modello_valvola = ERROR_MESSAGES.required;

  const dimValvolaError = validateRange('dimensioneValvola', data.dimensione_valvola);
  if (dimValvolaError) errors.dimensione_valvola = dimValvolaError;

  return Object.keys(errors).length > 0 ? errors : null;
}

/**
 * Formatta un messaggio di errore per display
 */
export function formatError(error) {
  if (typeof error === 'string') return error;
  if (error?.message) return error.message;
  return 'Si è verificato un errore';
}
