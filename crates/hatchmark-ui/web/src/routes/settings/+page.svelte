<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listChannels,
    createChannel,
    updateChannel,
    deleteChannel,
    listLayers,
    createLayer,
    renameLayer,
    deleteLayer,
    getCurrentLayer,
    setCurrentLayer,
    listBindings,
    upsertBinding,
    deleteBinding,
    reloadDaemon,
    revealDataDir,
    getSetting,
    setSetting,
    exportCsv,
    listProfiles,
    getActiveProfile,
    createProfile,
    deleteProfile,
    switchProfile,
    clearEvents
  } from '$lib/tauri';
  import { open as openShell } from '@tauri-apps/plugin-shell';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import KeyCaptureButton from '$lib/components/KeyCaptureButton.svelte';
  import { bindingConflicts } from '$lib/daemon';
  import type { Channel, Layer, Binding } from '$lib/types';

  type Tab = 'channels' | 'layers' | 'bindings' | 'profiles' | 'data';
  let tab: Tab = 'channels';

  let channels: Channel[] = [];
  let layers: Layer[] = [];
  let currentLayerId = 1;
  let bindings: Binding[] = [];

  let autostartOn = true;
  let toastOn = false;

  let profiles: string[] = [];
  let activeProfile = 'default';
  let newProfileName = '';

  async function reloadAll() {
    [channels, layers, currentLayerId, profiles, activeProfile] = await Promise.all([
      listChannels(),
      listLayers(),
      getCurrentLayer(),
      listProfiles(),
      getActiveProfile()
    ]);
    bindings = await listBindings(currentLayerId);
    autostartOn = (await getSetting('autostart')) === 'true';
    toastOn = (await getSetting('toast_enabled')) === 'true';
  }

  async function doSwitchProfile(name: string) {
    if (name === activeProfile) return;
    await switchProfile(name);
    await reloadAll();
  }

  async function doCreateProfile() {
    const name = newProfileName.trim();
    if (!name) return;
    try {
      await createProfile(name);
      newProfileName = '';
      await reloadAll();
    } catch (e) {
      alert(String(e));
    }
  }

  async function doDeleteProfile(name: string) {
    if (!confirm(`Delete profile "${name}" and all its data? This cannot be undone.`)) return;
    try {
      await deleteProfile(name);
      await reloadAll();
    } catch (e) {
      alert(String(e));
    }
  }

  async function doClearEvents() {
    if (!confirm(`Clear all counts in profile "${activeProfile}"? Channels and bindings are kept; only the press history is deleted.`)) return;
    const n = await clearEvents();
    alert(`Cleared ${n} events.`);
    await reloadAll();
  }

  onMount(reloadAll);

  async function addChannel() {
    await createChannel('Channel', '#7dd3fc', null, null);
    await reloadAll();
  }

  async function saveChannel(c: Channel) {
    await updateChannel(c);
    await reloadAll();
  }

  async function removeChannel(id: number) {
    await deleteChannel(id);
    await reloadAll();
  }

  async function addLayer() {
    const name = prompt('Layer name', 'New Layer');
    if (!name) return;
    await createLayer(name);
    await reloadAll();
  }

  async function switchLayer(id: number) {
    await setCurrentLayer(id);
    currentLayerId = id;
    bindings = await listBindings(id);
    await reloadDaemon();
  }

  async function setIncrement(key: string, channelId: number) {
    await upsertBinding({
      layer_id: currentLayerId,
      key_code: key,
      action: { kind: 'increment', channel_id: channelId }
    });
    bindings = await listBindings(currentLayerId);
    await reloadDaemon();
  }

  async function setSwitch(key: string, target: number) {
    await upsertBinding({
      layer_id: currentLayerId,
      key_code: key,
      action: { kind: 'switch_layer', target_layer_id: target }
    });
    bindings = await listBindings(currentLayerId);
    await reloadDaemon();
  }

  async function removeBinding(key: string) {
    await deleteBinding(currentLayerId, key);
    bindings = await listBindings(currentLayerId);
    await reloadDaemon();
  }

  async function captureAndBind(k: string) {
    if (channels.length === 0) {
      alert('Create a channel first.');
      return;
    }
    await setIncrement(k, channels[0].id);
  }

  async function toggleAutostart() {
    autostartOn = !autostartOn;
    await setSetting('autostart', autostartOn ? 'true' : 'false');
  }

  async function toggleToast() {
    toastOn = !toastOn;
    await setSetting('toast_enabled', toastOn ? 'true' : 'false');
  }

  async function revealData() {
    const dir = await revealDataDir();
    await openShell(dir);
  }

  async function doExport() {
    const path = await saveDialog({
      defaultPath: 'multi-channel-counter.csv',
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    });
    if (!path) return;
    const n = await exportCsv(path as string);
    alert(`Exported ${n} rows to\n${path}`);
  }

  function conflictFor(key: string): string | null {
    return $bindingConflicts[`${currentLayerId}:${key}`] ?? null;
  }
</script>

<div class="space-y-4">
  <div class="flex gap-2 text-sm">
    {#each ['channels', 'layers', 'bindings', 'profiles', 'data'] as t}
      <button
        class="pressable rounded-xl px-3 py-1 {tab === t
          ? 'bg-white/70 shadow-glass dark:bg-white/10'
          : 'hover:bg-white/40'}"
        on:click={() => (tab = t as Tab)}>
        {t}
      </button>
    {/each}
  </div>

  {#if tab === 'channels'}
    <div class="space-y-2">
      {#each channels as c (c.id)}
        <div class="glass-strong shadow-glass flex items-center gap-2 rounded-2xl p-3">
          <input
            class="h-8 w-8 cursor-pointer rounded"
            type="color"
            bind:value={c.color}
            on:change={() => saveChannel(c)} />
          <input
            class="flex-1 bg-transparent outline-none"
            bind:value={c.name}
            on:change={() => saveChannel(c)} />
          <input
            class="w-20 bg-transparent text-right tabular-nums"
            type="number"
            placeholder="goal"
            bind:value={c.daily_goal}
            on:change={() => saveChannel(c)} />
          <input
            class="w-20 bg-transparent text-right tabular-nums"
            type="number"
            placeholder="limit"
            bind:value={c.daily_limit}
            on:change={() => saveChannel(c)} />
          <button
            class="pressable rounded-xl px-2 py-1 text-sm text-red-500 hover:bg-red-500/10"
            on:click={() => removeChannel(c.id)}>
            Delete
          </button>
        </div>
      {/each}
      <button class="glass pressable rounded-xl px-3 py-2" on:click={addChannel}>
        + Add channel
      </button>
    </div>
  {:else if tab === 'layers'}
    <div class="space-y-2">
      {#each layers as l (l.id)}
        <div class="glass-strong shadow-glass flex items-center gap-2 rounded-2xl p-3">
          <input
            class="flex-1 bg-transparent outline-none"
            value={l.name}
            on:change={(e) =>
              renameLayer(l.id, (e.target as HTMLInputElement).value).then(reloadAll)} />
          <button
            class="pressable rounded-xl px-2 py-1 text-sm {currentLayerId === l.id
              ? 'bg-emerald-400/70 text-white'
              : 'bg-white/50'}"
            on:click={() => switchLayer(l.id)}>
            {currentLayerId === l.id ? 'Active' : 'Set active'}
          </button>
          <button
            class="pressable rounded-xl px-2 py-1 text-sm text-red-500 hover:bg-red-500/10"
            on:click={() => deleteLayer(l.id).then(reloadAll).catch((e) => alert(e))}>
            Delete
          </button>
        </div>
      {/each}
      <button class="glass pressable rounded-xl px-3 py-2" on:click={addLayer}>
        + Add layer
      </button>
    </div>
  {:else if tab === 'bindings'}
    <div class="space-y-2">
      <div class="text-xs text-neutral-500">
        Layer: {layers.find((l) => l.id === currentLayerId)?.name ?? '?'}
      </div>
      {#each bindings as b (b.key_code)}
        <div class="glass-strong shadow-glass flex items-center gap-2 rounded-2xl p-3">
          <div class="w-14 font-mono text-sm">{b.key_code}</div>
          {#if b.action.kind === 'increment'}
            <select
              class="flex-1 bg-transparent"
              on:change={(e) =>
                setIncrement(b.key_code, Number((e.target as HTMLSelectElement).value))}
              value={b.action.channel_id}>
              {#each channels as c}
                <option value={c.id}>{c.name}</option>
              {/each}
            </select>
          {:else}
            <select
              class="flex-1 bg-transparent"
              on:change={(e) =>
                setSwitch(b.key_code, Number((e.target as HTMLSelectElement).value))}
              value={b.action.target_layer_id}>
              {#each layers as l}
                <option value={l.id}>Switch → {l.name}</option>
              {/each}
            </select>
          {/if}
          {#if conflictFor(b.key_code)}
            <span
              class="rounded-full bg-red-500/15 px-2 py-0.5 text-[10px] font-medium text-red-500"
              title={conflictFor(b.key_code) ?? ''}>
              conflict
            </span>
          {/if}
          <button
            class="pressable rounded-xl px-2 py-1 text-sm text-red-500 hover:bg-red-500/10"
            on:click={() => removeBinding(b.key_code)}>
            Remove
          </button>
        </div>
      {/each}

      <div class="glass-strong shadow-glass flex items-center gap-2 rounded-2xl p-3">
        <KeyCaptureButton onchange={captureAndBind} />
        <span class="text-xs text-neutral-500">
          Press F13-F24. Default action is increment first channel; edit above.
        </span>
      </div>
    </div>
  {:else if tab === 'profiles'}
    <div class="space-y-2">
      <div class="text-xs text-neutral-500">
        Each profile is a separate database. Active profile:
        <span class="font-medium">{activeProfile}</span>
      </div>
      {#each profiles as p (p)}
        <div class="glass-strong shadow-glass flex items-center gap-2 rounded-2xl p-3">
          <div class="flex-1 font-medium">
            {p}
            {#if p === activeProfile}
              <span class="ml-2 rounded-full bg-emerald-400/30 px-2 py-0.5 text-[10px] text-emerald-600 dark:text-emerald-300">active</span>
            {/if}
            {#if p === 'default'}
              <span class="ml-2 text-[10px] text-neutral-500">protected</span>
            {/if}
          </div>
          {#if p !== activeProfile}
            <button
              class="pressable rounded-xl bg-white/50 px-2 py-1 text-sm dark:bg-white/10"
              on:click={() => doSwitchProfile(p)}>
              Switch
            </button>
          {/if}
          {#if p !== 'default' && p !== activeProfile}
            <button
              class="pressable rounded-xl px-2 py-1 text-sm text-red-500 hover:bg-red-500/10"
              on:click={() => doDeleteProfile(p)}>
              Delete
            </button>
          {/if}
        </div>
      {/each}

      <div class="glass-strong shadow-glass flex items-center gap-2 rounded-2xl p-3">
        <input
          class="flex-1 bg-transparent outline-none"
          placeholder="new profile name (letters, digits, - _)"
          bind:value={newProfileName}
          on:keydown={(e) => e.key === 'Enter' && doCreateProfile()} />
        <button
          class="pressable glass rounded-xl px-3 py-1 text-sm"
          on:click={doCreateProfile}>
          + Create
        </button>
      </div>

      <div class="glass-strong shadow-glass mt-4 flex items-center gap-3 rounded-2xl p-3">
        <div class="flex-1">
          <div class="text-sm font-medium text-red-500">Clear counts</div>
          <div class="text-xs text-neutral-500">
            Deletes every event in the active profile ("{activeProfile}"). Channels, layers, and bindings stay.
          </div>
        </div>
        <button
          class="pressable rounded-xl bg-red-500/15 px-3 py-2 text-sm text-red-600 hover:bg-red-500/25 dark:text-red-400"
          on:click={doClearEvents}>
          Clear
        </button>
      </div>
    </div>
  {:else}
    <div class="space-y-2">
      <label
        class="glass-strong shadow-glass flex items-center gap-3 rounded-2xl p-3">
        <input type="checkbox" checked={autostartOn} on:change={toggleAutostart} />
        Start daemon on Windows login
      </label>
      <label
        class="glass-strong shadow-glass flex items-center gap-3 rounded-2xl p-3">
        <input type="checkbox" checked={toastOn} on:change={toggleToast} />
        Show toast on each press
      </label>
      <div class="flex flex-wrap gap-2">
        <button class="glass pressable rounded-xl px-3 py-2" on:click={revealData}>
          Reveal data folder
        </button>
        <button class="glass pressable rounded-xl px-3 py-2" on:click={doExport}>
          Export CSV…
        </button>
      </div>
    </div>
  {/if}
</div>
