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
}

export interface ButtonMap {
  profile_name: string;
  bindings: Record<string, number>; // LogicalKey -> Button Mask (u16)
}

export interface LastSaveResult {
  success: boolean;
  message: string;
  timestamp: string; // SystemTime serializes to string or object? serde default is ISO string usually if configured, or struct.
  // Wait, SystemTime default serialization in serde is... complicated.
  // It usually serializes as `{ secs_since_epoch: ..., nanos: ... }` unless `serde_with` or configuration used.
  // In Rust code: `pub timestamp: SystemTime`.
  // I should check if I need to adjust Rust side to be friendly (e.g., using `chrono` or `serde_with::TimestampSeconds`).
  // For now, assume it might be complex object or string. I'll check Rust side later.
  // Let's type it as `any` for now or check serde defaults.
}

export interface MonitorSharedState {
  is_connected: boolean;
  is_game_running: boolean;
  config: AppConfig;
  profile_name: string;
  bindings: Record<string, number>;
  switches: Record<string, SwitchData>;
  current_pressed_keys: LogicalKey[]; // HashSet serializes to list
  raw_button_state: number;
  last_status_message: string | null;
  last_save_result: LastSaveResult | null;
}
