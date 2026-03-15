import { writable } from 'svelte/store';

const INITIAL_STATUS = {
  state: 'idle',
  currentVersion: '',
  availableVersion: null,
  notes: null,
  publishedAt: null,
  downloadUrl: null,
  installerPath: null,
  deferred: false,
  updateAvailable: false,
  shouldPromptDownload: false,
  shouldPromptInstall: false,
  lastCheck: null,
  lastError: null,
};

const INITIAL_PROGRESS = {
  active: false,
  version: '',
  downloadedBytes: 0,
  totalBytes: null,
  percent: 0,
};

export const updateStatus = writable({ ...INITIAL_STATUS });
export const updateDownloadProgress = writable({ ...INITIAL_PROGRESS });

export function setUpdateStatus(status = {}) {
  updateStatus.set({
    ...INITIAL_STATUS,
    ...status,
  });
}

export function setUpdateDownloadProgress(progress = {}) {
  updateDownloadProgress.set({
    ...INITIAL_PROGRESS,
    active: true,
    ...progress,
  });
}

export function resetUpdateDownloadProgress() {
  updateDownloadProgress.set({ ...INITIAL_PROGRESS });
}
