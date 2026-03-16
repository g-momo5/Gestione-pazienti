<script>
  import Card from '../components/ui/Card.svelte';
  import Button from '../components/ui/Button.svelte';
  import Input from '../components/ui/Input.svelte';
  import IconBadge from '../components/ui/IconBadge.svelte';
  import BackCircleButton from '../components/ui/BackCircleButton.svelte';
  import Select from '../components/ui/Select.svelte';
  import { formatDateIT, calculateAge, getTodayISO } from '../utils/dateUtils.js';
  import { notifyError, notifySuccess } from '../utils/notify.js';
  import { updatePatient } from '../stores/patientStore.js';

  const AMBULATORIO_STANDARD_SLOTS = ['08:30', '09:00', '09:45', '10:30', '11:15'];
  const AMBULATORIO_TC_TIME = '11:45';
  const CUSTOM_TIME_OPTION = '__custom_time__';
  const SLOT_SORT_ORDER = { standard: 0, tc: 1, custom: 2 };

  export let patients = [];
  export let openDates = [];
  export let onBack = () => {};
  export let onOpenPatient = () => {};
  export let onOpenNote = () => {};
  export let onPersistOpenDates = async () => false;

  let listDate = getTodayISO();
  let showAvailableDatesModal = false;
  let showAssignModal = false;
  let showMoveModal = false;

  let assignPatientId = '';
  let assignSlot = '';
  let assignDate = '';
  let assignSearch = '';
  let assignDateInvalid = false;
  let savingAssign = false;

  let movePatientEntry = null;
  let moveDate = '';
  let moveDateInvalid = false;
  let moveTimeSelection = '';
  let moveCustomTime = '';
  let savingMove = false;

  const normalizeSearchValue = (value) =>
    String(value || '')
      .toLowerCase()
      .replace(/\s+/g, ' ')
      .trim();

  const normalizeSearchCompact = (value) =>
    String(value || '')
      .toLowerCase()
      .replace(/\s+/g, '');

  const matchesPatientQuery = (entry, normalized, compact) => {
    if (!normalized && !compact) return true;
    const data = entry?.patient ?? entry;
    const nome = normalizeSearchValue(data?.nome);
    const cognome = normalizeSearchValue(data?.cognome);
    const cf = normalizeSearchValue(data?.codice_fiscale);
    const full = `${nome} ${cognome}`.trim();
    const fullReverse = `${cognome} ${nome}`.trim();

    return (
      nome.includes(normalized) ||
      cognome.includes(normalized) ||
      cf.includes(normalized) ||
      full.includes(normalized) ||
      fullReverse.includes(normalized) ||
      normalizeSearchCompact(full).includes(compact)
    );
  };

  const isIsoDate = (value) => /^\d{4}-\d{2}-\d{2}$/.test(value || '');
  const normalizeIsoDate = (value) => String(value || '').split('T')[0];
  const toIsoDate = (date) => {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  };
  const addDaysToIso = (iso, delta) => {
    const base = isIsoDate(iso) ? new Date(`${iso}T00:00:00`) : new Date();
    base.setDate(base.getDate() + delta);
    return toIsoDate(base);
  };
  const sanitizeOpenDates = (dates) => {
    if (!Array.isArray(dates)) return [];
    return [...new Set(dates.map((value) => normalizeIsoDate(value)).filter((value) => isIsoDate(value)))].sort();
  };
  const normalizeSlotTime = (value) => {
    const raw = String(value || '').trim();
    if (!raw) return '';
    const compact = raw.replace('.', ':');
    if (!/^\d{1,2}:\d{1,2}$/.test(compact)) return raw;
    const [hoursRaw, minutesRaw] = compact.split(':');
    const hours = Number(hoursRaw);
    const minutes = Number(minutesRaw);
    if (!Number.isInteger(hours) || !Number.isInteger(minutes)) return raw;
    if (hours < 0 || hours > 23 || minutes < 0 || minutes > 59) return raw;
    return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
  };
  const isValidSlotTime = (value) => /^\d{2}:\d{2}$/.test(value || '');
  const timeToMinutes = (value) => {
    if (!isValidSlotTime(value)) return Number.POSITIVE_INFINITY;
    const [hour, minute] = value.split(':').map(Number);
    return hour * 60 + minute;
  };
  const formatSlotLabel = (value) => {
    const normalized = normalizeSlotTime(value);
    if (!isValidSlotTime(normalized)) return value || '-';
    const [hour, minute] = normalized.split(':');
    return `${Number(hour)}.${minute}`;
  };
  const isStandardSlot = (value) => AMBULATORIO_STANDARD_SLOTS.includes(normalizeSlotTime(value));
  const sortPatientByName = (a, b) => {
    const aLast = (a?.patient?.cognome || '').toLowerCase();
    const bLast = (b?.patient?.cognome || '').toLowerCase();
    if (aLast !== bLast) return aLast.localeCompare(bLast);
    const aFirst = (a?.patient?.nome || '').toLowerCase();
    const bFirst = (b?.patient?.nome || '').toLowerCase();
    return aFirst.localeCompare(bFirst);
  };
  const getEntryId = (entry) => String(entry?.patient?.id ?? entry?.id ?? '');
  const getPatientData = (entry) => entry?.patient ?? entry ?? null;
  const hasPatientNote = (entry) => Boolean(String(getPatientData(entry)?.note || '').trim());
  const formatAppointmentSummary = (entry) => {
    const date = normalizeIsoDate(entry?.patient?.ambulatorio_data_visita);
    const time = normalizeSlotTime(entry?.patient?.ambulatorio_orario_visita);
    if (!isIsoDate(date) || !isValidSlotTime(time)) return null;
    return `${formatDateIT(date)} alle ${formatSlotLabel(time)}`;
  };

  const buildRows = (entries = []) => {
    const bookedByTime = new Map();
    const fallbackCustomRows = [];

    for (const entry of entries) {
      const rawTime = String(entry?.patient?.ambulatorio_orario_visita || '').trim();
      const normalizedTime = normalizeSlotTime(rawTime);
      if (isValidSlotTime(normalizedTime)) {
        bookedByTime.set(normalizedTime, entry);
      } else if (rawTime) {
        fallbackCustomRows.push({
          rowType: 'custom',
          time: rawTime,
          timeLabel: rawTime,
          patient: entry,
        });
      }
    }

    const rows = [];
    for (const slot of AMBULATORIO_STANDARD_SLOTS) {
      rows.push({
        rowType: 'standard',
        time: slot,
        timeLabel: formatSlotLabel(slot),
        patient: bookedByTime.get(slot) || null,
      });
    }

    rows.push({
      rowType: 'tc',
      time: AMBULATORIO_TC_TIME,
      timeLabel: formatSlotLabel(AMBULATORIO_TC_TIME),
      label: 'Valutazione TC',
      patient: null,
    });

    const customTimes = [...bookedByTime.keys()]
      .filter((time) => !AMBULATORIO_STANDARD_SLOTS.includes(time))
      .sort((a, b) => timeToMinutes(a) - timeToMinutes(b));

    for (const customTime of customTimes) {
      rows.push({
        rowType: 'custom',
        time: customTime,
        timeLabel: formatSlotLabel(customTime),
        patient: bookedByTime.get(customTime) || null,
      });
    }

    rows.push(...fallbackCustomRows);

    return rows.sort((a, b) => {
      const minuteDiff = timeToMinutes(a.time) - timeToMinutes(b.time);
      if (minuteDiff !== 0) return minuteDiff;
      return (SLOT_SORT_ORDER[a.rowType] ?? 99) - (SLOT_SORT_ORDER[b.rowType] ?? 99);
    });
  };

  const getBookedStandardSlots = (dateIso, excludeId = null) => {
    if (!isIsoDate(dateIso)) return new Set();
    const booked = new Set();

    for (const entry of patients || []) {
      const id = getEntryId(entry);
      if (excludeId !== null && id === String(excludeId)) continue;
      if (normalizeIsoDate(entry?.patient?.ambulatorio_data_visita) !== dateIso) continue;
      const time = normalizeSlotTime(entry?.patient?.ambulatorio_orario_visita);
      if (isStandardSlot(time)) {
        booked.add(time);
      }
    }

    return booked;
  };

  const findAppointmentConflict = (dateIso, timeIso, excludeId = null) => {
    return (patients || []).find((entry) => {
      const id = getEntryId(entry);
      if (excludeId !== null && id === String(excludeId)) return false;
      const date = normalizeIsoDate(entry?.patient?.ambulatorio_data_visita);
      const time = normalizeSlotTime(entry?.patient?.ambulatorio_orario_visita);
      return date === dateIso && time === timeIso;
    });
  };

  $: availableDates = sanitizeOpenDates(openDates);
  $: dateInvalid = Boolean(listDate) && !isIsoDate(listDate);
  $: isCurrentDateAvailable = availableDates.includes(listDate);
  $: previousAvailableDate = availableDates.filter((date) => date < listDate).at(-1) || '';
  $: nextAvailableDate = availableDates.find((date) => date > listDate) || '';
  $: patientsForDate = (patients || [])
    .filter((entry) => normalizeIsoDate(entry?.patient?.ambulatorio_data_visita) === listDate)
    .sort((a, b) => {
      const aTime = normalizeSlotTime(a?.patient?.ambulatorio_orario_visita);
      const bTime = normalizeSlotTime(b?.patient?.ambulatorio_orario_visita);
      const timeDiff = timeToMinutes(aTime) - timeToMinutes(bTime);
      if (timeDiff !== 0) return timeDiff;
      return sortPatientByName(a, b);
    });
  $: rows = buildRows(patientsForDate);

  $: daValutarePatients = ((patients || []).filter((entry) => entry?.status === 'Da valutare') ?? [])
    .slice()
    .sort(sortPatientByName);
  $: assignQueryNormalized = normalizeSearchValue(assignSearch);
  $: assignQueryCompact = normalizeSearchCompact(assignSearch);
  $: filteredAssignablePatients = daValutarePatients.filter((entry) =>
    matchesPatientQuery(entry, assignQueryNormalized, assignQueryCompact)
  );

  $: movePatientId = getEntryId(movePatientEntry);
  $: moveBookedStandardSlots = getBookedStandardSlots(moveDate, movePatientId || null);
  $: moveAvailableStandardSlots = AMBULATORIO_STANDARD_SLOTS.filter((slot) => !moveBookedStandardSlots.has(slot));
  $: moveTimeOptions = [
    ...moveAvailableStandardSlots.map((slot) => ({
      value: slot,
      label: formatSlotLabel(slot),
    })),
    { value: CUSTOM_TIME_OPTION, label: 'Orario personalizzato' },
  ];

  function shiftDate(delta) {
    listDate = addDaysToIso(listDate, delta);
  }

  function shiftToAvailableDate(direction) {
    if (!isIsoDate(listDate)) return;
    if (direction < 0 && previousAvailableDate) {
      listDate = previousAvailableDate;
      return;
    }
    if (direction > 0 && nextAvailableDate) {
      listDate = nextAvailableDate;
    }
  }

  function setToday() {
    listDate = getTodayISO();
  }

  async function addAvailableDate(date, opts = {}) {
    if (!isIsoDate(date)) {
      notifyError('Seleziona una data valida');
      return false;
    }
    if (availableDates.includes(date)) return true;
    return Boolean(
      await onPersistOpenDates([...availableDates, date], {
        successMessage: opts.successMessage || `Data ${formatDateIT(date)} aggiunta tra le date disponibili`,
      })
    );
  }

  async function removeAvailableDate(date) {
    if (!isIsoDate(date)) {
      notifyError('Seleziona una data valida');
      return false;
    }
    if (!availableDates.includes(date)) return true;
    return Boolean(
      await onPersistOpenDates(
        availableDates.filter((entry) => entry !== date),
        { successMessage: `Data ${formatDateIT(date)} rimossa dalle date disponibili` }
      )
    );
  }

  function openAssignModal(slot) {
    assignSlot = slot;
    assignDate = listDate;
    assignDateInvalid = false;
    assignPatientId = '';
    assignSearch = '';
    showAssignModal = true;
  }

  function closeAssignModal() {
    showAssignModal = false;
    assignSlot = '';
    assignDate = '';
    assignDateInvalid = false;
    assignPatientId = '';
    assignSearch = '';
    savingAssign = false;
  }

  async function saveAssignedAppointment() {
    if (savingAssign) return;

    const selectedDate = normalizeIsoDate(assignDate);
    const selectedTime = normalizeSlotTime(assignSlot);

    if (!isIsoDate(selectedDate) || assignDateInvalid) {
      notifyError("Seleziona una data appuntamento valida");
      return;
    }
    if (!isValidSlotTime(selectedTime)) {
      notifyError("Seleziona un orario appuntamento valido");
      return;
    }
    if (!assignPatientId) {
      notifyError('Seleziona un paziente da assegnare');
      return;
    }

    const selectedPatient = daValutarePatients.find((entry) => getEntryId(entry) === String(assignPatientId));
    if (!selectedPatient?.patient) {
      notifyError('Paziente non trovato');
      return;
    }

    const existingDate = normalizeIsoDate(selectedPatient.patient.ambulatorio_data_visita);
    const existingTime = normalizeSlotTime(selectedPatient.patient.ambulatorio_orario_visita);
    const hasExistingAppointment = isIsoDate(existingDate) && isValidSlotTime(existingTime);
    const isChangingAppointment =
      hasExistingAppointment && (existingDate !== selectedDate || existingTime !== selectedTime);

    if (isChangingAppointment) {
      const confirmed = window.confirm(
        `${selectedPatient.patient.cognome} ${selectedPatient.patient.nome} ha già un appuntamento il ${formatDateIT(existingDate)} alle ${formatSlotLabel(existingTime)}. Vuoi riassegnarlo a ${formatDateIT(selectedDate)} alle ${formatSlotLabel(selectedTime)}?`
      );
      if (!confirmed) return;
    }

    if (!availableDates.includes(selectedDate)) {
      const confirmAddDate = window.confirm(
        `La data ${formatDateIT(selectedDate)} non è tra le date disponibili. Vuoi aggiungerla adesso?`
      );
      if (!confirmAddDate) return;
      const added = await addAvailableDate(selectedDate, {
        successMessage: `Data ${formatDateIT(selectedDate)} aggiunta tra le date disponibili`,
      });
      if (!added) return;
    }

    const conflict = findAppointmentConflict(selectedDate, selectedTime, getEntryId(selectedPatient));
    if (conflict) {
      notifyError(`Slot occupato: ${formatDateIT(selectedDate)} alle ${formatSlotLabel(selectedTime)}`);
      return;
    }

    savingAssign = true;
    try {
      await updatePatient({
        ...selectedPatient.patient,
        ambulatorio_data_visita: selectedDate,
        ambulatorio_orario_visita: selectedTime,
      });
      notifySuccess(
        `Appuntamento assegnato a ${selectedPatient.patient.cognome} ${selectedPatient.patient.nome}`
      );
      closeAssignModal();
    } catch (e) {
      console.error(e);
      notifyError(e, "Errore durante l'assegnazione dell'appuntamento");
    } finally {
      savingAssign = false;
    }
  }

  async function removeAppointment(entry) {
    const patientData = entry?.patient;
    if (!patientData?.id) return;

    const confirmed = window.confirm(
      `Vuoi eliminare l'appuntamento di ${patientData.cognome || ''} ${patientData.nome || ''}?`
    );
    if (!confirmed) return;

    try {
      await updatePatient({
        ...patientData,
        ambulatorio_data_visita: null,
        ambulatorio_orario_visita: null,
      });
      notifySuccess('Appuntamento eliminato');
    } catch (e) {
      console.error(e);
      notifyError(e, "Errore durante l'eliminazione dell'appuntamento");
    }
  }

  function openMoveModal(entry) {
    const patientData = entry?.patient;
    if (!patientData?.id) return;

    const currentDate = normalizeIsoDate(patientData.ambulatorio_data_visita);
    const currentTime = normalizeSlotTime(patientData.ambulatorio_orario_visita);

    movePatientEntry = entry;
    moveDate = isIsoDate(currentDate) ? currentDate : listDate;
    moveDateInvalid = false;

    if (isStandardSlot(currentTime)) {
      moveTimeSelection = currentTime;
      moveCustomTime = '';
    } else if (isValidSlotTime(currentTime)) {
      moveTimeSelection = CUSTOM_TIME_OPTION;
      moveCustomTime = currentTime;
    } else {
      moveTimeSelection = '';
      moveCustomTime = '';
    }

    showMoveModal = true;
  }

  function closeMoveModal() {
    showMoveModal = false;
    movePatientEntry = null;
    moveDate = '';
    moveDateInvalid = false;
    moveTimeSelection = '';
    moveCustomTime = '';
    savingMove = false;
  }

  async function saveMovedAppointment() {
    if (savingMove) return;
    const patientData = movePatientEntry?.patient;
    if (!patientData?.id) {
      notifyError('Paziente non valido');
      return;
    }

    const selectedDate = normalizeIsoDate(moveDate);
    if (!isIsoDate(selectedDate) || moveDateInvalid) {
      notifyError("Seleziona una data valida");
      return;
    }

    let selectedTime = '';
    if (moveTimeSelection === CUSTOM_TIME_OPTION) {
      selectedTime = normalizeSlotTime(moveCustomTime);
      if (!isValidSlotTime(selectedTime)) {
        notifyError("Inserisci un orario personalizzato valido");
        return;
      }
    } else {
      selectedTime = normalizeSlotTime(moveTimeSelection);
      if (!isValidSlotTime(selectedTime)) {
        notifyError("Seleziona un orario valido");
        return;
      }
    }

    if (!availableDates.includes(selectedDate)) {
      const confirmAddDate = window.confirm(
        `La data ${formatDateIT(selectedDate)} non è tra le date disponibili. Vuoi aggiungerla adesso?`
      );
      if (!confirmAddDate) return;
      const added = await addAvailableDate(selectedDate, {
        successMessage: `Data ${formatDateIT(selectedDate)} aggiunta tra le date disponibili`,
      });
      if (!added) return;
    }

    const conflict = findAppointmentConflict(selectedDate, selectedTime, patientData.id);
    if (conflict) {
      notifyError(`Slot occupato: ${formatDateIT(selectedDate)} alle ${formatSlotLabel(selectedTime)}`);
      return;
    }

    savingMove = true;
    try {
      await updatePatient({
        ...patientData,
        ambulatorio_data_visita: selectedDate,
        ambulatorio_orario_visita: selectedTime,
      });
      notifySuccess('Appuntamento spostato');
      closeMoveModal();
    } catch (e) {
      console.error(e);
      notifyError(e, "Errore durante lo spostamento dell'appuntamento");
    } finally {
      savingMove = false;
    }
  }
</script>

<div class="max-w-7xl mx-auto space-y-6">
  <div class="space-y-2">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="flex items-start gap-3">
        <BackCircleButton onClick={onBack} />
        <div>
          <h2 class="text-2xl font-bold text-textPrimary">Visite ambulatorio</h2>
          <p class="text-textSecondary">Programmazione a slot dell'ambulatorio strutturale.</p>
        </div>
      </div>
      <Button
        variant="secondary"
        class="flex flex-col items-center gap-1 min-w-[120px]"
        on:click={() => (showAvailableDatesModal = true)}
      >
        <IconBadge icon="calendar" size="lg" tone="neutral" class="mb-1" />
        <span class="text-xs font-semibold">Date disponibili</span>
      </Button>
    </div>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-[320px_minmax(0,1fr)] gap-6">
    <Card padding="lg" class="border border-gray-200 space-y-4">
      <div>
        <h3 class="text-base font-semibold text-textPrimary">Navigazione date</h3>
        <p class="text-xs text-textSecondary mt-1">Scegli la data da programmare o salta tra date disponibili.</p>
      </div>

      <div class="w-full">
        <Input label="Data" type="date" bind:value={listDate} />
      </div>

      <div class="grid grid-cols-[1fr_1fr_1.4fr_1fr_1fr] gap-2 w-full">
        <Button
          variant="secondary"
          size="sm"
          class="w-full"
          on:click={() => shiftToAvailableDate(-1)}
          disabled={!previousAvailableDate}
        >
          &lt;&lt;
        </Button>
        <Button variant="secondary" size="sm" class="w-full" on:click={() => shiftDate(-1)}>
          &lt;
        </Button>
        <Button variant="secondary" size="sm" class="w-full text-center" on:click={setToday}>
          Oggi
        </Button>
        <Button variant="secondary" size="sm" class="w-full" on:click={() => shiftDate(1)}>
          &gt;
        </Button>
        <Button
          variant="secondary"
          size="sm"
          class="w-full"
          on:click={() => shiftToAvailableDate(1)}
          disabled={!nextAvailableDate}
        >
          &gt;&gt;
        </Button>
      </div>

      <div class="space-y-2">
        <h4 class="text-sm font-semibold text-textPrimary">Date disponibili</h4>
        {#if availableDates.length === 0}
          <p class="text-sm text-textSecondary">Nessuna data disponibile configurata.</p>
        {:else}
          <div class="max-h-72 overflow-y-auto space-y-1 pr-1">
            {#each availableDates as openDate}
              <button
                type="button"
                class={`w-full text-left px-3 py-2 rounded-lg border text-sm transition-colors ${openDate === listDate ? 'border-primary bg-primary/10 text-primary font-medium' : 'border-gray-200 bg-surface-strong text-textPrimary hover:border-primary/40'}`}
                on:click={() => (listDate = openDate)}
              >
                {formatDateIT(openDate)}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </Card>

    <div class="bg-surface rounded-xl shadow-sm border border-gray-200 overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
        <h3 class="text-lg font-semibold text-textPrimary">
          {formatDateIT(listDate) ? `Programmazione del ${formatDateIT(listDate)}` : 'Programmazione visite'}
        </h3>
        <span class={`text-xs px-2.5 py-1 rounded-full font-medium ${isCurrentDateAvailable ? 'bg-green-100 text-green-800' : 'bg-amber-100 text-amber-800'}`}>
          {isCurrentDateAvailable ? 'Data disponibile' : 'Data non disponibile'}
        </span>
      </div>

      {#if !isIsoDate(listDate) || dateInvalid}
        <div class="px-6 py-10 text-center text-textSecondary">
          Inserisci una data valida per vedere la programmazione.
        </div>
      {:else if !isCurrentDateAvailable}
        <div class="px-6 py-10 text-center text-textSecondary">
          Gli slot sono visibili solo per le date disponibili.
        </div>
      {:else}
        <div class="divide-y divide-gray-200">
          {#each rows as row}
            <div class={`px-6 py-3 grid grid-cols-1 md:grid-cols-[max-content_30px_minmax(0,1fr)_auto] gap-y-3 md:gap-y-2 gap-x-0.5 items-center ${row.rowType === 'tc' ? 'bg-surface-strong' : ''}`}>
              <div class="text-sm font-semibold text-textPrimary pr-0.5">{row.timeLabel}</div>

              {#if row.rowType === 'tc'}
                <div class="hidden md:block"></div>
                <div class="text-sm text-textSecondary">{row.label}</div>
                <div class="text-xs text-textSecondary">Informativo</div>
              {:else if row.patient}
                {@const slotPatient = getPatientData(row.patient)}
                {@const slotHasNote = hasPatientNote(row.patient)}
                <button
                  type="button"
                  class="p-1.5 rounded-full hover:bg-primary/10 transition focus:outline-none focus:ring-2 focus:ring-primary/30 justify-self-center"
                  on:click|stopPropagation={() => onOpenNote(row.patient)}
                  aria-label="Note paziente"
                  title={slotHasNote ? 'Modifica nota paziente' : 'Aggiungi nota paziente'}
                >
                  <IconBadge icon="note" tone={slotHasNote ? 'primary' : 'neutral'} />
                </button>
                <button
                  type="button"
                  class="w-full text-left rounded-lg border border-gray-200 bg-surface-strong px-3 py-2 hover:border-primary/40 transition-colors"
                  on:click={() => onOpenPatient(row.patient)}
                >
                  <p class="font-medium text-textPrimary">
                    {slotPatient?.cognome} {slotPatient?.nome}
                  </p>
                  <p class="text-xs text-textSecondary">
                    {slotPatient?.data_nascita ? `${formatDateIT(slotPatient?.data_nascita)} (${calculateAge(slotPatient?.data_nascita)} anni)` : 'Data nascita non disponibile'}
                  </p>
                </button>
                <div class="flex flex-wrap items-center gap-2 justify-start md:justify-end">
                  <Button variant="secondary" size="sm" class="min-w-[84px]" on:click={() => onOpenPatient(row.patient)}>
                    Apri
                  </Button>
                  <Button variant="secondary" size="sm" class="min-w-[84px]" on:click={() => openMoveModal(row.patient)}>
                    Sposta
                  </Button>
                  <Button variant="secondary" size="sm" class="min-w-[84px]" on:click={() => removeAppointment(row.patient)}>
                    Elimina
                  </Button>
                </div>
              {:else if row.rowType === 'standard'}
                <button
                  type="button"
                  class="md:col-start-3 md:col-span-2 text-left rounded-lg border border-dashed border-primary/40 bg-primary/5 px-3 py-2 hover:bg-primary/10 transition-colors"
                  on:click={() => openAssignModal(row.time)}
                >
                  <p class="text-sm font-semibold text-primary">Slot libero</p>
                  <p class="text-xs text-textSecondary">Clicca sullo slot per assegnare un appuntamento</p>
                </button>
              {:else}
                <div class="hidden md:block"></div>
                <div class="text-sm text-textSecondary">Orario personalizzato libero</div>
                <div class="text-xs text-textSecondary">Non standard</div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

{#if showAvailableDatesModal}
  <div class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4">
    <div class="max-w-xl w-full" role="dialog" aria-modal="true" tabindex="-1">
      <Card padding="lg" class="bg-surface space-y-4">
        <div class="flex items-start justify-between gap-3">
          <div>
            <h3 class="text-lg font-semibold text-textPrimary">Date disponibili</h3>
            <p class="text-sm text-textSecondary">Gestione completa delle date disponibili dell'ambulatorio.</p>
          </div>
          <Button variant="text" size="sm" on:click={() => (showAvailableDatesModal = false)}>Chiudi</Button>
        </div>

        <div class="flex flex-wrap gap-2">
          <Button
            variant="primary"
            size="sm"
            on:click={() => addAvailableDate(listDate)}
            disabled={!isIsoDate(listDate) || dateInvalid}
          >
            Aggiungi data selezionata
          </Button>
        </div>

        {#if availableDates.length === 0}
          <div class="rounded-lg border border-gray-200 bg-surface-strong px-4 py-6 text-sm text-textSecondary text-center">
            Nessuna data disponibile configurata.
          </div>
        {:else}
          <div class="max-h-80 overflow-y-auto space-y-2 pr-1">
            {#each availableDates as openDate}
              <div class="rounded-lg border border-gray-200 bg-surface-strong px-3 py-2 flex items-center justify-between gap-2">
                <button
                  type="button"
                  class="text-sm text-left font-medium text-textPrimary hover:text-primary transition-colors"
                  on:click={() => {
                    listDate = openDate;
                    showAvailableDatesModal = false;
                  }}
                >
                  {formatDateIT(openDate)}
                </button>
                <Button variant="text" size="sm" on:click={() => removeAvailableDate(openDate)}>
                  Rimuovi
                </Button>
              </div>
            {/each}
          </div>
        {/if}
      </Card>
    </div>
  </div>
{/if}

{#if showAssignModal}
  <div class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4">
    <div class="max-w-2xl w-full" role="dialog" aria-modal="true" tabindex="-1">
      <Card padding="lg" class="bg-surface space-y-4">
        <div class="flex items-start justify-between gap-3">
          <div>
            <h3 class="text-lg font-semibold text-textPrimary">Assegna appuntamento</h3>
            <p class="text-sm text-textSecondary">
              Slot selezionato: <span class="font-semibold text-textPrimary">{formatDateIT(assignDate)} alle {formatSlotLabel(assignSlot)}</span>
            </p>
          </div>
          <Button variant="text" size="sm" on:click={closeAssignModal}>Chiudi</Button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <Input label="Data appuntamento" type="date" bind:value={assignDate} />
          <Input label="Ricerca paziente" placeholder="Cognome, nome o codice fiscale" bind:value={assignSearch} />
        </div>

        {#if filteredAssignablePatients.length === 0}
          <div class="rounded-lg border border-gray-200 bg-surface-strong px-4 py-6 text-sm text-textSecondary text-center">
            Nessun paziente "Da valutare" trovato con i filtri correnti.
          </div>
        {:else}
          <div class="max-h-80 overflow-y-auto space-y-2 pr-1">
            {#each filteredAssignablePatients as patientEntry}
              {@const appointmentSummary = formatAppointmentSummary(patientEntry)}
              {@const assignPatient = getPatientData(patientEntry)}
              {@const assignHasNote = hasPatientNote(patientEntry)}
              <button
                type="button"
                class={`w-full text-left rounded-lg border px-3 py-2 transition-colors ${assignPatientId === getEntryId(patientEntry) ? 'border-primary bg-primary/5' : 'border-gray-200 bg-surface hover:border-primary/40'}`}
                on:click={() => (assignPatientId = getEntryId(patientEntry))}
              >
                <div class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-2 gap-y-1">
                  <span
                    class={`inline-flex row-span-2 self-center shrink-0 items-center justify-center rounded-full border p-0.5 ${assignHasNote ? 'border-amber-300 bg-amber-100' : 'border-gray-300 bg-surface'}`}
                    title={assignHasNote ? 'Nota presente' : 'Nessuna nota'}
                  >
                    <IconBadge icon="note" size="md" tone={assignHasNote ? 'warning' : 'neutral'} />
                  </span>
                  <p class="text-sm font-semibold text-textPrimary">
                    {assignPatient?.cognome} {assignPatient?.nome}
                  </p>
                  <p class="text-xs text-textSecondary">
                    {assignPatient?.data_nascita ? formatDateIT(assignPatient?.data_nascita) : 'Data nascita non disponibile'}
                  </p>
                  {#if appointmentSummary}
                    <p class="col-start-2 text-xs text-textSecondary">
                      Appuntamento attuale: {appointmentSummary}
                    </p>
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/if}

        <div class="flex justify-end gap-2">
          <Button variant="text" size="sm" on:click={closeAssignModal}>Annulla</Button>
          <Button variant="primary" size="sm" on:click={saveAssignedAppointment} disabled={savingAssign}>
            {savingAssign ? 'Salvataggio...' : 'Assegna appuntamento'}
          </Button>
        </div>
      </Card>
    </div>
  </div>
{/if}

{#if showMoveModal}
  <div class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4">
    <div class="max-w-lg w-full" role="dialog" aria-modal="true" tabindex="-1">
      <Card padding="lg" class="bg-surface space-y-4">
        <div class="flex items-start justify-between gap-3">
          <div>
            <h3 class="text-lg font-semibold text-textPrimary">Sposta appuntamento</h3>
            <p class="text-sm text-textSecondary">
              {movePatientEntry?.patient?.cognome} {movePatientEntry?.patient?.nome}
            </p>
          </div>
          <Button variant="text" size="sm" on:click={closeMoveModal}>Chiudi</Button>
        </div>

        <Input label="Data appuntamento" type="date" bind:value={moveDate} />

        <Select
          label="Orario"
          options={moveTimeOptions}
          bind:value={moveTimeSelection}
          placeholder="Seleziona orario"
        />

        {#if moveTimeSelection === CUSTOM_TIME_OPTION}
          <Input label="Orario personalizzato (HH:MM)" placeholder="Es. 12:20" bind:value={moveCustomTime} />
        {/if}

        <div class="flex justify-end gap-2">
          <Button variant="text" size="sm" on:click={closeMoveModal}>Annulla</Button>
          <Button variant="primary" size="sm" on:click={saveMovedAppointment} disabled={savingMove}>
            {savingMove ? 'Salvataggio...' : 'Salva spostamento'}
          </Button>
        </div>
      </Card>
    </div>
  </div>
{/if}
