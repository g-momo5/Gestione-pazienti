<script>
  import { onMount, afterUpdate } from 'svelte';

  export let label = '';
  export let value = '';
  export let placeholder = '';
  export let minHeight = '140px';

  let editorRef;
  let lastValue = '';

  const saveCursorPosition = () => {
    const selection = window.getSelection();
    if (!selection.rangeCount || !editorRef) return null;

    const range = selection.getRangeAt(0);
    const preCaretRange = range.cloneRange();
    preCaretRange.selectNodeContents(editorRef);
    preCaretRange.setEnd(range.endContainer, range.endOffset);

    return preCaretRange.toString().length;
  };

  const restoreCursorPosition = (position) => {
    if (!editorRef || position === null) return;

    const selection = window.getSelection();
    const range = document.createRange();

    let charCount = 0;
    let nodeStack = [editorRef];
    let node, foundNode, foundOffset;

    while (!foundNode && (node = nodeStack.pop())) {
      if (node.nodeType === Node.TEXT_NODE) {
        const nextCharCount = charCount + node.length;
        if (position <= nextCharCount) {
          foundNode = node;
          foundOffset = position - charCount;
        }
        charCount = nextCharCount;
      } else {
        for (let i = node.childNodes.length - 1; i >= 0; i--) {
          nodeStack.push(node.childNodes[i]);
        }
      }
    }

    if (foundNode) {
      range.setStart(foundNode, foundOffset);
      range.collapse(true);
      selection.removeAllRanges();
      selection.addRange(range);
    }
  };

  const execCommand = (command) => {
    const cursorPos = saveCursorPosition();
    document.execCommand(command, false, null);
    setTimeout(() => {
      updateValue();
      if (cursorPos !== null) {
        restoreCursorPosition(cursorPos);
      }
    }, 0);
  };

  const execBold = () => execCommand('bold');
  const execItalic = () => execCommand('italic');
  const execUnderline = () => execCommand('underline');

  const updateValue = () => {
    if (!editorRef) return;

    const newValue = editorRef.innerHTML
      .replace(/<br\s*\/?>/gi, '\n')
      .replace(/<\/div><div>/gi, '\n')
      .replace(/<div>/gi, '\n')
      .replace(/<\/div>/gi, '')
      .replace(/<strong>(.*?)<\/strong>/gi, '**$1**')
      .replace(/<b>(.*?)<\/b>/gi, '**$1**')
      .replace(/<em>(.*?)<\/em>/gi, '*$1*')
      .replace(/<i>(.*?)<\/i>/gi, '*$1*')
      .replace(/<u>(.*?)<\/u>/gi, '__$1__')
      .replace(/&nbsp;/g, ' ')
      .replace(/&amp;/g, '&')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'");

    if (newValue !== lastValue) {
      value = newValue;
      lastValue = newValue;
    }
  };

  const renderToEditor = (text) => {
    if (!text) return '';
    let html = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');

    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/__(.+?)__/g, '<u>$1</u>');
    html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');
    html = html.replace(/\n/g, '<br>');

    return html;
  };

  const handleInput = () => {
    updateValue();
  };

  onMount(() => {
    if (editorRef && value) {
      editorRef.innerHTML = renderToEditor(value);
      lastValue = value;
    }
  });

  // Solo quando il valore cambia da fuori (es. caricamento dati)
  $: if (editorRef && value !== lastValue && !editorRef.contains(document.activeElement)) {
    editorRef.innerHTML = renderToEditor(value);
    lastValue = value;
  }
</script>

<div class="space-y-2">
  {#if label}
    <div class="flex items-center justify-between">
      <p class="text-sm font-semibold text-textPrimary">{label}</p>
      <div class="flex gap-1">
        <button
          type="button"
          class="w-8 h-8 flex items-center justify-center text-sm border border-gray-200 rounded-md hover:bg-surface-stronger"
          on:click={execBold}
          aria-label="Grassetto"
        >
          <span class="font-bold">B</span>
        </button>
        <button
          type="button"
          class="w-8 h-8 flex items-center justify-center text-sm border border-gray-200 rounded-md hover:bg-surface-stronger italic"
          on:click={execItalic}
          aria-label="Corsivo"
        >
          I
        </button>
        <button
          type="button"
          class="w-8 h-8 flex items-center justify-center text-sm border border-gray-200 rounded-md hover:bg-surface-stronger underline"
          on:click={execUnderline}
          aria-label="Sottolineato"
        >
          U
        </button>
      </div>
    </div>
  {/if}

  <div
    class="w-full border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary bg-surface text-textPrimary text-left leading-relaxed rich-editor"
    bind:this={editorRef}
    contenteditable="true"
    on:input={handleInput}
    data-placeholder={placeholder}
    dir="ltr"
    style={`direction: ltr; text-align: left; min-height: ${minHeight};`}
    role="textbox"
    tabindex="0"
  ></div>
</div>

<style>
  .rich-editor:empty:before {
    content: attr(data-placeholder);
    color: #9ca3af;
  }

  .rich-editor:focus {
    outline: none;
  }
</style>
