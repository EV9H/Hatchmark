<script lang="ts">
  import { onMount } from 'svelte';
  import ChannelCard from '$lib/components/ChannelCard.svelte';
  import { listChannels, todayCounts } from '$lib/tauri';
  import { lastDaemonMsg } from '$lib/daemon';
  import type { Channel } from '$lib/types';

  let channels: Channel[] = [];
  let counts: Record<number, number> = {};
  let mounted = false;

  async function load() {
    channels = await listChannels();
    const today = await todayCounts();
    counts = Object.fromEntries(today.map((t) => [t.channel_id, t.count]));
  }

  onMount(async () => {
    await load();
    mounted = true;
  });

  $: if ($lastDaemonMsg?.type === 'increment') {
    counts = {
      ...counts,
      [$lastDaemonMsg.channel_id]: $lastDaemonMsg.new_total_today
    };
  }
</script>

<div
  class="grid grid-cols-1 gap-4 transition-all duration-300 ease-out sm:grid-cols-2 xl:grid-cols-3"
  style="opacity: {mounted ? 1 : 0}; transform: translateY({mounted ? 0 : 4}px);">
  {#each channels as c (c.id)}
    <ChannelCard channel={c} count={counts[c.id] ?? 0} />
  {/each}
  {#if channels.length === 0}
    <div class="glass col-span-full rounded-3xl p-8 text-center text-neutral-500">
      No channels yet. Open
      <a class="underline" href="/settings">Settings</a> to add one.
    </div>
  {/if}
</div>
