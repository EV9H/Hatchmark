<script lang="ts">
  import { adjust, todayCounts } from '$lib/tauri';
  import type { Channel } from '$lib/types';
  export let channel: Channel;
  export let count: number;

  let expanded = false;

  async function bump(delta: 1 | -1) {
    await adjust(channel.id, delta);
    const all = await todayCounts();
    count = all.find((r) => r.channel_id === channel.id)?.count ?? 0;
  }

  $: goalPct =
    channel.daily_goal && channel.daily_goal > 0
      ? Math.min(1, count / channel.daily_goal)
      : null;
  $: overLimit =
    channel.daily_limit != null && count > channel.daily_limit;
</script>

<div
  class="glass-strong shadow-glass pressable rounded-3xl p-5 hover:shadow-glassHi"
  style="--accent: {channel.color}">
  <button
    type="button"
    class="no-drag w-full text-left"
    on:click={() => (expanded = !expanded)}>
    <div class="flex items-baseline justify-between">
      <span class="text-xs uppercase tracking-wider text-neutral-500">
        {channel.name}
      </span>
      <span
        class="tabular-nums text-4xl font-semibold"
        style="color: var(--accent)">
        {count}
      </span>
    </div>
    {#if goalPct != null}
      <div class="mt-3 h-1.5 w-full rounded-full bg-black/10 dark:bg-white/10">
        <div
          class="h-full rounded-full transition-all duration-300"
          style="width: {Math.round(goalPct * 100)}%; background: var(--accent)">
        </div>
      </div>
      <div class="mt-1 text-[10px] text-neutral-500">
        Goal {channel.daily_goal}
      </div>
    {/if}
    {#if overLimit}
      <div class="mt-2 text-[10px] font-medium text-red-500">
        over limit {channel.daily_limit}
      </div>
    {/if}
  </button>
  {#if expanded}
    <div class="no-drag mt-4 flex gap-2">
      <button
        class="pressable glass rounded-xl px-3 py-1 text-sm"
        on:click={() => bump(-1)}>
        &#8722;
      </button>
      <button
        class="pressable glass rounded-xl px-3 py-1 text-sm"
        on:click={() => bump(1)}>
        +
      </button>
    </div>
  {/if}
</div>
