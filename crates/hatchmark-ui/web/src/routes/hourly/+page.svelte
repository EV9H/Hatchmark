<script lang="ts">
  import { onMount } from 'svelte';
  import BarPlot from '$lib/components/BarPlot.svelte';
  import { listChannels, hourly } from '$lib/tauri';
  import type { Channel } from '$lib/types';

  let channels: Channel[] = [];
  let selected: number | null = null;
  let fromDate = new Date(Date.now() - 29 * 864e5).toISOString().slice(0, 10);
  let toDate = new Date().toISOString().slice(0, 10);
  const labels = Array.from({ length: 24 }, (_, h) => h.toString().padStart(2, '0'));
  let values: number[] = Array(24).fill(0);

  async function reload() {
    if (selected == null) return;
    const rows = await hourly(selected, fromDate, toDate);
    const m = new Map(rows.map((r) => [r.hour, r.count]));
    values = Array.from({ length: 24 }, (_, h) => m.get(h) ?? 0);
  }

  onMount(async () => {
    channels = await listChannels();
    selected = channels[0]?.id ?? null;
    await reload();
  });
</script>

<div class="space-y-4">
  <div class="flex flex-wrap items-center gap-2">
    <select
      class="glass pressable rounded-xl px-3 py-1"
      bind:value={selected}
      on:change={reload}>
      {#each channels as c}
        <option value={c.id}>{c.name}</option>
      {/each}
    </select>
    <input
      class="glass rounded-xl px-3 py-1"
      type="date"
      bind:value={fromDate}
      on:change={reload} />
    <span class="text-neutral-500">to</span>
    <input
      class="glass rounded-xl px-3 py-1"
      type="date"
      bind:value={toDate}
      on:change={reload} />
  </div>
  <div class="glass-strong shadow-glass rounded-3xl p-4">
    <BarPlot
      {labels}
      {values}
      accent={channels.find((c) => c.id === selected)?.color ?? '#7dd3fc'} />
  </div>
</div>
