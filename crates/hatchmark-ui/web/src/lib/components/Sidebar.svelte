<script lang="ts">
  import { page } from '$app/stores';

  const items = [
    { href: '/today', label: 'Today' },
    { href: '/history', label: 'History' },
    { href: '/heatmap', label: 'Heatmap' },
    { href: '/hourly', label: 'Hourly' },
    { href: '/rollups', label: 'Rollups' },
    { href: '/settings', label: 'Settings' }
  ];
</script>

<nav class="glass shadow-glass m-2 flex w-48 flex-col gap-1 rounded-3xl p-2">
  {#each items as item}
    <a
      href={item.href}
      class="pressable rounded-2xl px-3 py-2 text-sm {$page.url.pathname.startsWith(item.href)
        ? 'bg-white/70 shadow-glass dark:bg-white/10'
        : 'hover:bg-white/40 dark:hover:bg-white/5'}">
      {item.label}
    </a>
  {/each}
</nav>
