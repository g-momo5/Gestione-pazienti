<script>
  import { createEventDispatcher, tick } from 'svelte';
  import Input from './Input.svelte';

  export let label = '';
  export let value = '';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let placeholder = 'hh:mm';
  export let invalid = false;
  export let inputMode = 'numeric';
  export let autoComplete = 'off';

  const dispatch = createEventDispatcher();

  let displayValue = '';
  let internalError = '';
  let internalUpdate = false;

  const digitsOnly = (raw) => String(raw || '').replace(/\D/g, '').slice(0, 4);

  const formatDisplay = (digits) => {
    if (digits.length <= 2) return digits;
    return `${digits.slice(0, 2)}:${digits.slice(2, 4)}`;
  };

  const formatTimeToDisplay = (time) => {
    if (!time) return '';
    const [hour, minute] = String(time).split(':');
    if (!hour || !minute) return '';
    return `${hour.padStart(2, '0')}:${minute.padStart(2, '0')}`;
  };

  const isValidTime = (time) => {
    if (!time || time.length !== 5) return false;
    const [hourStr, minuteStr] = time.split(':');
    const hour = Number(hourStr);
    const minute = Number(minuteStr);
    if (!Number.isFinite(hour) || !Number.isFinite(minute)) return false;
    if (hour < 0 || hour > 23) return false;
    if (minute < 0 || minute > 59) return false;
    return true;
  };

  $: if (internalUpdate) {
    internalUpdate = false;
  } else {
    displayValue = value ? formatTimeToDisplay(value) : '';
    internalError = '';
    invalid = false;
  }

  $: displayError = error || internalError;

  const clamp = (value, min, max) => Math.max(min, Math.min(max, value));

  const caretFromDigits = (digitsCount) => {
    let pos = digitsCount;
    if (digitsCount > 2) pos += 1;
    return pos;
  };

  async function handleInput(e) {
    const inputEl = e?.detail?.target;
    const raw = inputEl?.value ?? '';
    const caret = inputEl?.selectionStart ?? raw.length;
    const digitsBefore = clamp(raw.slice(0, caret).replace(/\D/g, '').length, 0, 4);
    const digits = digitsOnly(raw);
    const safeDigitsBefore = clamp(digitsBefore, 0, digits.length);
    displayValue = formatDisplay(digits);
    internalUpdate = true;

    if (digits.length === 0) {
      invalid = false;
      internalError = '';
      value = '';
      dispatch('input', { value, invalid });
      if (inputEl) {
        await tick();
        const nextPos = caretFromDigits(safeDigitsBefore);
        inputEl.setSelectionRange(nextPos, nextPos);
      }
      return;
    }

    if (digits.length < 4) {
      invalid = false;
      internalError = '';
      value = '';
      dispatch('input', { value, invalid });
      if (inputEl) {
        await tick();
        const nextPos = caretFromDigits(safeDigitsBefore);
        inputEl.setSelectionRange(nextPos, nextPos);
      }
      return;
    }

    const candidate = formatDisplay(digits);
    if (!isValidTime(candidate)) {
      invalid = true;
      internalError = 'Ora non valida';
      value = '';
      dispatch('input', { value, invalid });
      if (inputEl) {
        await tick();
        const nextPos = caretFromDigits(safeDigitsBefore);
        inputEl.setSelectionRange(nextPos, nextPos);
      }
      return;
    }

    invalid = false;
    internalError = '';
    value = candidate;
    dispatch('input', { value, invalid });
    if (inputEl) {
      await tick();
      const nextPos = caretFromDigits(safeDigitsBefore);
      inputEl.setSelectionRange(nextPos, nextPos);
    }
  }
</script>

<Input
  {label}
  {required}
  {disabled}
  placeholder={placeholder}
  value={displayValue}
  type="text"
  inputMode={inputMode}
  autoComplete={autoComplete}
  maxLength={5}
  error={displayError}
  on:input={handleInput}
/>
