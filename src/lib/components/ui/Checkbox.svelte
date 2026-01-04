<script>
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let label = '';
  export let checked = false;
  export let disabled = false;

  const handleChange = (e) => {
    checked = e.target.checked;
    dispatch('change', { checked });
  };
</script>

<label class="flex items-center cursor-pointer {disabled ? 'opacity-50 cursor-not-allowed' : ''}">
  <input
    type="checkbox"
    {checked}
    {disabled}
    on:change={handleChange}
    class="w-5 h-5 text-primary border-gray-300 rounded focus:ring-primary focus:ring-2 cursor-pointer
           {disabled ? 'cursor-not-allowed' : ''}"
  />

  {#if label}
    <span class="ml-2 text-sm font-medium text-textPrimary select-none">
      {label}
    </span>
  {/if}
</label>
