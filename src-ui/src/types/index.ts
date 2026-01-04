export type LogicalKey =
  | "Key1" | "Key2" | "Key3" | "Key4" | "Key5" | "Key6" | "Key7"
  | "E1" | "E2" | "E3" | "E4"
  | string; // For "Other-{id}"

export interface AppConfig {
  target_controller_index: number;
  input_method: "XInput" | "DirectInput";
  chatter_threshold_ms: number;
  polling_rate_ms_connected: number;
  polling_rate_ms_disconnected: number;
  target_process_name: string;
}

export interface ButtonStats {
  total_presses: number;
  total_releases: number;
  total_chatters: number;
  total_chatter_releases: number;
  last_session_presses: number;
  last_session_chatters: number;
  last_session_chatter_releases: number;
}

export interface SwitchData {
  switch_model_id: string;
  stats: ButtonStats;
  last_replaced_at: string | null;
}

export interface ButtonMap {
  profile_name: string;
  bindings: Record<string, number>; // LogicalKey -> Button Mask (u32)
}

export interface SwitchHistoryEntry {
  date: string;
  key: LogicalKey;
  old_model_id: string;
  new_model_id: string;
  previous_stats: ButtonStats;
  event_type: string;
}

export interface LastSaveResult {
  success: boolean;
  message: string;
  timestamp: string;
}

export interface MonitorSharedState {
  is_connected: boolean;
  is_game_running: boolean;
  config: AppConfig;
  profile_name: string;
  bindings: Record<string, number>;
  switches: Record<string, SwitchData>;
  switch_history: SwitchHistoryEntry[];
  current_pressed_keys: LogicalKey[]; // HashSet serializes to list
  raw_button_state: number;
  last_status_message: string | null;
  last_save_result: LastSaveResult | null;
}
