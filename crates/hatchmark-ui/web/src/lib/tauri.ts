import { invoke } from '@tauri-apps/api/core';
import type {
  Binding,
  Channel,
  DailyCount,
  DateCount,
  HourCount,
  Layer,
  Rollup
} from './types';

export const listChannels = () => invoke<Channel[]>('list_channels');
export const createChannel = (
  name: string,
  color: string,
  dailyGoal: number | null,
  dailyLimit: number | null
) => invoke<number>('create_channel', { name, color, dailyGoal, dailyLimit });
export const updateChannel = (channel: Channel) =>
  invoke<void>('update_channel', { channel });
export const deleteChannel = (id: number) => invoke<void>('delete_channel', { id });

export const listLayers = () => invoke<Layer[]>('list_layers');
export const createLayer = (name: string) => invoke<number>('create_layer', { name });
export const renameLayer = (id: number, name: string) =>
  invoke<void>('rename_layer', { id, name });
export const deleteLayer = (id: number) => invoke<void>('delete_layer', { id });
export const getCurrentLayer = () => invoke<number>('get_current_layer');
export const setCurrentLayer = (id: number) => invoke<void>('set_current_layer', { id });

export const listBindings = (layerId: number) =>
  invoke<Binding[]>('list_bindings', { layerId });
export const upsertBinding = (binding: Binding) =>
  invoke<void>('upsert_binding', { binding });
export const deleteBinding = (layerId: number, keyCode: string) =>
  invoke<void>('delete_binding', { layerId, keyCode });

export const todayCounts = () => invoke<DailyCount[]>('today_counts');
export const history = (channelId: number, days: number) =>
  invoke<DateCount[]>('history', { channelId, days });
export const heatmap = (channelId: number, days: number) =>
  invoke<DateCount[]>('heatmap', { channelId, days });
export const hourly = (channelId: number, fromDate: string, toDate: string) =>
  invoke<HourCount[]>('hourly', { channelId, fromDate, toDate });
export const rollup = (channelId: number, days: number) =>
  invoke<Rollup>('rollup', { channelId, days });

export const adjust = (channelId: number, delta: 1 | -1) =>
  invoke<void>('adjust', { channelId, delta });
export const exportCsv = (path: string) => invoke<number>('export_csv', { path });

export const getSetting = (key: string) => invoke<string | null>('get_setting', { key });
export const setSetting = (key: string, value: string) =>
  invoke<void>('set_setting', { key, value });

export const reloadDaemon = () => invoke<void>('reload_daemon');
export const revealDataDir = () => invoke<string>('reveal_data_dir');
