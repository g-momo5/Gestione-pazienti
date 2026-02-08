<script>
  import Icon from './Icon.svelte';

  export let onClick = () => {};
  export let ariaLabel = 'Torna alla home';
  export let size = 22;
  export let className = '';
  export { className as class };
</script>

<button
  type="button"
  class={`back-circle-button back-circle ${className}`}
  aria-label={ariaLabel}
  on:click={onClick}
>
  <Icon name="arrowLeft" size={size} class="text-current" />
</button>
