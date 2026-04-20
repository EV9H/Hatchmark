<script lang="ts">
  export let value: string = '';
  export let onchange: (v: string) => void;
  let listening = false;

  function startCapture() {
    listening = true;
    const handler = (ev: KeyboardEvent) => {
      ev.preventDefault();
      ev.stopPropagation();
      const key = ev.key;
      if (/^F(1[3-9]|2[0-4])$/.test(key)) {
        value = key;
        onchange(key);
        listening = false;
        window.removeEventListener('keydown', handler, true);
      } else if (key === 'Escape') {
        listening = false;
        window.removeEventListener('keydown', handler, true);
      }
    };
    window.addEventListener('keydown', handler, true);
  }
</script>

<button class="no-drag glass pressable rounded-xl px-3 py-1 text-sm" on:click={startCapture}>
  {listening ? 'Press F13-F24…' : value || 'Bind key'}
</button>
