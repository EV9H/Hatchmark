<script lang="ts">
  import { onMount } from 'svelte';
  import Heatmap from '$lib/components/Heatmap.svelte';
  import { listChannels, heatmap } from '$lib/tauri';
  import type { Channel } from '$lib/types';

  const DAYS = 7 * 52;
  let channels: Channel[] = [];
  let data: Record<number, { date: string; count: number }[]> = {};

  function fillRange(rows: { date: string; count: number }[]) {
    const map = new Map(rows.map((r) => [r.date, r.count]));
    const out: { date: string; count: number }[] = [];
    const today = new Date();
    for (let i = DAYS - 1; i >= 0; i--) {
      const d = new Date(today);
      d.setDate(today.getDate() - i);
      const iso = d.toISOString().slice(0, 10);
      out.push({ date: iso, count: map.get(iso) ?? 0 });
    }
    return out;
  }

  onMount(async () => {
    channels = await listChannels();
    const next: Record<number, { date: string; count: number }[]> = {};
    for (const c of channels) {
      next[c.id] = fillRange(await heatmap(c.id, DAYS));
    }
    data = next;
  });
</script>

<div class="space-y-6">
  {#each channels as c (c.id)}
    <section>
      <div class="mb-2 flex items-baseline justify-between">
        <span class="text-sm text-neutral-500">{c.name}</span>
        <span class="text-xs text-neutral-400">{DAYS} days</span>
      </div>
      <div class="glass-strong shadow-glass rounded-3xl p-4">
        <Heatmap
          dates={(data[c.id] ?? []).map((r) => r.date)}
          values={(data[c.id] ?? []).map((r) => r.count)}
          accent={c.color} />
      </div>
    </section>
  {/each}
</div>
