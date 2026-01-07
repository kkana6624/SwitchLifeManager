# 実装計画: データ永続化 (SQLite導入)

**Status**: Verified Plan
**Target**: SwitchLifeManager (Tauri v2)
**Approach**: TDD + Clean Architecture

## 1. 目的 (Goal)
詳細なセッション履歴（各キーごとの統計情報）とメンテナンスログを永続化するため、SQLiteを導入します。
Clean Architectureに基づき、ドメインロジックをデータベースの実装詳細から分離し、自動テスト可能な設計とします。

## 2. アーキテクチャ設計 (Architecture)

### 2.1 レイヤ構造 (Layer Structure)

```
src-tauri/src/
├── domain/                  # 外部依存なし
│   ├── entities/            # Session, ButtonStats, etc.
│   └── repositories/        # SessionRepository (Trait)
├── infrastructure/          # データベース、ファイルシステム実装
│   └── repository_impl/     # SqliteSessionRepository
├── application/ (or usecase)# ユースケース
│   ├── save_session.rs
│   └── get_history.rs
└── interface/ (tauri)       # コントローラ
    └── commands/            # Tauri Commands
```

### 2.2 ドメイン層 (Domain Layer)

`src-tauri/src/domain/repositories.rs`:
```rust
use async_trait::async_trait;
use crate::domain::entities::{SessionRecord, SessionKeyStats};

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &SessionRecord, stats: &[SessionKeyStats]) -> anyhow::Result<i64>;
    async fn get_recent(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<SessionRecord>>;
    async fn get_details(&self, session_id: i64) -> anyhow::Result<Vec<SessionKeyStats>>;
}
```

### 2.3 データベーススキーマ (Schema)

`migrations/YYYYMMDDHHMMSS_init.sql`:

```sql
-- セッション情報の親テーブル
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TEXT NOT NULL, -- ISO8601
    end_time TEXT NOT NULL,   -- ISO8601
    duration_secs INTEGER NOT NULL
);

-- セッションごとのキー統計（詳細データ）
CREATE TABLE IF NOT EXISTS session_keys (
    session_id INTEGER NOT NULL,
    key_name TEXT NOT NULL,
    presses INTEGER NOT NULL,
    chatters INTEGER NOT NULL,
    chatter_releases INTEGER NOT NULL,
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    PRIMARY KEY(session_id, key_name)
);

-- メンテナンスログ（今回は設計のみで後回し可、あるいは同時に実装）
CREATE TABLE IF NOT EXISTS maintenance_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    key_name TEXT NOT NULL,
    event_type TEXT NOT NULL,
    details TEXT -- JSON string
);
```

## 3. 実装ステップ (TDD Workflow)

### Step 1: 環境構築と依存関係の追加
`Cargo.toml` に `sqlx` を追加します。
**Features**: `runtime-tokio`, `tls-native-tls`, `sqlite`

### Step 2: ドメイン層の定義
*   `SessionRepository` トレイトを定義。
*   必要なエンティティ (`SessionKeyStats` など) を定義（既存モデルの調整含む）。

### Step 3: インフラ層 (Repository) の実装 (TDD)
インメモリSQLite (`sqlite::memory:`) を使用した結合テストを作成し、リポジトリの実装を駆動します。

*   **Test 1**: `save` メソッドがエラーなく完了し、IDを返すこと。
*   **Test 2**: 保存したセッションを `get_recent` で取得できること。
*   **Test 3**: `get_details` で保存したキー統計が取得できること。

### Step 4: アプリケーション層 (AppState) への統合
*   `AppState` に `Arc<dyn SessionRepository>` (または具象型 `SqliteRepository`) を持たせる。
*   アプリ起動時 (`setup`) にDB接続とマイグレーション実行。

### Step 5: ユースケースの実装 (コマンド)
*   `stop_session` ロジック内で、リポジトリの `save` を呼び出す。
*   `get_history_sessions` コマンドの実装。
*   `get_session_details` コマンドの実装。

## 4. 検証計画 (Verification)

### 自動テスト
*   `cargo test` により、インメモリDBを用いたリポジトリのCRUD動作が保証されていること。

### 手動検証
1.  アプリを起動し、正常に立ち上がること (DBファイル生成確認)。
2.  ゲーム（またはメモ帳など監視対象）を起動・終了し、1セッションを完了させる。
3.  「Past Sessions」タブを開き、新しいセッションが表示されること。
4.  そのセッションをクリックし、右ペインにキーごとの統計が表示されること。
