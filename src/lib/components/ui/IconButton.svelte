<script>
  import Icon from './Icon.svelte';

  export let icon = '';
  export let label = '';
  export let variant = 'outline'; // 'outline' | 'ghost'
  export let size = 'sm'; // 'sm' | 'md'
  export let disabled = false;
  export let preventFocusLoss = false;
  export let className = '';
  export { className as class };

  const baseClasses =
    'inline-flex items-center justify-center rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-primary/20 disabled:opacity-50 disabled:cursor-not-allowed';

  const variantClasses = {
    outline: 'border border-gray-200 bg-surface text-textPrimary hover:bg-surface-stronger',
    ghost: 'text-textPrimary hover:bg-surface-stronger',
  };

  const sizeClasses = {
    sm: 'w-8 h-8',
    md: 'w-9 h-9',
  };

  const iconSizes = {
    sm: 16,
    md: 18,
  };

  const handleMouseDown = (event) => {
    if (preventFocusLoss) {
      event.preventDefault();
    }
  };

  $: classes = `${baseClasses} ${variantClasses[variant] ?? variantClasses.outline} ${
    sizeClasses[size] ?? sizeClasses.sm
  } ${className}`;

  $: iconSize = iconSizes[size] ?? iconSizes.sm;
</script>

<button
  type="button"
  {disabled}
  class={classes}
  aria-label={label}
  title={label}
  on:click
  on:mousedown={handleMouseDown}
>
  <Icon name={icon} size={iconSize} />
</button>
