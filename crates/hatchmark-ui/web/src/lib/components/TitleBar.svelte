<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  const win = getCurrentWindow();
</script>

<header class="drag flex h-9 items-center justify-between px-4 text-xs text-neutral-500">
  <div class="select-none">Hatchmark</div>
  <div class="no-drag flex items-center gap-1">
    <button
      class="pressable flex h-6 w-6 items-center justify-center rounded-md hover:bg-black/5 dark:hover:bg-white/10"
      on:click={() => win.minimize()}
      aria-label="Minimize">
      &#8211;
    </button>
    <button
      class="pressable flex h-6 w-6 items-center justify-center rounded-md hover:bg-red-500/70 hover:text-white"
      on:click={() => win.close()}
      aria-label="Close">
      &#215;
    </button>
  </div>
</header>
