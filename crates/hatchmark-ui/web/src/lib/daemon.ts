import { listen } from '@tauri-apps/api/event';
import { writable } from 'svelte/store';
import type { DaemonMsg } from './types';

export const lastDaemonMsg = writable<DaemonMsg | null>(null);
export const bindingConflicts = writable<Record<string, string>>({});

let started = false;

export async function initDaemonStream() {
  if (started) return;
  started = true;
  await listen<DaemonMsg>('daemon-msg', (ev) => {
    const msg = ev.payload;
    lastDaemonMsg.set(msg);
    if (msg.type === 'binding_conflict') {
      bindingConflicts.update((all) => ({
        ...all,
        [`${msg.layer_id}:${msg.key_code}`]: msg.reason
      }));
    }
  });
}
