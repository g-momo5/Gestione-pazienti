<script>
  export let active = false;
  export let version = '';
  export let downloadedBytes = 0;
  export let totalBytes = null;
  export let percent = 0;

  const clampPercent = (value) => {
    const n = Number(value);
    if (!Number.isFinite(n)) return 0;
    return Math.max(0, Math.min(100, n));
  };

  const formatBytes = (value) => {
    const bytes = Number(value || 0);
    if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const exp = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const size = bytes / (1024 ** exp);
    const digits = size >= 100 ? 0 : size >= 10 ? 1 : 2;
    return `${size.toFixed(digits)} ${units[exp]}`;
  };

  $: normalizedPercent = clampPercent(percent);
  $: progressLabel = totalBytes
    ? `${formatBytes(downloadedBytes)} / ${formatBytes(totalBytes)}`
    : `${formatBytes(downloadedBytes)}`;
</script>

{#if active}
  <div class="fixed left-4 bottom-16 z-50 w-[320px] max-w-[calc(100vw-2rem)]">
    <div class="bg-surface/95 border border-gray-200 rounded-lg shadow-card p-3">
      <div class="flex items-center justify-between gap-2 mb-1.5">
        <p class="text-sm font-semibold text-textPrimary">Download aggiornamento</p>
        <p class="text-xs font-semibold text-textSecondary">{Math.round(normalizedPercent)}%</p>
      </div>
      {#if version}
        <p class="text-xs text-textSecondary mb-2">Versione {version}</p>
      {/if}
      <div class="h-2 bg-gray-100 rounded-full overflow-hidden">
        <div
          class="h-full bg-primary transition-[width] duration-150 ease-out"
          style={`width: ${normalizedPercent}%`}
        ></div>
      </div>
      <p class="text-xs text-textSecondary mt-2">{progressLabel}</p>
    </div>
  </div>
{/if}
