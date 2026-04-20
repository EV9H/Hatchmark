<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import { initDaemonStream } from '$lib/daemon';

  onMount(initDaemonStream);
</script>

<div class="flex h-screen flex-col">
  <TitleBar />
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-auto p-6">
      <slot />
    </main>
  </div>
</div>
