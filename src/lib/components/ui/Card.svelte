<script>
  export let padding = 'md'; // 'sm', 'md', 'lg', 'none'
  export let hover = false;
  export let clickable = false;

  // Supporta classi custom aggiuntive
  let className = '';
  export { className as class };

  const paddingClasses = {
    none: '',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6',
  };

  $: classes = `bg-surface rounded-lg border border-gray-200 ${paddingClasses[padding]} ${
    hover ? 'hover:border-gray-300' : ''
  } ${clickable ? 'cursor-pointer' : ''} card ${className}`;
</script>

{#if clickable}
  <div class={classes} on:click on:keydown role="button" tabindex="0">
    <slot />
  </div>
{:else}
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class={classes} on:click on:keydown>
    <slot />
  </div>
{/if}
