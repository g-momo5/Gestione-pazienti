<script>
  import IconBadge from './ui/IconBadge.svelte';

  export let icon = '';
  export let title = '';
  export let collapsible = false;
  export let collapsed = false;

  const toggleCollapse = () => {
    if (collapsible) {
      collapsed = !collapsed;
    }
  };
</script>

<div
  class="flex items-center gap-3 mb-4 pb-2 border-b-2 border-primary/20
         {collapsible ? 'cursor-pointer hover:bg-gray-50 -mx-2 px-2 py-2 rounded-lg' : ''}"
  on:click={toggleCollapse}
  on:keydown={(e) => e.key === 'Enter' && toggleCollapse()}
  role={collapsible ? 'button' : undefined}
  tabindex={collapsible ? 0 : undefined}
>
{#if icon}
    <IconBadge icon={icon} size="sm" />
  {/if}

  <h3 class="text-lg font-semibold text-textPrimary flex-1">
    {title}
  </h3>

  {#if collapsible}
    <span class="text-textSecondary transition-transform {collapsed ? '' : 'rotate-180'}">
      ▼
    </span>
  {/if}
</div>

{#if !collapsed}
  <div class="animate-slide-in">
    <slot />
  </div>
{/if}
