/**
 * Formatta una data nel formato italiano (DD/MM/YYYY)
 */
export function formatDateIT(dateString) {
  if (!dateString) return '';

  const date = new Date(dateString);
  if (isNaN(date.getTime())) return '';

  const day = String(date.getDate()).padStart(2, '0');
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const year = date.getFullYear();

  return `${day}/${month}/${year}`;
}

/**
 * Formatta una data nel formato ISO (YYYY-MM-DD) per input HTML
 */
export function formatDateISO(dateString) {
  if (!dateString) return '';

  const date = new Date(dateString);
  if (isNaN(date.getTime())) return '';

  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');

  return `${year}-${month}-${day}`;
}

/**
 * Formatta un orario (HH:MM)
 */
export function formatTime(timeString) {
  if (!timeString) return '';

  // Se già in formato HH:MM, ritorna così
  if (/^\d{2}:\d{2}$/.test(timeString)) {
    return timeString;
  }

  // Altrimenti prova a parsare
  const time = new Date(`2000-01-01T${timeString}`);
  if (isNaN(time.getTime())) return '';

  const hours = String(time.getHours()).padStart(2, '0');
  const minutes = String(time.getMinutes()).padStart(2, '0');

  return `${hours}:${minutes}`;
}

/**
 * Calcola l'età in anni completi da una data di nascita
 */
export function calculateAge(birthDateString) {
  if (!birthDateString) return null;

  const birth = new Date(birthDateString);
  if (isNaN(birth.getTime())) return null;

  const today = new Date();
  let age = today.getFullYear() - birth.getFullYear();
  const monthDiff = today.getMonth() - birth.getMonth();

  if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birth.getDate())) {
    age--;
  }

  return age;
}

/**
 * Calcola la durata in minuti tra due orari
 */
export function calculateDurationMinutes(startTime, endTime) {
  if (!startTime || !endTime) return null;

  const [startH, startM] = startTime.split(':').map(Number);
  const [endH, endM] = endTime.split(':').map(Number);

  if (isNaN(startH) || isNaN(startM) || isNaN(endH) || isNaN(endM)) {
    return null;
  }

  const startMinutes = startH * 60 + startM;
  const endMinutes = endH * 60 + endM;

  return endMinutes - startMinutes;
}

/**
 * Formatta la durata in formato leggibile (es: "1h 30min" o "45min")
 */
export function formatDuration(minutes) {
  if (minutes === null || minutes === undefined) return '';

  if (minutes < 60) {
    return `${minutes} min`;
  }

  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;

  if (mins === 0) {
    return `${hours}h`;
  }

  return `${hours}h ${mins}min`;
}

/**
 * Ottiene la data di oggi in formato ISO
 */
export function getTodayISO() {
  return formatDateISO(new Date());
}

/**
 * Ottiene l'orario attuale in formato HH:MM
 */
export function getCurrentTime() {
  const now = new Date();
  const hours = String(now.getHours()).padStart(2, '0');
  const minutes = String(now.getMinutes()).padStart(2, '0');
  return `${hours}:${minutes}`;
}

/**
 * Verifica se una data è nel futuro
 */
export function isFutureDate(dateString) {
  if (!dateString) return false;

  const date = new Date(dateString);
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  return date > today;
}

/**
 * Formatta timestamp per display (es: "15/11/2024 14:30")
 */
export function formatDateTime(dateString, timeString) {
  const datePart = formatDateIT(dateString);
  const timePart = formatTime(timeString);

  if (datePart && timePart) {
    return `${datePart} ${timePart}`;
  } else if (datePart) {
    return datePart;
  }

  return '';
}

/**
 * Capitalizza la prima lettera di ogni parola
 * Es: "mario rossi" -> "Mario Rossi"
 */
export function capitalizeWords(text) {
  if (!text) return '';

  return text
    .toLowerCase()
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}
