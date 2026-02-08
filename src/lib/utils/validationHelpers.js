import { showToast } from '../stores/toastStore.js';

export const requireCondition = (condition, message, type = 'warning') => {
  if (!condition) {
    showToast(message, type);
    return false;
  }
  return true;
};

export const requireValue = (value, message, type = 'warning') =>
  requireCondition(!!value, message, type);

export const requireValid = (isInvalid, message, type = 'warning') =>
  requireCondition(!isInvalid, message, type);
