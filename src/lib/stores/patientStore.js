import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

// State stores
export const patients = writable([]);
export const loading = writable(false);
export const error = writable(null);
export const selectedPatient = writable(null);
export const statusCounts = writable([]);

// Load all patients
export async function loadPatients() {
  loading.set(true);
  error.set(null);

  try {
    const data = await invoke('get_all_patients', { filters: null });
    patients.set(data);
  } catch (err) {
    error.set(err);
    console.error('Error loading patients:', err);
  } finally {
    loading.set(false);
  }
}

// Load patient by ID
export async function loadPatient(id) {
  loading.set(true);
  error.set(null);
  selectedPatient.set(null);

  try {
    const patient = await invoke('get_patient_by_id', { id });
    selectedPatient.set(patient);
    return patient;
  } catch (err) {
    error.set(err);
    console.error('Error loading patient:', err);
  } finally {
    loading.set(false);
  }
}

// Create new patient
export async function createPatient(patientData) {
  loading.set(true);
  error.set(null);

  try {
    const id = await invoke('create_patient', { patient: patientData });
    await Promise.all([loadPatients(), loadStatusCounts()]); // Reload list + counts
    return id;
  } catch (err) {
    error.set(err);
    console.error('Error creating patient:', err);
    throw err;
  } finally {
    loading.set(false);
  }
}

// Update patient
export async function updatePatient(patientData) {
  loading.set(true);
  error.set(null);

  try {
    await invoke('update_patient', { patient: patientData });
    await Promise.all([loadPatients(), loadStatusCounts()]);
  } catch (err) {
    error.set(err);
    console.error('Error updating patient:', err);
    throw err;
  } finally {
    loading.set(false);
  }
}

// Delete patient
export async function deletePatient(id) {
  loading.set(true);
  error.set(null);

  try {
    await invoke('delete_patient', { id });
    await Promise.all([loadPatients(), loadStatusCounts()]); // Reload list + counts
  } catch (err) {
    error.set(err);
    console.error('Error deleting patient:', err);
    throw err;
  } finally {
    loading.set(false);
  }
}

// Change patient status
export async function changePatientStatus(patientId, newStatus) {
  loading.set(true);
  error.set(null);

  try {
    await invoke('change_patient_status', { patientId, newStatus });
    await loadPatients(); // Reload list
    await loadStatusCounts(); // Update counts
  } catch (err) {
    error.set(err);
    console.error('Error changing patient status:', err);
    throw err;
  } finally {
    loading.set(false);
  }
}

// Load status counts
export async function loadStatusCounts() {
  try {
    const counts = await invoke('get_patient_status_counts');
    statusCounts.set(counts);
  } catch (err) {
    error.set(err);
    console.error('Error loading status counts:', err);
  }
}

// Get patients by status
export async function getPatientsByStatus(status) {
  try {
    return await invoke('get_patients_by_status', { status });
  } catch (err) {
    error.set(err);
    console.error('Error loading patients by status:', err);
    return [];
  }
}

// Derived stores
export const patientsByStatus = derived(patients, $patients => {
  const grouped = {
    'Da valutare': [],
    'In corso di accertamenti': [],
    'In attesa di TAVI': [],
    'Non candidabile a TAVI': [],
    'TAVI eseguita': []
  };

  $patients.forEach(p => {
    if (grouped[p.status]) {
      grouped[p.status].push(p);
    }
  });

  return grouped;
});
