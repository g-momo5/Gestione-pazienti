<script>
  import { onMount } from 'svelte';
  import Input from '../components/ui/Input.svelte';
  import MaskedDateInput from '../components/ui/MaskedDateInput.svelte';
  import MaskedTimeInput from '../components/ui/MaskedTimeInput.svelte';
  import Select from '../components/ui/Select.svelte';
  import Checkbox from '../components/ui/Checkbox.svelte';
  import Button from '../components/ui/Button.svelte';
  import IconBadge from '../components/ui/IconBadge.svelte';
  import SectionHeader from '../components/SectionHeader.svelte';
  import { createProcedure, updateProcedure } from '../stores/procedureStore.js';
  import { validateProcedureForm } from '../utils/validators.js';
  import { getTodayISO, getCurrentTime, calculateDurationMinutes, capitalizeWords } from '../utils/dateUtils.js';
  import {
    BALLOON_EXPANDABLE_MODELS,
    SELF_EXPANDABLE_MODELS,
    VALVE_TYPES,
    ERROR_MESSAGES,
  } from '../constants.js';

  export let editingProcedure = null;
  export let onSaved = null;
  export let onCancel = null;

  // Form data
  let formData = {
    // Dati Paziente
    nome: '',
    cognome: '',
    data_nascita: '',
    altezza: null,
    peso: null,

    // Dati Pre-procedurali
    fe: null,
    vmax: null,
    gmax: null,
    gmed: null,
    ava: null,
    anulus_aortico: null,
    valvola_protesica: false,
    protesica_modello: '',
    protesica_dimensione: '',

    // Dati Procedurali
    data_procedura: getTodayISO(),
    ora_inizio: getCurrentTime(),
    ora_fine: '',
    tipo_valvola: '',
    modello_valvola: '',
    dimensione_valvola: null,
    pre_dilatazione: false,
    post_dilatazione: false,
  };

  let errors = {};
  let dataNascitaInvalid = false;
  let dataProceduraInvalid = false;
  let oraInizioInvalid = false;
  let oraFineInvalid = false;
  let saving = false;
  let successMessage = '';
  let duration = null;

  // Dropdown options
  $: valveModelOptions = formData.tipo_valvola === VALVE_TYPES.BALLOON
    ? BALLOON_EXPANDABLE_MODELS.map((m) => ({ value: m, label: m }))
    : formData.tipo_valvola === VALVE_TYPES.SELF
    ? SELF_EXPANDABLE_MODELS.map((m) => ({ value: m, label: m }))
    : [];

  // Calcola durata automaticamente
  $: {
    duration = calculateDurationMinutes(formData.ora_inizio, formData.ora_fine);
  }

  // Se cambia il tipo valvola, resetta il modello
  $: {
    if (formData.tipo_valvola) {
      // Se il modello attuale non è nella nuova lista, resettalo
      const validModels = formData.tipo_valvola === VALVE_TYPES.BALLOON
        ? BALLOON_EXPANDABLE_MODELS
        : SELF_EXPANDABLE_MODELS;

      if (!validModels.includes(formData.modello_valvola)) {
        formData.modello_valvola = '';
      }
    }
  }

  onMount(() => {
    // Se stiamo modificando, carica i dati
    if (editingProcedure) {
      formData = { ...editingProcedure };
    }
  });

  async function handleSubmit() {
    // Valida form
    errors = {};
    if (dataNascitaInvalid) errors.data_nascita = ERROR_MESSAGES.invalidDate;
    if (dataProceduraInvalid) errors.data_procedura = ERROR_MESSAGES.invalidDate;
    if (oraInizioInvalid) errors.ora_inizio = ERROR_MESSAGES.invalidTime;
    if (oraFineInvalid) errors.ora_fine = ERROR_MESSAGES.invalidTime;
    if (Object.keys(errors).length > 0) {
      return;
    }
    const validationErrors = validateProcedureForm(formData);

    if (validationErrors) {
      errors = validationErrors;
      return;
    }

    // Sanitizza i dati per Rust/Serde
    // Converti stringhe in numeri e gestisci valori opzionali
    const parseOptionalFloat = (value) => {
      if (value === null || value === undefined || value === '') return undefined;
      const parsed = parseFloat(value);
      return isNaN(parsed) ? undefined : parsed;
    };

    const sanitizedData = {
      ...formData,
      // Capitalizza nome e cognome
      nome: capitalizeWords(formData.nome),
      cognome: capitalizeWords(formData.cognome),
      // Converti stringhe in numeri per campi numerici opzionali
      altezza: parseOptionalFloat(formData.altezza),
      peso: parseOptionalFloat(formData.peso),
      fe: parseOptionalFloat(formData.fe),
      vmax: parseOptionalFloat(formData.vmax),
      gmax: parseOptionalFloat(formData.gmax),
      gmed: parseOptionalFloat(formData.gmed),
      ava: parseOptionalFloat(formData.ava),
      anulus_aortico: parseOptionalFloat(formData.anulus_aortico),
      dimensione_valvola: parseOptionalFloat(formData.dimensione_valvola),
      // Campi stringa opzionali: converti stringa vuota in null
      protesica_modello: formData.protesica_modello || null,
      protesica_dimensione: formData.protesica_dimensione || null,
    };

    // Salva
    saving = true;
    successMessage = '';

    try {
      let result;
      if (editingProcedure?.id) {
        result = await updateProcedure(sanitizedData);
      } else {
        result = await createProcedure(sanitizedData);
      }

      if (result.success) {
        successMessage = ERROR_MESSAGES.saveSuccess;

        // Reset form dopo 1 secondo
        setTimeout(() => {
          resetForm();
          successMessage = '';
          if (onSaved) onSaved();
        }, 1000);
      } else {
        errors.general = result.error?.message || ERROR_MESSAGES.saveError;
      }
    } catch (err) {
      console.error('Errore salvataggio:', err);
      errors.general = err.message || ERROR_MESSAGES.saveError;
    } finally {
      saving = false;
    }
  }

  function resetForm() {
    formData = {
      nome: '',
      cognome: '',
      data_nascita: '',
      altezza: null,
      peso: null,
      fe: null,
      vmax: null,
      gmax: null,
      gmed: null,
      ava: null,
      anulus_aortico: null,
      valvola_protesica: false,
      protesica_modello: '',
      protesica_dimensione: '',
      data_procedura: getTodayISO(),
      ora_inizio: getCurrentTime(),
      ora_fine: '',
      tipo_valvola: '',
      modello_valvola: '',
      dimensione_valvola: null,
      pre_dilatazione: false,
      post_dilatazione: false,
    };
    errors = {};
  }

  function handleCancel() {
    if (onCancel) {
      onCancel();
    } else {
      resetForm();
    }
  }
</script>

<div class="max-w-5xl mx-auto">
  <h2 class="text-2xl font-bold text-textPrimary mb-6">
    {editingProcedure ? 'Modifica Procedura TAVI' : 'Nuova Procedura TAVI'}
  </h2>

  {#if successMessage}
    <div class="mb-6 p-4 bg-success/10 border border-success text-success rounded-lg animate-fade-in">
      ✓ {successMessage}
    </div>
  {/if}

  {#if errors.general}
    <div class="mb-6 p-4 bg-error/10 border border-error text-error rounded-lg">
      ✗ {errors.general}
    </div>
  {/if}

  <form on:submit|preventDefault={handleSubmit} class="space-y-8">
    <!-- SEZIONE 1: DATI PAZIENTE -->
    <div class="bg-surface rounded-lg p-6 shadow-card">
      <SectionHeader icon="clipboard" title="Dati Paziente" />

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Input
          label="Nome"
          bind:value={formData.nome}
          placeholder="Mario"
          required
          error={errors.nome}
        />

        <Input
          label="Cognome"
          bind:value={formData.cognome}
          placeholder="Rossi"
          required
          error={errors.cognome}
        />

        <MaskedDateInput
          label="Data di Nascita"
          bind:value={formData.data_nascita}
          bind:invalid={dataNascitaInvalid}
          required
          error={errors.data_nascita}
        />

        <div></div>

        <Input
          label="Altezza (cm)"
          type="number"
          bind:value={formData.altezza}
          placeholder="170"
          min="100"
          max="250"
          step="0.1"
          error={errors.altezza}
        />

        <Input
          label="Peso (kg)"
          type="number"
          bind:value={formData.peso}
          placeholder="70"
          min="30"
          max="200"
          step="0.1"
          error={errors.peso}
        />
      </div>
    </div>

    <!-- SEZIONE 2: DATI TEMPORALI -->
    <div class="bg-surface rounded-lg p-6 shadow-card">
      <SectionHeader icon="🕐" title="Dati Temporali" />

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <MaskedDateInput
          label="Data Procedura"
          bind:value={formData.data_procedura}
          bind:invalid={dataProceduraInvalid}
          required
          error={errors.data_procedura}
        />

        <MaskedTimeInput
          label="Ora Inizio"
          bind:value={formData.ora_inizio}
          bind:invalid={oraInizioInvalid}
          required
          error={errors.ora_inizio}
        />

        <MaskedTimeInput
          label="Ora Fine"
          bind:value={formData.ora_fine}
          bind:invalid={oraFineInvalid}
          required
          error={errors.ora_fine}
        />
      </div>

      {#if duration !== null && duration > 0}
        <div class="mt-4 p-3 bg-primary/10 text-primary rounded-lg text-sm font-medium flex items-center gap-2">
          <IconBadge icon="timer" size="sm" tone="primary" />
          <span>Durata calcolata: {duration} minuti ({Math.floor(duration / 60)}h {duration % 60}min)</span>
        </div>
      {/if}
    </div>

    <!-- SEZIONE 3: DATI PRE-PROCEDURALI -->
    <div class="bg-surface rounded-lg p-6 shadow-card">
      <SectionHeader icon="heart" title="Dati Pre-procedurali" />

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
        <Input
          label="FE (%)"
          type="number"
          bind:value={formData.fe}
          placeholder="55"
          min="0"
          max="100"
          step="0.1"
          error={errors.fe}
        />

        <Input
          label="Vmax (m/s)"
          type="number"
          bind:value={formData.vmax}
          placeholder="4.2"
          min="0"
          max="10"
          step="0.1"
          error={errors.vmax}
        />

        <Input
          label="Gmax (mmHg)"
          type="number"
          bind:value={formData.gmax}
          placeholder="85"
          min="0"
          max="200"
          step="0.1"
          error={errors.gmax}
        />

        <Input
          label="Gmed (mmHg)"
          type="number"
          bind:value={formData.gmed}
          placeholder="52"
          min="0"
          max="150"
          step="0.1"
          error={errors.gmed}
        />

        <Input
          label="AVA (cm²)"
          type="number"
          bind:value={formData.ava}
          placeholder="0.8"
          min="0"
          max="5"
          step="0.01"
          error={errors.ava}
        />

        <Input
          label="Anulus Aortico (mm)"
          type="number"
          bind:value={formData.anulus_aortico}
          placeholder="25"
          min="15"
          max="35"
          step="0.1"
          error={errors.anulus_aortico}
        />
      </div>

      <!-- Valvola Protesica -->
      <div class="border-t pt-4">
        <Checkbox
          label="Valvola Protesica Presente"
          bind:checked={formData.valvola_protesica}
        />

        {#if formData.valvola_protesica}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4 animate-slide-in">
            <Input
              label="Modello Protesica"
              bind:value={formData.protesica_modello}
              placeholder="Edwards SAPIEN 3"
              required={formData.valvola_protesica}
              error={errors.protesica_modello}
            />

            <Input
              label="Dimensione Protesica"
              bind:value={formData.protesica_dimensione}
              placeholder="26 mm"
              required={formData.valvola_protesica}
              error={errors.protesica_dimensione}
            />
          </div>
        {/if}
      </div>
    </div>

    <!-- SEZIONE 4: DATI PROCEDURALI -->
    <div class="bg-surface rounded-lg p-6 shadow-card">
      <SectionHeader icon="hospital" title="Dati Procedurali" />

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
        <Select
          label="Tipo Valvola"
          bind:value={formData.tipo_valvola}
          options={[
            { value: VALVE_TYPES.BALLOON, label: VALVE_TYPES.BALLOON },
            { value: VALVE_TYPES.SELF, label: VALVE_TYPES.SELF },
          ]}
          placeholder="Seleziona tipo valvola"
          required
          error={errors.tipo_valvola}
        />

        <Select
          label="Modello Valvola"
          bind:value={formData.modello_valvola}
          options={valveModelOptions}
          placeholder={formData.tipo_valvola ? 'Seleziona modello' : 'Prima seleziona il tipo'}
          required
          disabled={!formData.tipo_valvola}
          error={errors.modello_valvola}
        />

        <Input
          label="Dimensione Valvola (mm)"
          type="number"
          bind:value={formData.dimensione_valvola}
          placeholder="26"
          min="15"
          max="35"
          step="0.5"
          error={errors.dimensione_valvola}
        />
      </div>

      <!-- Dilatazioni -->
      <div class="flex flex-col gap-3 border-t pt-4">
        <Checkbox label="Pre-dilatazione" bind:checked={formData.pre_dilatazione} />
        <Checkbox label="Post-dilatazione" bind:checked={formData.post_dilatazione} />
      </div>
    </div>

    <!-- AZIONI -->
    <div class="flex justify-end gap-4 sticky bottom-0 bg-background py-4 border-t">
      <Button variant="secondary" on:click={handleCancel} disabled={saving}>
        Annulla
      </Button>

      <Button type="submit" variant="primary" disabled={saving}>
        {#if saving}
          Salvataggio...
        {:else}
          💾 Salva Procedura
        {/if}
      </Button>
    </div>
  </form>
</div>
