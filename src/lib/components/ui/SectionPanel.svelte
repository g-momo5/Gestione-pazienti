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

  const handleKeydown = (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      toggle();
    }
  };
</script>

<Card padding={padding} class="shadow-card">
  <div
    class={`flex items-center justify-between mb-2 gap-2 -mx-2 px-2 py-0.5 rounded-md ${collapsed ? 'cursor-pointer' : 'cursor-pointer hover:bg-surface-stronger'}`}
    role="button"
    tabindex="0"
    aria-expanded={!collapsed}
    aria-label={`Apri sezione ${title}`}
    on:click={toggle}
    on:keydown={handleKeydown}
  >
    <div class="flex items-center gap-2">
      <span class="text-textSecondary transition-colors">
        <Icon name="chevronRight" size={18} class={`transition-transform ${collapsed ? '' : 'rotate-90'}`} />
      </span>
      <IconBadge icon={icon} tone="primary" />
      <h2 class="text-xl font-semibold">{title}</h2>
    </div>
    <div class="flex items-center gap-2" on:click|stopPropagation on:keydown|stopPropagation>
      <slot name="actions" />
    </div>
  </div>

  {#if !collapsed}
    <slot />
  {/if}
</Card>
