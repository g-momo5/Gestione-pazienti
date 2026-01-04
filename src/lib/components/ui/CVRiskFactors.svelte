<script>
  import { CV_RISK_FACTORS } from '../../constants.js';
  import Checkbox from './Checkbox.svelte';

  export let label = 'Fattori di rischio CV';
  export let selectedFactors = [];

  function toggleFactor(factor) {
    if (selectedFactors.includes(factor)) {
      selectedFactors = selectedFactors.filter(f => f !== factor);
    } else {
      selectedFactors = [...selectedFactors, factor];
    }
  }

  $: isSelected = (factor) => selectedFactors.includes(factor);
</script>

<div class="space-y-3">
  {#if label}
    <p class="font-semibold text-textPrimary">{label}</p>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
    {#each CV_RISK_FACTORS as factor}
      <div class="flex items-center">
        <input
          type="checkbox"
          id={factor}
          checked={isSelected(factor)}
          on:change={() => toggleFactor(factor)}
          class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary/20"
        />
        <label
          for={factor}
          class="ml-2 text-sm text-textPrimary cursor-pointer select-none"
        >
          {factor}
        </label>
      </div>
    {/each}
  </div>
</div>
