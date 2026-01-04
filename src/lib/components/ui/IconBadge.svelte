<script>
  import Icon from './Icon.svelte';

  export let icon = '';
  export let size = 'md'; // 'sm' | 'md' | 'lg' | 'xl'
  export let tone = 'primary'; // 'primary' | 'secondary' | 'neutral' | 'success' | 'error' | 'warning'

  const sizeMap = { sm: 20, md: 24, lg: 28, xl: 34 };
  const toneClasses = {
    primary: 'text-primary',
    secondary: 'text-secondary',
    neutral: 'text-textSecondary',
    success: 'text-success',
    error: 'text-error',
    warning: 'text-warning',
  };

  // Allow passing extra classes
  export let className = '';
  export { className as class };
</script>

<Icon name={icon} size={sizeMap[size] ?? 22} class={`${toneClasses[tone]} ${className}`} />
