<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import IconButton from './IconButton.svelte';
  import Card from './Card.svelte';
  import Button from './Button.svelte';
  import CloseButton from './CloseButton.svelte';

  export let label = '';
  export let value = '';
  export let placeholder = '';
  export let storageKey = '';
  export let disabled = false;
  export let id = '';

  const fallbackId = `preset-textarea-${Math.random().toString(36).slice(2, 9)}`;
  $: textareaId = id || fallbackId;
  let textareaRef;
  let presets = [];
  let showPresets = false;
  let selectionStart = null;
  let selectionEnd = null;

  const loadPresets = () => {
    if (!storageKey || typeof localStorage === 'undefined') return;
    try {
      const raw = localStorage.getItem(storageKey);
      const parsed = raw ? JSON.parse(raw) : [];
      presets = Array.isArray(parsed) ? parsed : [];
    } catch {
      presets = [];
    }
  };

  const persistPresets = (nextPresets) => {
    if (!storageKey || typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(storageKey, JSON.stringify(nextPresets));
    } catch {
      // ignore storage errors
    }
  };

  const updateSelection = () => {
    if (!textareaRef) return;
    selectionStart = textareaRef.selectionStart;
    selectionEnd = textareaRef.selectionEnd;
  };

  const handleAddPreset = () => {
    updateSelection();
    const text = (value ?? '').trim();
    if (!text) return;
    if (presets.includes(text)) return;
    presets = [text, ...presets];
    persistPresets(presets);
  };

  const handleTogglePresets = () => {
    updateSelection();
    showPresets = !showPresets;
  };

  const insertPreset = async (text) => {
    const current = value ?? '';
    const start = selectionStart ?? current.length;
    const end = selectionEnd ?? start;
    value = `${current.slice(0, start)}${text}${current.slice(end)}`;
    await tick();
    if (textareaRef) {
      const caretPos = start + text.length;
      textareaRef.focus();
      textareaRef.setSelectionRange(caretPos, caretPos);
      selectionStart = caretPos;
      selectionEnd = caretPos;
    }
    showPresets = false;
  };

  const closeModal = () => {
    showPresets = false;
  };

  $: canSave = (value ?? '').trim().length > 0;

  const handleKeydown = (event) => {
    if (event.key === 'Escape' && showPresets) {
      showPresets = false;
    }
  };

  const stopPropagation = (event) => {
    event.stopPropagation();
  };

  const preventMouseDown = (event) => {
    event.preventDefault();
  };

  onMount(() => {
    loadPresets();
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="space-y-2">
  <div class="flex items-center justify-between gap-2">
    <label class="block text-sm font-semibold text-textPrimary" for={textareaId}>{label}</label>
    <div class="flex items-center gap-2">
      <IconButton
        icon="add"
        label="Salva frase"
        disabled={disabled || !canSave}
        preventFocusLoss={true}
        on:click={handleAddPreset}
      />
      <IconButton
        icon="file"
        label="Mostra frasi"
        disabled={disabled}
        preventFocusLoss={true}
        on:click={handleTogglePresets}
      />
    </div>
  </div>

  {#if showPresets}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/30" on:click={closeModal}>
      <Card
        padding="sm"
        class="w-full max-w-md max-h-[70vh] overflow-hidden animate-slide-in"
        on:click={stopPropagation}
      >
        <div class="flex items-center justify-between gap-2 mb-2">
          <p class="text-sm font-semibold text-textPrimary">Frasi preimpostate</p>
          <CloseButton size="sm" ariaLabel="Chiudi" on:click={closeModal} />
        </div>

        <div class="space-y-1 max-h-[50vh] overflow-y-auto text-left">
          {#if presets.length === 0}
            <p class="text-sm text-textSecondary px-2 py-1">Nessuna frase salvata.</p>
          {:else}
            {#each presets as preset}
              <Button
                variant="text"
                size="sm"
                fullWidth
                className="justify-start text-left"
                on:click={() => insertPreset(preset)}
                on:mousedown={preventMouseDown}
              >
                <span class="block w-full text-left">{preset}</span>
              </Button>
            {/each}
          {/if}
        </div>
      </Card>
    </div>
  {/if}

  <textarea
    class="w-full min-h-[140px] px-3 py-2 border rounded-lg border-gray-200 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary text-textPrimary {disabled
      ? 'bg-surface-strong cursor-not-allowed text-textSecondary'
      : 'bg-surface'}"
    {placeholder}
    {disabled}
    id={textareaId}
    bind:this={textareaRef}
    bind:value
    on:input={updateSelection}
    on:click={updateSelection}
    on:keyup={updateSelection}
    on:select={updateSelection}
    on:focus={updateSelection}
  ></textarea>
</div>
