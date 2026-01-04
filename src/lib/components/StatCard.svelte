<script>
  import Card from './ui/Card.svelte';
  import IconBadge from './ui/IconBadge.svelte';

  export let title = '';
  export let value = '';
  export let icon = '';
  export let color = 'primary'; // 'primary', 'success', 'error', 'secondary'

  const colorClasses = {
    primary: 'text-primary bg-primary/10',
    success: 'text-success bg-success/10',
    error: 'text-error bg-error/10',
    secondary: 'text-secondary bg-secondary/10',
  };
</script>

<Card padding="lg">
  <div class="flex flex-col items-center text-center">
    {#if icon}
      <IconBadge icon={icon} size="lg" tone={color} class="mb-3" />
    {/if}

    <div class="text-3xl font-bold {`text-${color}`} mb-1">
      {value}
    </div>

    <div class="text-sm text-textSecondary font-medium">
      {title}
    </div>
  </div>
</Card>
