<script>
  import Card from './Card.svelte';
  import IconBadge from './IconBadge.svelte';
  import Icon from './Icon.svelte';

  export let title = '';
  export let icon = 'info';
  export let collapsed = false;
  export let padding = 'lg';

  const toggle = () => {
    collapsed = !collapsed;
  };
</script>

<Card padding={padding} class="shadow-card">
  <div class="flex items-center justify-between mb-4 gap-3">
    <div class="flex items-center gap-2">
      <button
        class="text-textSecondary hover:text-textPrimary transition-colors"
        aria-label={`Collassa sezione ${title}`}
        on:click={toggle}
      >
        <Icon name="chevronRight" size={18} class={`transition-transform ${collapsed ? '' : 'rotate-90'}`} />
      </button>
      <IconBadge icon={icon} tone="primary" />
      <h2 class="text-xl font-semibold">{title}</h2>
    </div>
    <div class="flex items-center gap-2">
      <slot name="actions" />
    </div>
  </div>

  {#if !collapsed}
    <slot />
  {/if}
</Card>
