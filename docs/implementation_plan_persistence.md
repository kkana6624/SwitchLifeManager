# 実装計画: データ永続化 (SQLite)

**Status**: Draft
**Target**: SwitchLifeManager (Tauri v2)

## 1. 概要 (Goal)
セッション履歴とメンテナンスログの保存に SQLite を導入し、データのスケーラビリティと詳細な統計記録を実現します。

## 2. 変更内容 (Proposed Changes)

### 2.1 Backend (Rust)

#### `src-tauri/Cargo.toml`
*   **依存関係の追加**:
    *   `sqlx`: バージョン `0.7`
        *   Features: `runtime-tokio-native-tls`, `sqlite`

#### `src-tauri/src/infrastructure/database.rs` (新規作成)
*   **目的**: データベース接続とクエリの管理。
*   **実装内容**:
    *   `Database` 構造体: `sqlx::SqlitePool` をラップ。
    *   `init()` メソッド:
        *   データベースファイル (`history.db`) への接続。
        *   テーブル作成 (`sessions`, `session_keys`, `maintenance_events`) のマイグレーション実行。
    *   `insert_session(session: &SessionRecord, key_stats: &HashMap<LogicalKey, ButtonStats>)`:
        *   トランザクションを使用して `sessions` と `session_keys` に一括挿入。
    *   `get_recent_sessions(limit: u32, offset: u32)`:
        *   `sessions` テーブルから降順で取得。
    *   `get_session_details(session_id: i64)`:
        *   指定されたIDに関連する `session_keys` を取得。
    *   `insert_maintenance_event(event: &SwitchHistoryEntry)`:
        *   `maintenance_events` テーブルにログを挿入。

#### `src-tauri/src/application/session_manager.rs`
*   **変更点**: `stop_session` メソッド（または同等の終了ロジック）。
    *   従来: メモリ上の `AppState.recent_sessions` ベクタに追加。
    *   **変更後**: `Database::insert_session` を呼び出し、SQLiteへ永続化。

#### `src-tauri/src/commands.rs`
*   **新規コマンド追加**:
    *   `get_history_sessions(limit, offset)` -> `Vec<SessionRecord>`
        *   セッション一覧表示用（軽量データ）。
    *   `get_session_details(session_id)` -> `SessionDetails`
        *   セッション選択時の詳細表示用（全キーの統計）。
    *   `get_maintenance_log(limit)` -> `Vec<MaintenanceEntry>`
        *   メンテナンスログ表示用。

### 2.2 Frontend (React)

#### `src-ui/src/features/sessions/SessionHistory.tsx`
*   **変更点**: データ取得ロジックの刷新。
    *   `MonitorSharedState` (メモリ) への依存を廃止（または直近1件のみに限定）。
    *   **マウント時**: `invoke('get_history_sessions')` を呼び出し、リストを表示。
    *   **セッション選択時**: `invoke('get_session_details', { id })` を呼び出し、右ペインに詳細を表示。

## 3. 検証計画 (Verification Plan)

### 3.1 自動テスト (Automated Tests)
*   **Unit Tests (`database.rs`)**:
    *   インメモリデータベース (`:memory:`) を使用してテスト。
    *   セッションの挿入と読み出しが正確に行えるか検証。
    *   外部キー制約（セッション削除時の統計データ削除など）の動作確認。

### 3.2 手動検証 (Manual Verification)
1.  **起動確認**: アプリ起動時に `history.db` ファイルが作成されること。
2.  **データ保存**: ゲームプレイ終了後、DBブラウザ等で `history.db` を開き、新しい行が追加されていることを確認。
3.  **UI表示**:
    *   「Past Sessions」タブが正しくDBからデータを読み込み、リスト表示すること。
    *   リストをクリックした際、詳細情報（各キーのプレス数）が正しく表示されること。
