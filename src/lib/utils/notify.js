import { showToast } from '../stores/toastStore.js';

export const notifySuccess = (message, duration) =>
  showToast(message, 'success', duration);

export const notifyError = (err, fallback = 'Errore imprevisto', duration) => {
  const msg =
    typeof err === 'string'
      ? err
      : err?.message || fallback;
  showToast(msg, 'error', duration);
};
