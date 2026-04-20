<script lang="ts">
  import { onMount } from 'svelte';
  import { listChannels, rollup } from '$lib/tauri';
  import type { Channel } from '$lib/types';

  let channels: Channel[] = [];
  let windowDays = 7;
  type Row = { channel: Channel; total: number; avg: number; prev: number };
  let rows: Row[] = [];

  async function reload() {
    rows = await Promise.all(
      channels.map(async (c) => {
        const r = await rollup(c.id, windowDays);
        return {
          channel: c,
          total: r.total,
          avg: r.daily_average,
          prev: r.previous_total
        };
      })
    );
  }

  onMount(async () => {
    channels = await listChannels();
    await reload();
  });
</script>

<div class="space-y-4">
  <div class="flex items-center gap-2">
    <select
      class="glass pressable rounded-xl px-3 py-1"
      bind:value={windowDays}
      on:change={reload}>
      {#each [7, 14, 30, 90] as w}
        <option value={w}>{w}-day window</option>
      {/each}
    </select>
  </div>
  <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
    {#each rows as r (r.channel.id)}
      <div class="glass-strong shadow-glass rounded-3xl p-5">
        <div class="flex items-baseline justify-between">
          <span class="text-xs uppercase tracking-wider text-neutral-500">
            {r.channel.name}
          </span>
          <span
            class="tabular-nums text-3xl font-semibold"
            style="color: {r.channel.color}">
            {r.total}
          </span>
        </div>
        <div class="mt-2 text-sm text-neutral-500">
          avg <span class="tabular-nums">{r.avg.toFixed(1)}</span>/day
          <span class="ml-2">
            {#if r.total > r.prev}
              <span class="text-emerald-600 dark:text-emerald-400">▲</span>
            {:else if r.total < r.prev}
              <span class="text-red-500">▼</span>
            {:else}
              <span>·</span>
            {/if}
            <span class="tabular-nums">{Math.abs(r.total - r.prev)}</span>
            vs prior {windowDays}d
          </span>
        </div>
      </div>
    {/each}
  </div>
</div>
