<script>
  import { fly } from 'svelte/transition';
  import { toasts, removeToast } from '../../stores/toastStore.js';
  import IconBadge from './IconBadge.svelte';
  import CloseButton from './CloseButton.svelte';

  const toneMap = {
    success: { border: 'border-l-4 border-success', text: 'text-textPrimary', icon: 'check', iconTone: 'success' },
    error: { border: 'border-l-4 border-error', text: 'text-textPrimary', icon: 'close', iconTone: 'error' },
    warning: { border: 'border-l-4 border-warning', text: 'text-textPrimary', icon: 'alert', iconTone: 'warning' },
    info: { border: 'border-l-4 border-primary', text: 'text-textPrimary', icon: 'info', iconTone: 'primary' },
  };
</script>

<div class="fixed top-3 right-4 z-50 space-y-2.5 w-80 max-w-full">
  {#each $toasts as toast (toast.id)}
    {#if toneMap[toast.type] || toneMap.info}
      {#if toneMap[toast.type]}
        <div
          class={`flex items-start gap-2.5 p-3 rounded-lg shadow-card bg-surface/95 ${toneMap[toast.type].text} ${toneMap[toast.type].border}`}
          role="status"
          in:fly={{ x: 120, duration: 260, opacity: 0.25 }}
          out:fly={{ x: 120, duration: 220, opacity: 0 }}
        >
          <IconBadge icon={toneMap[toast.type].icon} tone={toneMap[toast.type].iconTone} size="sm" />
          <div class="flex-1 text-sm leading-snug text-textPrimary">{toast.message}</div>
          <CloseButton
            size="sm"
            ariaLabel="Chiudi notifica"
            className="opacity-70 hover:opacity-100"
            on:click={() => removeToast(toast.id)}
          />
        </div>
      {:else}
        <div
          class={`flex items-start gap-2.5 p-3 rounded-lg shadow-card bg-surface/95 ${toneMap.info.text} ${toneMap.info.border}`}
          role="status"
          in:fly={{ x: 120, duration: 260, opacity: 0.25 }}
          out:fly={{ x: 120, duration: 220, opacity: 0 }}
        >
          <IconBadge icon="info" tone="primary" size="sm" />
          <div class="flex-1 text-sm leading-snug text-textPrimary">{toast.message}</div>
          <CloseButton
            size="sm"
            ariaLabel="Chiudi notifica"
            className="opacity-70 hover:opacity-100"
            on:click={() => removeToast(toast.id)}
          />
        </div>
      {/if}
    {/if}
  {/each}
</div>
