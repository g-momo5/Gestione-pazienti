<script>
  export let variant = 'primary'; // 'primary', 'secondary', 'text', 'danger'
  export let size = 'md'; // 'sm', 'md', 'lg'
  export let disabled = false;
  export let type = 'button';
  export let fullWidth = false;
  export let className = '';
  export { className as class };

  const baseClasses = 'font-medium rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';

  const variantClasses = {
    primary: 'border border-primary text-primary bg-transparent hover:bg-primary/5 focus:ring-primary/30',
    secondary: 'bg-white border border-gray-200 text-textPrimary hover:bg-gray-50 focus:ring-primary/20',
    text: 'text-textPrimary hover:bg-gray-50 focus:ring-primary/20',
    danger: 'bg-transparent border border-error text-error hover:bg-red-50 focus:ring-error/30',
  };

  const sizeClasses = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-base',
    lg: 'px-6 py-3 text-lg',
  };

  $: classes = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${fullWidth ? 'w-full' : ''} ${className}`;
</script>

<button
  {type}
  {disabled}
  class={classes}
  on:click
>
  <slot />
</button>
