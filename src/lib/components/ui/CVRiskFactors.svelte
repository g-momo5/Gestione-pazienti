<script>
  import { CV_RISK_FACTORS } from '../../constants.js';
  import Checkbox from './Checkbox.svelte';

  export let label = 'Fattori di rischio CV';
  export let selectedFactors = [];

  const SMOKING_BASE = 'Fumo di sigaretta';
  const SMOKING_CURRENT = 'Fumo di sigaretta (attuale)';
  const SMOKING_FORMER = 'Fumo di sigaretta (ex)';
  const SMOKING_VALUES = [SMOKING_BASE, SMOKING_CURRENT, SMOKING_FORMER];
  const smokingRadioName = `smoking-status-${Math.random().toString(36).slice(2, 9)}`;

  const isSmokingValue = (value) => String(value || '').toLowerCase().includes('fumo');
  const isSmokingFactor = (factor) => isSmokingValue(factor);

  const normalizeFactors = (value) => (Array.isArray(value) ? value : value ? [String(value)] : []);

  let normalizedFactors = [];
  let smokingSelected = false;
  let smokingStatus = null;

  $: normalizedFactors = normalizeFactors(selectedFactors);
  $: smokingSelected = normalizedFactors.some((factor) => isSmokingValue(factor));
  $: {
    const lower = normalizedFactors.map((f) => String(f || '').toLowerCase());
    if (lower.some((f) => f.includes('attuale'))) {
      smokingStatus = 'current';
    } else if (lower.some((f) => f.includes('(ex)') || f.includes('ex fumat'))) {
      smokingStatus = 'former';
    } else if (normalizedFactors.includes(SMOKING_CURRENT)) {
      smokingStatus = 'current';
    } else if (normalizedFactors.includes(SMOKING_FORMER)) {
      smokingStatus = 'former';
    } else {
      smokingStatus = null;
    }
  }

  function toggleFactor(factor) {
    if (isSmokingFactor(factor)) {
      if (smokingSelected) {
        selectedFactors = normalizedFactors.filter((f) => !isSmokingValue(f));
      } else {
        selectedFactors = [
          ...normalizedFactors.filter((f) => !isSmokingValue(f)),
          SMOKING_CURRENT,
        ];
      }
      return;
    }
    if (normalizedFactors.includes(factor)) {
      selectedFactors = normalizedFactors.filter(f => f !== factor);
    } else {
      selectedFactors = [...normalizedFactors, factor];
    }
  }

  function setSmokingStatus(status) {
    const base = normalizedFactors.filter((f) => !isSmokingValue(f));
    if (status === 'current') {
      selectedFactors = [...base, SMOKING_CURRENT];
    } else if (status === 'former') {
      selectedFactors = [...base, SMOKING_FORMER];
    } else {
      selectedFactors = base;
    }
  }

  $: if (smokingSelected && !smokingStatus) {
    setSmokingStatus('current');
  }
</script>

<div class="space-y-3">
  {#if label}
    <p class="font-semibold text-textPrimary">{label}</p>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
    {#each CV_RISK_FACTORS as factor}
      <div class="flex flex-col">
        <Checkbox
          checked={isSmokingFactor(factor) ? smokingSelected : normalizedFactors.includes(factor)}
          label={factor}
          on:change={() => toggleFactor(factor)}
        />

        {#if isSmokingFactor(factor) && smokingSelected}
          <div class="ml-6 mt-2 flex flex-wrap gap-4 text-sm text-textSecondary">
            <label class="inline-flex items-center gap-2">
              <input
                type="radio"
                name={smokingRadioName}
                checked={smokingStatus === 'current'}
                on:change={() => setSmokingStatus('current')}
                class="h-4 w-4 border-gray-300 text-primary focus:ring-2 focus:ring-primary/20"
              />
              Ancora fumatore
            </label>
            <label class="inline-flex items-center gap-2">
              <input
                type="radio"
                name={smokingRadioName}
                checked={smokingStatus === 'former'}
                on:change={() => setSmokingStatus('former')}
                class="h-4 w-4 border-gray-300 text-primary focus:ring-2 focus:ring-primary/20"
              />
              Ex fumatore
            </label>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>
