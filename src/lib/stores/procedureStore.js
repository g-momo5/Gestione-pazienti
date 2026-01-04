import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

// State principale
export const procedures = writable([]);
export const loading = writable(false);
export const error = writable(null);
export const selectedProcedure = writable(null);

// Filtri
export const filters = writable({
  search_query: '',
  tipo_valvola: 'all',
  period: 'all',
});

// Derived stores
export const procedureCount = derived(procedures, ($procedures) => $procedures.length);

export const filteredProcedures = derived(
  [procedures, filters],
  ([$procedures, $filters]) => {
    let filtered = [...$procedures];

    // Filtro ricerca testuale
    if ($filters.search_query && $filters.search_query.trim() !== '') {
      const query = $filters.search_query.toLowerCase();
      filtered = filtered.filter((proc) => {
        const nome = (proc.nome || '').toLowerCase();
        const cognome = (proc.cognome || '').toLowerCase();
        const modello = (proc.modello_valvola || '').toLowerCase();

        return nome.includes(query) || cognome.includes(query) || modello.includes(query);
      });
    }

    // Filtro tipo valvola
    if ($filters.tipo_valvola && $filters.tipo_valvola !== 'all') {
      filtered = filtered.filter((proc) => proc.tipo_valvola === $filters.tipo_valvola);
    }

    // Filtro periodo
    if ($filters.period && $filters.period !== 'all') {
      const now = new Date();
      const cutoffDate = new Date();

      switch ($filters.period) {
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
      }

      filtered = filtered.filter((proc) => {
        const procDate = new Date(proc.data_procedura);
        return procDate >= cutoffDate;
      });
    }

    return filtered;
  }
);

// Actions

/**
 * Carica tutte le procedure dal backend
 */
export async function loadProcedures() {
  try {
    loading.set(true);
    error.set(null);

    const result = await invoke('get_all_procedures', { filters: null });
    procedures.set(result);

    return { success: true };
  } catch (err) {
    console.error('Error loading procedures:', err);
    error.set(err.message || 'Errore durante il caricamento delle procedure');
    return { success: false, error: err };
  } finally {
    loading.set(false);
  }
}

/**
 * Carica una procedura singola per ID
 */
export async function loadProcedure(id) {
  try {
    loading.set(true);
    error.set(null);

    const result = await invoke('get_procedure_by_id', { id });
    selectedProcedure.set(result);

    return { success: true, data: result };
  } catch (err) {
    console.error('Error loading procedure:', err);
    error.set(err.message || 'Errore durante il caricamento della procedura');
    return { success: false, error: err };
  } finally {
    loading.set(false);
  }
}

/**
 * Crea una nuova procedura
 */
export async function createProcedure(procedureData) {
  try {
    loading.set(true);
    error.set(null);

    const id = await invoke('create_procedure', { procedure: procedureData });

    // Ricarica la lista
    await loadProcedures();

    return { success: true, id };
  } catch (err) {
    console.error('Error creating procedure:', err);
    error.set(err.message || 'Errore durante la creazione della procedura');
    return { success: false, error: err };
  } finally {
    loading.set(false);
  }
}

/**
 * Aggiorna una procedura esistente
 */
export async function updateProcedure(procedureData) {
  try {
    loading.set(true);
    error.set(null);

    await invoke('update_procedure', { procedure: procedureData });

    // Ricarica la lista
    await loadProcedures();

    return { success: true };
  } catch (err) {
    console.error('Error updating procedure:', err);
    error.set(err.message || 'Errore durante l\'aggiornamento della procedura');
    return { success: false, error: err };
  } finally {
    loading.set(false);
  }
}

/**
 * Elimina una procedura
 */
export async function deleteProcedure(id) {
  try {
    loading.set(true);
    error.set(null);

    await invoke('delete_procedure', { id });

    // Ricarica la lista
    await loadProcedures();

    return { success: true };
  } catch (err) {
    console.error('Error deleting procedure:', err);
    error.set(err.message || 'Errore durante l\'eliminazione della procedura');
    return { success: false, error: err };
  } finally {
    loading.set(false);
  }
}

/**
 * Calcola le statistiche
 */
export async function calculateStatistics(filterOptions = null) {
  try {
    loading.set(true);
    error.set(null);

    const stats = await invoke('calculate_statistics', { filters: filterOptions });

    return { success: true, data: stats };
  } catch (err) {
    console.error('Error calculating statistics:', err);
    error.set(err.message || 'Errore durante il calcolo delle statistiche');
    return { success: false, error: err };
  } finally {
    loading.set(false);
  }
}

/**
 * Ottieni il conteggio delle procedure
 */
export async function getProcedureCount() {
  try {
    const count = await invoke('get_procedure_count');
    return count;
  } catch (err) {
    console.error('Error getting procedure count:', err);
    return 0;
  }
}

/**
 * Aggiorna i filtri
 */
export function updateFilters(newFilters) {
  filters.update((current) => ({ ...current, ...newFilters }));
}

/**
 * Reset filtri
 */
export function resetFilters() {
  filters.set({
    search_query: '',
    tipo_valvola: 'all',
    period: 'all',
  });
}

/**
 * Clear error
 */
export function clearError() {
  error.set(null);
}

/**
 * Clear selected procedure
 */
export function clearSelectedProcedure() {
  selectedProcedure.set(null);
}
