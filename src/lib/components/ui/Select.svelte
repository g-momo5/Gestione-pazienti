<script>
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let label = '';
  export let value = '';
  export let options = []; // Array of {value, label}
  export let error = '';
  export let required = false;
  export let disabled = false;
  export let placeholder = 'Seleziona...';

  const handleChange = (e) => {
    value = e.target.value;
    dispatch('change', { value });
  };
</script>

<div class="w-full">
  {#if label}
    <label class="block text-sm font-semibold text-textPrimary mb-1">
      {label}
      {#if required}
        <span class="text-error">*</span>
      {/if}
    </label>
  {/if}

  <select
    {required}
    {disabled}
    bind:value
    on:change={handleChange}
    on:blur
    class="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 transition-all appearance-none
           {error
             ? 'border-error focus:ring-error'
             : 'border-gray-300 focus:ring-primary focus:border-primary'}
           {disabled ? 'bg-gray-100 cursor-not-allowed' : 'bg-white cursor-pointer'}"
  >
    {#if placeholder}
      <option value="" disabled selected={!value}>{placeholder}</option>
    {/if}

    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  {#if error}
    <p class="mt-1 text-sm text-error">{error}</p>
  {/if}
</div>

<style>
  select {
    background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3e%3c/svg%3e");
    background-position: right 0.5rem center;
    background-repeat: no-repeat;
    background-size: 1.5em 1.5em;
    padding-right: 2.5rem;
  }
</style>
