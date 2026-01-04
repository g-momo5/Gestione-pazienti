import { writable } from 'svelte/store';

const DEFAULT_DURATION = 4000;

export const toasts = writable([]);

let counter = 0;

export function showToast(message, type = 'info', duration = DEFAULT_DURATION) {
  const id = ++counter;
  toasts.update((current) => [...current, { id, message, type, duration }]);

  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }

  return id;
}

export function removeToast(id) {
  toasts.update((current) => current.filter((toast) => toast.id !== id));
}
