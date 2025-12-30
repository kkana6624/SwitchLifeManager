# SwitchLifeManager Tauri移行計画書

**作成日**: 2025年12月30日
**現状ステータス**: 計画策定完了

## 1. 背景と経緯

### 1.1 プロジェクトの目的
Beatmania IIDXコントローラー等のスイッチ寿命を管理するため、高精度の入力監視と寿命可視化を行うアプリケーションを開発中。GUIフレームワークとして当初は `egui` (eframe) を採用していた。

### 1.2 直面した課題
「タスクトレイ常駐（最小化してバックグラウンド動作）」機能の実装において、`eframe` の制約により以下の深刻な問題が発生した。

1.  **ループ停止問題**: Windows環境において `eframe` ウィンドウを非表示 (`Visible(false)`) にすると、メインスレッドのイベントループがOS仕様により停止またはスリープ状態となり、バックグラウンド処理や復帰処理が正しく行われない。
2.  **復帰の不安定さ**: 停止したループを外部から再開させる手段が標準ではなく、Win32 API (`ShowWindow`) を直接叩くなどのハックが必要となった。
3.  **終了時の挙動**: トレイからの終了処理において、ウィンドウが一瞬表示される（フラッシュ）問題や、プロセスが完全に終了しない（ゴーストアイコン）問題が発生した。

### 1.3 PoCによる検証結果
Win32 API (`PostThreadMessage`, `WM_NULL`, `WM_QUIT`) を駆使したPoC（概念実証）を実施し、以下の成果を得た。

*   **成果**: `WM_NULL` メッセージによるループの叩き起こしや、`WM_QUIT` による強制終了を用いることで、一応の要件（格納・復帰・静かな終了）は実現可能であることを確認した。
*   **結論**: 実現は可能だが、フレームワークの想定外の使い方（Win32ハック）を多用しており、**保守性、安定性、将来性の観点からリスクが高い** と判断した。また、`eframe` は本来タスクトレイ機能をスコープ外としている。

## 2. 移行方針: Tauriの採用

上記課題の根本解決およびアプリケーションの品質向上のため、GUIフレームワークを **Tauri** へ移行する。

### 2.1 Tauri採用のメリット
1.  **タスクトレイの公式サポート**: `SystemTray` APIが標準提供されており、Win32ハックなしで安定した格納・復帰・メニュー操作が可能。
2.  **イベントループの安定性**: バックグラウンド動作を前提としたアーキテクチャであり、ウィンドウ非表示時もCore Process（Rust側）は正常に稼働し続ける。
3.  **モダンなUI**: フロントエンドにWeb技術（React + TypeScript）を利用できるため、表現力豊かで使いやすいUI/UXを構築しやすい。
4.  **Rust資産の活用**: 監視ロジック（Domain/Usecase層）はRustで記述されているため、Tauriのバックエンドとしてそのまま流用可能。

## 3. アーキテクチャ設計

### 3.1 全体構成
```mermaid
graph TD
    subgraph Frontend [WebView Process / React]
        UI[React UI (Dashboard, Settings)] -->|Invoke| IPC[Tauri IPC Bridge]
        IPC -->|Emit (State Update)| UI
    end

    subgraph Backend [Core Process / Rust]
        IPC_Handler[Commands] -->|Call| Usecase[Usecase Layer (Existing)]
        Usecase -->|Update| State[Shared State]
        
        Tray[System Tray] -->|Event| AppLoop[Tauri Event Loop]
        MonitorThread[Input Monitor] -->|Update| State
        MonitorThread -->|Emit Event| IPC_Handler
    end
```

### 3.2 ディレクトリ構成案
```text
SwitchLifeManager/
├── src-tauri/              <-- バックエンド (旧 src を統合)
│   ├── src/
│   │   ├── main.rs         <-- エントリーポイント (Setup, Tray, Monitor起動)
│   │   ├── commands.rs     <-- フロントエンド向けAPI
│   │   ├── tray.rs         <-- タスクトレイ定義
│   │   ├── domain/         <-- 既存コード流用
│   │   ├── infrastructure/ <-- 既存コード流用
│   │   └── usecase/        <-- 既存コード流用
│   ├── tauri.conf.json
│   └── Cargo.toml
├── src-ui/                 <-- フロントエンド (新規作成)
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   └── ...
│   ├── package.json (Vite + React)
│   └── ...
```

## 4. 移行ステップ

### Phase 1: プロジェクト初期化
1.  `npm create tauri-app` (または `cargo tauri init`) で基本構造を作成。
    *   Frontend: React + TypeScript + Vite
    *   Package Manager: npm または pnpm
2.  既存のRustコード (`domain`, `infrastructure`, `usecase`) を `src-tauri/src` へ移動。
3.  `Cargo.toml` の依存関係（`windows`, `gilrs` 等）を統合。

### Phase 2: バックエンド実装 (Rust)
1.  **タスクトレイ実装**: `SystemTray` を設定し、左クリックでの表示/非表示、右クリックメニュー（Quit）を実装。
2.  **監視スレッド統合**: `main.rs` で `InputMonitor` を別スレッドで起動し、状態を `tauri::State` (Mutex/RwLock) で共有する。
3.  **IPCコマンド実装**: フロントエンドが必要とする機能（キー設定開始、リセット、設定保存）を `commands.rs` に実装。
4.  **イベント発行**: 監視スレッドから定期的に（例: 30fps）最新の状態を `window.emit()` でフロントエンドへ送信する仕組みを作成。

### Phase 3: フロントエンド実装 (React)
1.  **UI構築**: Dashboard（寿命バー）、Settings（キーコンフィグ）の画面を作成。
    *   UIライブラリとして `Mantine` または `shadcn/ui` を採用検討。
2.  **IPC連携**: Rust側からのイベント (`state-update`) を受信し、画面を更新。ボタン操作でRust側のコマンドを呼び出す。

### Phase 4: テスト・リリース
1.  動作確認: タスクトレイ格納・復帰、入力監視のリアルタイム性、設定保存。
2.  ビルド: `cargo tauri build` によるリリースビルド生成。

## 5. 技術スタック詳細
*   **Backend**: Rust (Tauri framework)
*   **Frontend**: React, TypeScript, Vite
*   **UI Framework**: Tailwind CSS + (Mantine / shadcn/ui)
*   **Input**: gilrs (DirectInput), windows-rs (XInput)
*   **State Management**: React Context / Hooks (Frontend), std::sync::Mutex (Backend)

この計画に基づき、順次移行作業を開始する。
