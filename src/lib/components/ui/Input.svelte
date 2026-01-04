<script>
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let label = '';
  export let value = '';
  export let type = 'text';
  export let placeholder = '';
  export let error = '';
  export let required = false;
  export let disabled = false;
  export let min = null;
  export let max = null;
  export let step = null;

  const handleInput = (e) => {
    value = e.target.value;
    dispatch('input', e);
  };

  const handleFocus = (e) => {
    dispatch('focus', e);
  };

  const handleBlur = (e) => {
    dispatch('blur', e);
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

  <input
    {type}
    {placeholder}
    {required}
    {disabled}
    {min}
    {max}
    {step}
    value={value}
    on:input={handleInput}
    on:focus={handleFocus}
    on:blur={handleBlur}
    on:change
    class="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 transition-all
           {error
             ? 'border-error focus:ring-error/30'
             : 'border-gray-200 focus:ring-primary/20 focus:border-primary'}
           {disabled ? 'bg-gray-50 cursor-not-allowed text-textSecondary' : 'bg-white'}"
  />

  {#if error}
    <p class="mt-1 text-sm text-error">{error}</p>
  {/if}
</div>

<style>
  /* Chrome, Safari, Edge, Opera */
  input::-webkit-outer-spin-button,
  input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  /* Firefox */
  input[type='number'] {
    -moz-appearance: textfield;
  }
</style>
