<script lang="ts">
  import { onMount } from 'svelte';
  import BarPlot from '$lib/components/BarPlot.svelte';
  import { listChannels, history } from '$lib/tauri';
  import type { Channel } from '$lib/types';

  let channels: Channel[] = [];
  let selected: number | null = null;
  let days = 30;
  let data: { labels: string[]; values: number[] } = { labels: [], values: [] };

  async function reload() {
    if (selected == null) return;
    const rows = await history(selected, days);
    data = {
      labels: rows.map((r) => r.date),
      values: rows.map((r) => r.count)
    };
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
    <select
      class="glass pressable rounded-xl px-3 py-1"
      bind:value={days}
      on:change={reload}>
      {#each [7, 14, 30, 60, 90] as d}
        <option value={d}>{d} days</option>
      {/each}
    </select>
  </div>
  <div class="glass-strong shadow-glass rounded-3xl p-4">
    <BarPlot
      labels={data.labels}
      values={data.values}
      accent={channels.find((c) => c.id === selected)?.color ?? '#7dd3fc'} />
  </div>
</div>
