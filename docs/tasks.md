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
- [ ] Implement Tray Icon (Tauri v2)
    - [ ] Create Tray Icon.
    - [ ] Left-click (or double-click) to toggle visibility.
    - [ ] Right-click menu with "Quit".
    - [ ] Handle Window Close event (intercept and hide window instead of closing).
- [ ] Integrate Input Monitor
    - [ ] Spawn `InputMonitor` thread from `main.rs` (or `lib.rs`).
    - [ ] Implement Shared State (`RwLock<Snapshot>` or `ArcSwap`).
    - [ ] Implement low-frequency event loop (30-60Hz) to emit `state-update` events to Frontend.
- [ ] Implement IPC Commands (`commands.rs`)
    - [ ] `get_snapshot`: Return full state.
    - [ ] `start_learning(logical_key)` / `cancel_learning`.
    - [ ] `set_binding(logical_key, physical)`: Update binding, handle duplicates (unbind old).
    - [ ] `reset_to_default_mapping`: Reset to preset.
    - [ ] `set_target_controller(index)`.
    - [ ] `reset_stats(logical_key)` / `bulk_apply(model_id, keys[])`.
    - [ ] `set_switch_model(logical_key, model_id)`.
- [ ] Implement Events
    - [ ] `state-update`: Periodic UI update.
    - [ ] `game-started` / `game-exited`: Process monitor events.
    - [ ] `save-succeeded` / `save-failed`.
    - [ ] `connection-changed`.

## Phase 2.5: Data Migration
- [ ] Implement Data Migration Logic
    - [ ] Define new data path (`%LOCALAPPDATA%/SwitchLifeManager/`).
    - [ ] Check for existing `eframe` data on startup.
    - [ ] Copy/Migrate data to new location.
    - [ ] Handle `schema_version`.

## Phase 3: Frontend Implementation (React)
- [ ] Setup UI Framework
    - [ ] Install UI library (Mantine or shadcn/ui + Tailwind).
    - [ ] Setup basic layout (Sidebar/Tabs).
- [ ] Implement Dashboard
    - [ ] Lifetime Progress Bars (Green/Yellow/Red).
    - [ ] Switch Model selection.
    - [ ] Bulk Actions (Reset Stats, Apply Model).
- [ ] Implement Settings (Key Config)
    - [ ] List Keys and current bindings.
    - [ ] "Set" button -> Learning Mode modal.
    - [ ] "Reset to Default" button.
- [ ] Implement Input Tester
    - [ ] Visual representation of buttons.
    - [ ] Real-time feedback (from `state-update`).
    - [ ] Chatter visualization.
- [ ] Implement Report View
    - [ ] Auto-show on `game-exited`.
    - [ ] Display session stats.
- [ ] IPC Integration
    - [ ] Create API wrapper for Tauri commands.
    - [ ] Listen for Tauri events.

## Phase 4: Testing & Release
- [ ] Verify Tray Functionality (Hide/Show, Quit, no ghost process).
- [ ] Verify Input Monitoring (DirectInput/XInput, Hotplug).
- [ ] Verify Key Config (Learning, Duplicate handling).
- [ ] Verify Data Persistence (Atomic Save, Migration).
- [ ] Verify Process Monitoring (Game start/exit detection).
- [ ] Build Release (`cargo tauri build`).
