# SwitchLifeManager Tauri移行計画書

**作成日**: 2025年12月30日
**現状ステータス**: 計画策定完了

## 0. 前提（この計画のスコープ固定）

*   **Tauri**: v2 を採用する（以降のAPI表記・設計は v2 前提）
*   **Frontend**: React + TypeScript + Vite
*   **対象OS**: まず Windows を主ターゲット（入力監視は既存のRust実装を流用）
*   **到達ゴール**: Dashboard / Settings に加えて **Input Tester / プロセス連携（自動ポップアップ/レポート）まで**を移行完了の範囲に含める

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
1.  **タスクトレイの公式サポート**: Tauri v2 のトレイAPI（Tray Icon）が標準提供されており、Win32ハックなしで安定した格納・復帰・メニュー操作が可能。
2.  **イベントループの安定性**: バックグラウンド動作を前提としたアーキテクチャであり、ウィンドウ非表示時もCore Process（Rust側）は正常に稼働し続ける。
3.  **モダンなUI**: フロントエンドにWeb技術（React + TypeScript）を利用できるため、表現力豊かで使いやすいUI/UXを構築しやすい。
4.  **Rust資産の活用**: 監視ロジック（Domain/Usecase層）はRustで記述されているため、Tauriのバックエンドとしてそのまま流用可能。

> 注: 本計画は Tauri v2 前提のため、トレイ周りの実装は v1 の `SystemTray` ではなく v2 のトレイAPI（Tray Icon）に合わせる。

## 3. アーキテクチャ設計

### 3.1 全体構成
```mermaid
graph TD
    subgraph Frontend [WebView Process / React]
        UI[React UI (Dashboard, Settings, Input Tester, Report)] -->|Invoke| IPC[Tauri IPC Bridge]
        IPC -->|Emit (State Update)| UI
    end

    subgraph Backend [Core Process / Rust]
        IPC_Handler[Commands] -->|Call| Usecase[Usecase Layer (Existing)]
        Usecase -->|Update| State[Shared State]
        
        Tray[Tray Icon] -->|Event| AppLoop[Tauri Event Loop]
        MonitorThread[Input Monitor] -->|Update| State
        MonitorThread -->|Emit (Event)| IPC_Handler
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

補足:

*   Tauri v2 の雛形・設定ファイル構成は v1 と異なるため、初期化時点で **Tauri v2** を明示して生成する（v2用テンプレートを使用）。
*   既存の `src/` は「Tauriバックエンド側へ取り込むRust資産」と「旧GUI（eframe）資産」に分離して扱い、旧GUIは一時的に退避（後で削除）する。

### Phase 2: バックエンド実装 (Rust)
1.  **トレイ実装（Tauri v2）**:
    *   トレイアイコンの生成。
    *   左クリック（またはダブルクリック）で **表示/非表示トグル**。
    *   右クリックメニューに **Quit** を実装。
    *   「ウィンドウを閉じる（×）」は **終了せずトレイ格納** とする（CloseRequestedを捕捉し、`hide` に置き換える）。
2.  **監視スレッド統合**:
    *   `main.rs`（または `lib.rs` + `main.rs`）で `InputMonitor` を別スレッドで起動。
    *   高頻度監視（接続中 1ms）を前提とし、共有状態は「GUI向けスナップショット」に絞る。
    *   監視スレッド→UI通知はイベント（チャンネル）で扱い、状態は `RwLock` もしくは参照差し替え方式で同期する（どちらかに統一）。
3.  **IPCコマンド実装（例）**: `commands.rs` に以下を実装し、フロントエンドから呼べる形にする。
    *   `get_snapshot`（初期表示・再接続時の全量取得）
    *   `start_learning(logical_key)` / `cancel_learning`（キー学習の開始/キャンセル）
    *   `set_binding(logical_key, physical)`（学習結果の反映、重複は旧キーをUnboundへ）
    *   `reset_to_default_mapping`
    *   `set_target_controller(index)`
    *   `reset_stats(logical_key)` / `bulk_apply(model_id, keys[])`
    *   `set_switch_model(logical_key, model_id)`
4.  **イベント発行**:
    *   UI更新は 30〜60Hz 程度で `emit`（例: `state-update`）を送る。
    *   監視スレッドは1msごとの更新で直接 `emit` せず、別の送信ループ（低頻度）で最新スナップショットを送る構成にする。
    *   プロセス連携イベント（例: `game-started` / `game-exited`）をUIへ発行し、レポート表示トリガにする。

命名規約（表記ゆれ防止）:

* Command名は `snake_case`、Event名は `kebab-case` に統一する。
* ペイロードのキーは `snake_case`。
* 詳細は [docs/architecture.md](docs/architecture.md) の「9.4 IPC設計（Commands / Events）」を正とする。

### Phase 2.5: データ移行（既存ユーザーデータ）
1.  旧版（eframe）保存先と新版（Tauri）の保存先を定義する（Windows: `%LOCALAPPDATA%/SwitchLifeManager/` を基本）。
2.  初回起動時に旧ファイルが存在すれば、新パスへコピー/インポートする。
3.  `schema_version` を確認し、必要ならマイグレーションを実行する（未知の `schema_version` は読み込み中止＋UIへエラー通知）。

### Phase 3: フロントエンド実装 (React)
1.  **UI構築**: Dashboard（寿命バー）、Settings（キーコンフィグ）、Input Tester（入力状態/チャタリング可視化）、Report（ゲーム終了時レポート）の画面を作成。
    *   UIライブラリとして `Mantine` または `shadcn/ui` を採用検討。
2.  **IPC連携**: Rust側からのイベント (`state-update`) を受信し、画面を更新。ボタン操作でRust側のコマンドを呼び出す。

補足:

*   UIは監視スレッドの1ms更新に追従する必要はなく、表示用途は 30〜60Hz のスナップショットで十分とする。
*   「Input Tester」のチャタリング表示は、スナップショット内のフラグ/イベント（直近チャタリング発生）を短時間表示する方式に寄せる。

### Phase 4: テスト・リリース
1.  動作確認:
    *   トレイ格納・復帰・終了（フラッシュ/ゴーストが出ない）
    *   入力監視（接続/切断/再接続、低負荷待機）
    *   Settingsの学習・重複解消
    *   Input Testerの表示とチャタリング検出の視覚フィードバック
    *   プロセス監視（`bm2dx.exe` の起動/終了検知）とレポート表示
    *   Atomic Save（クラッシュ/電源断を想定した破損耐性）
    *   旧データのインポート/マイグレーション
2.  ビルド: `cargo tauri build` によるリリースビルド生成。

## 5. 技術スタック詳細
*   **Backend**: Rust (Tauri framework)
*   **Frontend**: React, TypeScript, Vite
*   **UI Framework**: Tailwind CSS + (Mantine / shadcn/ui)
*   **Input**: gilrs (DirectInput), windows-rs (XInput)
*   **State Management**: React Context / Hooks (Frontend), `RwLock<Snapshot>` or reference-swap + channel (Backend)

この計画に基づき、順次移行作業を開始する。
