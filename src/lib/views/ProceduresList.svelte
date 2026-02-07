<script>
  import { onMount } from 'svelte';
  import Input from '../components/ui/Input.svelte';
  import Select from '../components/ui/Select.svelte';
  import Button from '../components/ui/Button.svelte';
  import Card from '../components/ui/Card.svelte';
  import IconBadge from '../components/ui/IconBadge.svelte';
  import CloseButton from '../components/ui/CloseButton.svelte';
  import {
    procedures,
    filteredProcedures,
    loading,
    loadProcedures,
    deleteProcedure,
    updateFilters,
    resetFilters,
  } from '../stores/procedureStore.js';
  import { exportToExcel } from '../utils/excelExport.js';
  import { formatDateIT, formatTime, calculateAge, calculateDurationMinutes, capitalizeWords } from '../utils/dateUtils.js';
  import { calculateBMI } from '../utils/statistics.js';
  import { VALVE_TYPE_FILTERS, FILTER_PERIODS, ERROR_MESSAGES } from '../constants.js';

  export let onEdit = null;
  export let onNewProcedure = null;

  let searchQuery = '';
  let selectedValveType = 'all';
  let selectedPeriod = 'all';
  let selectedProcedure = null;
  let showDetailModal = false;
  let showDeleteConfirm = false;
  let procedureToDelete = null;

  // Aggiorna filtri quando cambiano
  $: {
    updateFilters({
      search_query: searchQuery,
      tipo_valvola: selectedValveType,
      period: selectedPeriod,
    });
  }

  onMount(async () => {
    await loadProcedures();

    // Gestione tasto Escape per chiudere i modal
    const handleEscape = (e) => {
      if (e.key === 'Escape') {
        if (showDetailModal) {
          showDetailModal = false;
        } else if (showDeleteConfirm) {
          cancelDelete();
        }
      }
    };

    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  });

  function handleViewDetails(proc) {
    selectedProcedure = proc;
    showDetailModal = true;
  }

  function handleEdit(proc) {
    if (onEdit) {
      onEdit(proc);
    }
  }

  function handleDeleteClick(proc) {
    procedureToDelete = proc;
    showDeleteConfirm = true;
  }

  async function confirmDelete() {
    if (procedureToDelete) {
      const result = await deleteProcedure(procedureToDelete.id);
      if (result.success) {
        showDeleteConfirm = false;
        procedureToDelete = null;
      }
    }
  }

  function cancelDelete() {
    showDeleteConfirm = false;
    procedureToDelete = null;
  }

  async function handleExport() {
    const result = await exportToExcel($filteredProcedures);
    if (result.success) {
      alert('Excel esportato con successo!');
    } else if (!result.cancelled) {
      alert('Errore durante l\'export: ' + result.error);
    }
  }

  function handleResetFilters() {
    searchQuery = '';
    selectedValveType = 'all';
    selectedPeriod = 'all';
    resetFilters();
  }
</script>

<div class="max-w-7xl mx-auto">
  <!-- Header con contatore -->
  <div class="flex items-center justify-between mb-6">
    <div>
      <h2 class="text-2xl font-bold text-textPrimary">Elenco Procedure</h2>
      <p class="text-sm text-textSecondary mt-1">
        {$filteredProcedures.length} {$filteredProcedures.length === 1 ? 'procedura trovata' : 'procedure trovate'}
        {#if $filteredProcedures.length !== $procedures.length}
          (di {$procedures.length} totali)
        {/if}
      </p>
    </div>

    {#if onNewProcedure}
      <Button variant="primary" on:click={onNewProcedure}>
        <IconBadge icon="add" size="sm" tone="primary" class="mr-2" /> Nuova Procedura
      </Button>
    {/if}
  </div>

  <!-- Barra Ricerca e Filtri -->
  <Card padding="md" class="mb-6">
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
      <!-- Ricerca -->
      <div class="md:col-span-2">
        <Input
          type="text"
          placeholder="Cerca per nome, cognome o modello valvola..."
          bind:value={searchQuery}
        />
      </div>

      <!-- Filtro Tipo Valvola -->
      <Select
        bind:value={selectedValveType}
        options={VALVE_TYPE_FILTERS}
        placeholder="Tipo valvola"
      />

      <!-- Filtro Periodo -->
      <Select bind:value={selectedPeriod} options={FILTER_PERIODS} placeholder="Periodo" />
    </div>

    <!-- Azioni -->
    <div class="flex gap-3 mt-4 pt-4 border-t">
      <Button variant="text" size="sm" on:click={handleResetFilters}>
        <IconBadge icon="refresh" size="sm" tone="secondary" class="mr-2" /> Reset Filtri
      </Button>

      <Button variant="secondary" size="sm" on:click={handleExport} disabled={$filteredProcedures.length === 0}>
        <IconBadge icon="download" size="sm" tone="secondary" class="mr-2" /> Esporta Excel ({$filteredProcedures.length})
      </Button>
    </div>
  </Card>

  <!-- Lista Procedure -->
  {#if $loading}
    <div class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      <p class="mt-4 text-textSecondary">Caricamento...</p>
    </div>
  {:else if $filteredProcedures.length === 0}
    <Card padding="lg">
      <div class="text-center py-12">
        <IconBadge icon="clipboard" size="lg" class="mx-auto mb-4" />
        <h3 class="text-xl font-semibold text-textPrimary mb-2">Nessuna procedura trovata</h3>
        <p class="text-textSecondary">
          {#if searchQuery || selectedValveType !== 'all' || selectedPeriod !== 'all'}
            Prova a modificare i filtri di ricerca
          {:else}
            Inizia aggiungendo la tua prima procedura TAVI
          {/if}
        </p>
      </div>
    </Card>
  {:else}
    <Card padding="none" class="overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead class="bg-surface-strong border-b border-gray-200">
            <tr>
              <th class="px-4 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                Paziente
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                Età
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                Data Procedura
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                Modello
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                Dimensione
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-textSecondary uppercase tracking-wider">
                Durata
              </th>
              <th class="px-4 py-3 text-right text-xs font-medium text-textSecondary uppercase tracking-wider">
                Azioni
              </th>
            </tr>
          </thead>
          <tbody class="bg-surface divide-y divide-gray-200">
            {#each $filteredProcedures as procedure (procedure.id)}
              <tr
                class="hover:bg-surface-stronger transition-colors animate-fade-in"
              >
                <td class="px-4 py-3 whitespace-nowrap">
                  <div class="font-medium text-textPrimary">
                    {capitalizeWords(procedure.nome)} {capitalizeWords(procedure.cognome)}
                  </div>
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-textSecondary">
                  {calculateAge(procedure.data_nascita)} anni
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-textSecondary">
                  {formatDateIT(procedure.data_procedura)}
                </td>
                <td class="px-4 py-3 text-sm text-textSecondary">
                  {procedure.modello_valvola}
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-textSecondary">
                  {procedure.dimensione_valvola ? `${procedure.dimensione_valvola} mm` : '-'}
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-textSecondary">
                  {calculateDurationMinutes(procedure.ora_inizio, procedure.ora_fine)} min
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-right text-sm">
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="text"
                      size="sm"
                      on:click={(e) => {
                        e.stopPropagation();
                        handleViewDetails(procedure);
                      }}
                    >
                      <IconBadge icon="view" size="sm" tone="neutral" />
                    </Button>
                    <Button
                      variant="text"
                      size="sm"
                      on:click={(e) => {
                        e.stopPropagation();
                        handleEdit(procedure);
                      }}
                    >
                      <IconBadge icon="edit" size="sm" tone="neutral" />
                    </Button>
                    <Button
                      variant="text"
                      size="sm"
                      on:click={(e) => {
                        e.stopPropagation();
                        handleDeleteClick(procedure);
                      }}
                    >
                      <IconBadge icon="delete" size="sm" tone="neutral" />
                    </Button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </Card>
  {/if}
</div>

<!-- Modal Dettagli Procedura -->
{#if showDetailModal && selectedProcedure}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50 animate-fade-in"
    on:click={() => (showDetailModal = false)}
  >
    <Card
      padding="none"
      class="max-w-3xl w-full max-h-[90vh] overflow-auto animate-slide-in"
      on:click={(e) => e.stopPropagation()}
    >
      <div class="sticky top-0 bg-surface border-b border-gray-200 p-6 flex items-center justify-between">
        <div>
          <h3 class="text-2xl font-bold text-textPrimary">
            {capitalizeWords(selectedProcedure.nome)} {capitalizeWords(selectedProcedure.cognome)}
          </h3>
          <p class="text-textSecondary mt-1">
            {calculateAge(selectedProcedure.data_nascita)} anni
          </p>
        </div>
        <CloseButton size="md" ariaLabel="Chiudi" on:click={() => (showDetailModal = false)} />
      </div>

      <div class="p-6 space-y-6">
        <!-- Dati Paziente -->
        <div>
          <h4 class="font-semibold text-textPrimary mb-3 flex items-center gap-2">
            <IconBadge icon="clipboard" size="sm" tone="secondary" /> Dati Paziente
          </h4>
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span class="text-textSecondary">Data Nascita:</span>
              <span class="font-medium ml-2">{formatDateIT(selectedProcedure.data_nascita)}</span>
            </div>
            <div>
              <span class="text-textSecondary">Età:</span>
              <span class="font-medium ml-2">{calculateAge(selectedProcedure.data_nascita)} anni</span>
            </div>
            {#if selectedProcedure.altezza}
              <div>
                <span class="text-textSecondary">Altezza:</span>
                <span class="font-medium ml-2">{selectedProcedure.altezza} cm</span>
              </div>
            {/if}
            {#if selectedProcedure.peso}
              <div>
                <span class="text-textSecondary">Peso:</span>
                <span class="font-medium ml-2">{selectedProcedure.peso} kg</span>
              </div>
            {/if}
            {#if selectedProcedure.peso && selectedProcedure.altezza}
              <div>
                <span class="text-textSecondary">BMI:</span>
                <span class="font-medium ml-2">
                  {calculateBMI(selectedProcedure.peso, selectedProcedure.altezza)}
                </span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Dati Pre-procedurali -->
        <div class="border-t pt-4">
          <h4 class="font-semibold text-textPrimary mb-3 flex items-center gap-2">
            <IconBadge icon="heart" size="sm" tone="secondary" /> Dati Pre-procedurali
          </h4>
          <div class="grid grid-cols-2 gap-3 text-sm">
            {#if selectedProcedure.fe}
              <div><span class="text-textSecondary">FE:</span> <span class="font-medium ml-2">{selectedProcedure.fe}%</span></div>
            {/if}
            {#if selectedProcedure.vmax}
              <div><span class="text-textSecondary">Vmax:</span> <span class="font-medium ml-2">{selectedProcedure.vmax} m/s</span></div>
            {/if}
            {#if selectedProcedure.gmax}
              <div><span class="text-textSecondary">Gmax:</span> <span class="font-medium ml-2">{selectedProcedure.gmax} mmHg</span></div>
            {/if}
            {#if selectedProcedure.gmed}
              <div><span class="text-textSecondary">Gmed:</span> <span class="font-medium ml-2">{selectedProcedure.gmed} mmHg</span></div>
            {/if}
            {#if selectedProcedure.ava}
              <div><span class="text-textSecondary">AVA:</span> <span class="font-medium ml-2">{selectedProcedure.ava} cm²</span></div>
            {/if}
            {#if selectedProcedure.anulus_aortico}
              <div><span class="text-textSecondary">Anulus:</span> <span class="font-medium ml-2">{selectedProcedure.anulus_aortico} mm</span></div>
            {/if}
            {#if selectedProcedure.valvola_protesica}
              <div class="col-span-2">
                <span class="text-textSecondary">Valvola Protesica:</span>
                <span class="font-medium ml-2">
                  {selectedProcedure.protesica_modello}
                  {selectedProcedure.protesica_dimensione ? `(${selectedProcedure.protesica_dimensione})` : ''}
                </span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Dati Procedurali -->
        <div class="border-t pt-4">
          <h4 class="font-semibold text-textPrimary mb-3 flex items-center gap-2">
            <IconBadge icon="hospital" size="sm" tone="secondary" /> Dati Procedurali
          </h4>
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span class="text-textSecondary">Data:</span>
              <span class="font-medium ml-2">{formatDateIT(selectedProcedure.data_procedura)}</span>
            </div>
            <div>
              <span class="text-textSecondary">Orario:</span>
              <span class="font-medium ml-2">
                {formatTime(selectedProcedure.ora_inizio)} - {formatTime(selectedProcedure.ora_fine)}
              </span>
            </div>
            <div class="col-span-2">
              <span class="text-textSecondary">Tipo Valvola:</span>
              <span class="font-medium ml-2">{selectedProcedure.tipo_valvola}</span>
            </div>
            <div class="col-span-2">
              <span class="text-textSecondary">Modello:</span>
              <span class="font-medium ml-2">{selectedProcedure.modello_valvola}</span>
            </div>
            {#if selectedProcedure.dimensione_valvola}
              <div>
                <span class="text-textSecondary">Dimensione:</span>
                <span class="font-medium ml-2">{selectedProcedure.dimensione_valvola} mm</span>
              </div>
            {/if}
            <div class="col-span-2 flex gap-4">
              <div>
                <span class="text-textSecondary">Pre-dilatazione:</span>
                <span class="font-medium ml-2">{selectedProcedure.pre_dilatazione ? '✓ Sì' : '✗ No'}</span>
              </div>
              <div>
                <span class="text-textSecondary">Post-dilatazione:</span>
                <span class="font-medium ml-2">{selectedProcedure.post_dilatazione ? '✓ Sì' : '✗ No'}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Azioni -->
        <div class="flex gap-3 border-t pt-4">
          <Button variant="primary" fullWidth on:click={() => { showDetailModal = false; handleEdit(selectedProcedure); }}>
            <IconBadge icon="edit" size="sm" tone="primary" class="mr-2" /> Modifica
          </Button>
          <Button variant="danger" fullWidth on:click={() => { showDetailModal = false; handleDeleteClick(selectedProcedure); }}>
            <IconBadge icon="delete" size="sm" tone="error" class="mr-2" /> Elimina
          </Button>
        </div>
      </div>
    </Card>
  </div>
{/if}

<!-- Modal Conferma Eliminazione -->
{#if showDeleteConfirm && procedureToDelete}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50 animate-fade-in"
    on:click={cancelDelete}
  >
    <Card padding="lg" class="max-w-md animate-slide-in" on:click={(e) => e.stopPropagation()}>
      <h3 class="text-xl font-bold text-textPrimary mb-4">Conferma Eliminazione</h3>
      <p class="text-textSecondary mb-6">
        Sei sicuro di voler eliminare la procedura di <strong
          >{capitalizeWords(procedureToDelete.nome)} {capitalizeWords(procedureToDelete.cognome)}</strong
        >?
        <br />
        <span class="text-error text-sm mt-2 block">Questa azione non può essere annullata.</span>
      </p>

      <div class="flex gap-3">
        <Button variant="secondary" fullWidth on:click={cancelDelete}> Annulla </Button>
        <Button variant="danger" fullWidth on:click={confirmDelete}> Elimina </Button>
      </div>
    </Card>
  </div>
{/if}
