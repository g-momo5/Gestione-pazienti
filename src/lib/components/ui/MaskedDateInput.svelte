<script>
  import { createEventDispatcher, tick } from 'svelte';
  import Input from './Input.svelte';

  export let label = '';
  export let value = '';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let placeholder = 'gg/mm/aaaa';
  export let invalid = false;
  export let minDate = '';
  export let maxDate = '';
  export let inputMode = 'numeric';
  export let autoComplete = 'off';

  const dispatch = createEventDispatcher();

  let displayValue = '';
  let internalError = '';
  let internalUpdate = false;

  const digitsOnly = (raw) => String(raw || '').replace(/\D/g, '').slice(0, 8);

  const formatDisplay = (digits) => {
    let out = '';
    for (let i = 0; i < digits.length; i += 1) {
      out += digits[i];
      if (i === 1 || i === 3) out += '/';
    }
    return out;
  };

  const isoFromDigits = (digits) => {
    const day = digits.slice(0, 2);
    const month = digits.slice(2, 4);
    const year = digits.slice(4, 8);
    return `${year}-${month}-${day}`;
  };

  const isValidIsoDate = (iso) => {
    if (!iso || iso.length !== 10) return false;
    const [yearStr, monthStr, dayStr] = iso.split('-');
    const year = Number(yearStr);
    const month = Number(monthStr);
    const day = Number(dayStr);
    if (!year || !month || !day) return false;
    if (year < 1900 || year > 2100) return false;
    if (month < 1 || month > 12) return false;
    if (day < 1 || day > 31) return false;
    const candidate = new Date(`${iso}T00:00:00`);
    return (
      candidate.getFullYear() === year &&
      candidate.getMonth() + 1 === month &&
      candidate.getDate() === day
    );
  };

  const inRange = (iso) => {
    if (minDate && iso < minDate) return false;
    if (maxDate && iso > maxDate) return false;
    return true;
  };

  const formatIsoToDisplay = (iso) => {
    if (!iso) return '';
    const [year, month, day] = iso.split('-');
    if (!year || !month || !day) return '';
    return `${day.padStart(2, '0')}/${month.padStart(2, '0')}/${year}`;
  };

  $: if (internalUpdate) {
    internalUpdate = false;
  } else {
    displayValue = value ? formatIsoToDisplay(value) : '';
    internalError = '';
    invalid = false;
  }

  $: displayError = error || internalError;

  const clamp = (value, min, max) => Math.max(min, Math.min(max, value));

  const caretFromDigits = (digitsCount) => {
    let pos = digitsCount;
    if (digitsCount > 2) pos += 1;
    if (digitsCount > 4) pos += 1;
    return pos;
  };

  async function handleInput(e) {
    const inputEl = e?.detail?.target;
    const raw = inputEl?.value ?? '';
    const caret = inputEl?.selectionStart ?? raw.length;
    const digitsBefore = clamp(raw.slice(0, caret).replace(/\D/g, '').length, 0, 8);
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

    if (digits.length < 8) {
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

    const iso = isoFromDigits(digits);
    if (!isValidIsoDate(iso)) {
      invalid = true;
      internalError = 'Data non valida';
      value = '';
      dispatch('input', { value, invalid });
      if (inputEl) {
        await tick();
        const nextPos = caretFromDigits(safeDigitsBefore);
        inputEl.setSelectionRange(nextPos, nextPos);
      }
      return;
    }

    if (!inRange(iso)) {
      invalid = true;
      internalError = 'Data fuori intervallo';
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
    value = iso;
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
  maxLength={10}
  className="date-field"
  error={displayError}
  on:input={handleInput}
/>
