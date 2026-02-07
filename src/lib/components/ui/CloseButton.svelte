<script>
  export let size = 'md'; // 'sm' | 'md' | 'lg'
  export let ariaLabel = 'Chiudi';
  export let disabled = false;
  export let className = '';
  export { className as class };

  const sizeClasses = {
    sm: 'text-xl',
    md: 'text-2xl',
    lg: 'text-3xl',
  };

  $: classes = `close-x ${sizeClasses[size] || sizeClasses.md} ${className}`;
</script>

<button
  type="button"
  class={classes}
  aria-label={ariaLabel}
  {disabled}
  on:click
>
  <span aria-hidden="true">×</span>
</button>
