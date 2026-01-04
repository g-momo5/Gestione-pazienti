<script>
  import Card from './ui/Card.svelte';
  import Button from './ui/Button.svelte';
  import IconBadge from './ui/IconBadge.svelte';
  import { formatDateIT, formatTime, calculateAge, calculateDurationMinutes, capitalizeWords } from '../utils/dateUtils.js';

  export let procedure;
  export let onEdit = null;
  export let onDelete = null;
  export let onClick = null;

  $: age = calculateAge(procedure.data_nascita);
  $: duration = calculateDurationMinutes(procedure.ora_inizio, procedure.ora_fine);
</script>

<Card padding="md" hover={!!onClick} clickable={!!onClick}>
  <div on:click={onClick} on:keydown={(e) => e.key === 'Enter' && onClick?.()} role={onClick ? 'button' : undefined} tabindex={onClick ? 0 : undefined}>
    <!-- Header: Nome e Età -->
    <div class="flex items-start justify-between mb-3">
      <div class="flex-1">
        <h4 class="text-lg font-semibold text-textPrimary">
          {capitalizeWords(procedure.nome)} {capitalizeWords(procedure.cognome)}
        </h4>
        <p class="text-sm text-textSecondary">
          {age !== null ? `${age} anni` : 'Età non disponibile'}
        </p>
      </div>

      <div class="flex gap-2">
        {#if onEdit}
          <Button variant="text" size="sm" on:click={(e) => { e.stopPropagation(); onEdit(procedure); }}>
            <IconBadge icon="edit" size="sm" tone="primary" />
          </Button>
        {/if}
        {#if onDelete}
          <Button variant="text" size="sm" on:click={(e) => { e.stopPropagation(); onDelete(procedure); }}>
            <IconBadge icon="delete" size="sm" tone="error" />
          </Button>
        {/if}
      </div>
    </div>

    <!-- Info Procedura -->
    <div class="grid grid-cols-2 gap-3 text-sm">
      <div class="flex items-center gap-2">
        <IconBadge icon="calendar" size="sm" tone="secondary" />
        <span class="text-textSecondary">{formatDateIT(procedure.data_procedura)}</span>
      </div>

      <div class="flex items-center gap-2">
        <IconBadge icon="timer" size="sm" tone="secondary" />
        <span class="text-textSecondary">
          {duration !== null ? `${duration} min` : '-'}
        </span>
      </div>

      <div class="flex items-center gap-2 col-span-2">
        <IconBadge icon="pill" size="sm" tone="secondary" />
        <span class="text-textSecondary font-medium">{procedure.modello_valvola}</span>
      </div>

      <div class="flex items-center gap-2 col-span-2">
        <IconBadge icon="tag" size="sm" tone="secondary" />
        <span class="text-textSecondary text-xs">{procedure.tipo_valvola}</span>
      </div>
    </div>
  </div>
</Card>
