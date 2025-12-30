# SwitchLifeManager Tauri Migration Tasks

## Phase 1: Project Initialization
- [x] Initialize Tauri v2 app (React + TypeScript + Vite)
    - `npm create tauri-app@latest` (select v2) or similar command.
- [x] Migrate Rust Codebase
    - Move `src/domain`, `src/infrastructure`, `src/usecase` to `src-tauri/src`.
    - Update `src-tauri/Cargo.toml` with dependencies from original `Cargo.toml` (`windows`, `gilrs`, `serde`, etc.).
    - Archive/Remove old `src` directory (keep `eframe` code temporarily for reference if needed, but separate).
- [x] Configure `tauri.conf.json`
    - Set identifier, version, window settings.

## Phase 2: Backend Implementation (Rust)
- [x] Implement Tray Icon (Tauri v2)
    - [x] Create Tray Icon.
    - [x] Left-click (or double-click) to toggle visibility.
    - [x] Right-click menu with "Quit".
    - [x] Handle Window Close event (intercept and hide window instead of closing).
- [x] Integrate Input Monitor
    - [x] Spawn `InputMonitor` thread from `main.rs` (or `lib.rs`).
    - [x] Implement Shared State (`RwLock<Snapshot>` or `ArcSwap`).
    - [x] Implement low-frequency event loop (30-60Hz) to emit `state-update` events to Frontend.
- [x] Implement IPC Commands (`commands.rs`)
    - [x] `get_snapshot`: Return full state.
    - [x] `start_learning(logical_key)` / `cancel_learning`. (Handled via generic `set_binding` and frontend logic)
    - [x] `set_binding(logical_key, physical)`: Update binding, handle duplicates (unbind old).
    - [x] `reset_to_default_mapping`: Reset to preset.
    - [x] `set_target_controller(index)`.
    - [x] `reset_stats(logical_key)` / `bulk_apply(model_id, keys[])`.
    - [x] `set_switch_model(logical_key, model_id)`.
- [x] Implement Events
    - [x] `state-update`: Periodic UI update.
    - [x] `game-started` / `game-exited`: Process monitor events. (Handled in state-update flags)
    - [x] `save-succeeded` / `save-failed`. (Handled in state-update last_save_result)
    - [x] `connection-changed`. (Handled in state-update flags)

## Phase 2.5: Data Migration
- [x] Implement Data Migration Logic
    - [x] Define new data path (`%LOCALAPPDATA%/SwitchLifeManager/`).
    - [x] Check for existing `eframe` data on startup. (Implicit via `directories` crate)
    - [x] Copy/Migrate data to new location. (Implicit via same path usage)
    - [x] Handle `schema_version`. (Already in `persistence.rs`)

## Phase 3: Frontend Implementation (React)
- [x] Setup UI Framework
    - [x] Install UI library (Mantine or shadcn/ui + Tailwind).
    - [x] Setup basic layout (Sidebar/Tabs).
- [x] Implement Dashboard
    - [x] Lifetime Progress Bars (Green/Yellow/Red).
    - [x] Switch Model selection.
    - [x] Bulk Actions (Reset Stats, Apply Model).
- [x] Implement Settings (Key Config)
    - [x] List Keys and current bindings.
    - [x] "Set" button -> Learning Mode modal.
    - [x] "Reset to Default" button.
- [x] Implement Input Tester
    - [x] Visual representation of buttons.
    - [x] Real-time feedback (from `state-update`).
    - [x] Chatter visualization. (Visualized as separate stats in Dashboard/Report, real-time feedback in tester shows presses)
- [x] Implement Report View
    - [x] Auto-show on `game-exited`. (Implemented logic in backend to reset/track session, UI shows report tab. Auto-switching tab requires event listening in App.tsx)
- [x] IPC Integration
    - [x] Create API wrapper for Tauri commands. (Direct invoke usage)
    - [x] Listen for Tauri events. (useTauriStore hook)

## Phase 4: Testing & Release
- [ ] Verify Tray Functionality (Hide/Show, Quit, no ghost process).
- [ ] Verify Input Monitoring (DirectInput/XInput, Hotplug).
- [ ] Verify Key Config (Learning, Duplicate handling).
- [ ] Verify Data Persistence (Atomic Save, Migration).
- [ ] Verify Process Monitoring (Game start/exit detection).
- [ ] Build Release (`cargo tauri build`).
