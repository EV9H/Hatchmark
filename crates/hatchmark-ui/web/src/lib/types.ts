export type Channel = {
  id: number;
  name: string;
  color: string;
  daily_goal: number | null;
  daily_limit: number | null;
  sort_order: number;
};

export type Layer = {
  id: number;
  name: string;
  sort_order: number;
};

export type BindingAction =
  | { kind: 'increment'; channel_id: number }
  | { kind: 'switch_layer'; target_layer_id: number };

export type Binding = {
  layer_id: number;
  key_code: string;
  action: BindingAction;
};

export type DailyCount = { channel_id: number; count: number };
export type DateCount = { date: string; count: number };
export type HourCount = { hour: number; count: number };
export type Rollup = {
  channel_id: number;
  total: number;
  daily_average: number;
  previous_total: number;
};

export type DaemonMsg =
  | { type: 'hello'; version: string; current_layer_id: number }
  | { type: 'increment'; channel_id: number; new_total_today: number; timestamp: string }
  | { type: 'layer_changed'; current_layer_id: number }
  | { type: 'binding_conflict'; layer_id: number; key_code: string; reason: string }
  | { type: 'channels_updated'; channels: Channel[] };
