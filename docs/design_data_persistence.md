# データ永続化設計 (Requirements & Architecture)

**Status**: Draft
**Target**: SwitchLifeManager (Tauri v2)

## 1. 要件定義 (Requirement Definition)

現在の実装では、すべてのデータ（`UserProfile`）を単一のJSONファイル（`profile.json`）に保存しています。UIの刷新に伴う新要件（詳細な過去セッション履歴、メンテナンスログ）をサポートし、将来的なスケーラビリティを確保するため、設定データと履歴データを分離する必要があります。

### 1.1 主な要件
1.  **スケーラビリティ (Scalability)**:
    *   長期間の使用（例: 10,000回以上のセッション）を経てもパフォーマンスを維持すること。
    *   *現状の課題*: 数千件の詳細なセッション記録を含む巨大なJSONの読み書きは非効率的。
2.  **データ整合性 (Data Integrity)**:
    *   セッションデータやメンテナンスログは堅牢に保存されること。
    *   クラッシュ等によるデータ破損を防ぐため、書き込み操作は原子的（Atomic）であること。
3.  **詳細な履歴記録 (Detailed History)**:
    *   **セッションごとの統計**: 「Past Sessions」タブの詳細ビューをサポートするため、各セッションにおける全キーのプレス数・チャタリング数を記録すること。
    *   **メンテナンスログ**: スイッチの交換やリセットのイベントを恒久的に記録すること。
4.  **ポータビリティ (Portability - Partial)**:
    *   ユーザー設定（ボタンマッピング、OBS設定等）は、バックアップや共有が容易なJSON形式を維持すること。

## 2. アーキテクチャ設計 (Architecture Design)

**ハイブリッドアプローチ**を採用します：設定には **JSON**、履歴データには **SQLite** を使用します。

### 2.1 ストレージ構成

| データ種別 | ストレージ | ファイル名 | 理由 |
| :--- | :--- | :--- | :--- |
| **設定 (Configuration)** | JSON | `config.json` | 手動編集、共有、バージョン管理が容易。サイズが小さい。 |
| **累積統計 (Lifetime Stats)** | JSON | `config.json` | 現在のスイッチ状態と密結合しているため。バックアップの簡便さのためJSONに残す。 |
| **セッション履歴 (History)** | SQLite | `history.db` | データ量が多く、構造化されている。「過去50件を取得」等のクエリ効率が必要。 |
| **メンテナンスログ (Log)** | SQLite | `history.db` | 追記型の構造化ログ。 |

### 2.2 データベーススキーマ (SQLite)

#### Table: `sessions`
各ゲームセッションのメタデータを保存。

```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TEXT NOT NULL, -- ISO8601 UTC
    end_time TEXT NOT NULL,   -- ISO8601 UTC
    duration_secs INTEGER NOT NULL
);
```

#### Table: `session_keys`
セッションごとの各キーの統計情報を保存。

```sql
CREATE TABLE session_keys (
    session_id INTEGER NOT NULL,
    key_name TEXT NOT NULL,       -- "Key1", "E1", etc.
    presses INTEGER NOT NULL,
    chatters INTEGER NOT NULL,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    PRIMARY KEY(session_id, key_name)
);
```

#### Table: `maintenance_events`
スイッチの交換や統計リセットのイベントを保存。

```sql
CREATE TABLE maintenance_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    key_name TEXT NOT NULL,
    event_type TEXT NOT NULL,      -- "REPLACE", "RESET", "EDIT"
    old_model_id TEXT,
    new_model_id TEXT,
    details JSON                   -- リセット直前の統計スナップショットなど
);
```

### 2.3 データアクセス層 (Rust)

*   **ライブラリ**: `sqlx` (SQLite feature, Async)
    *   Tauriの非同期コマンドと相性が良く、マイグレーション機能も統合されているため推奨。
*   **リポジトリパターン**:
    *   `ConfigRepository`: `config.json` の読み書きを担当。
    *   `HistoryRepository`: SQLite への操作（セッション挿入、履歴クエリ）を担当。

### 2.4 アプリケーションロジックの変更点
1.  **起動時 (Startup)**:
    *   `config.json` をメモリ上のステートにロード。
    *   SQLite DBの初期化とマイグレーション実行。
2.  **セッション終了時 (Session End)**:
    *   セッション記録と各キーの統計を SQLite に書き込み (`INSERT`)。
    *   メモリ上の累積統計 (Lifetime Stats) を更新し、`config.json` に保存。
3.  **UI "Past Sessions" (一覧)**:
    *   フロントエンドから `get_recent_sessions(limit: 50)` を呼び出す。
    *   バックエンドは SQLite の `sessions` テーブルをクエリして返す。
4.  **UI "Session Details" (詳細)**:
    *   フロントエンドから `get_session_details(id)` を呼び出す。
    *   バックエンドは `session_keys` テーブルを `WHERE session_id = ?` でクエリして返す。

## 3. 代替案の検討 (Pure JSON)
*   **案**: `recent_sessions` を `history/sessions_YYYY-MM.jsonl` のように月別ファイルに分割する。
*   **メリット**: DB依存なし、可読性が高い。
*   **デメリット**: フィルタリングやソートを手動実装する必要がある。「直近5件」が月を跨ぐ場合の処理が複雑になる。並行書き込みの制御が必要。
*   **結論**: 信頼性の高いローカルデータ管理には SQLite が優れている。

## 4. 実装ステップ
1.  `Cargo.toml` に `sqlx` (sqlite) を追加。
2.  `infrastructure` 内に `Database` モジュールを作成。
3.  マイグレーションロジックの実装（起動時のテーブル作成）。
4.  `SessionManager` を更新し、セッション終了時に DB へ書き込むように変更。
5.  履歴取得用の新しい Tauri Command を実装。
