# OBS Integration Design Document

**Status:** Draft
**Target:** SwitchLifeManager (Tauri v2)

## 1. 概要
OBS Studio などの配信ソフトで「現在のセッション統計」を表示するための機能を提供する。
アプリケーション内に軽量な HTTP サーバーを組み込み、OBS の「ブラウザソース」から参照可能な HTML および JSON データを提供する。

**主な要件:**
*   ユーザーが機能の ON/OFF を切り替えられること。
*   使用するポート番号を変更できること。
*   ゲームプレイやメインアプリへの負荷を最小限に抑えること。

## 2. アーキテクチャ

### 2.1 全体構成
Tauri アプリケーション (Backend process) のバックグラウンドスレッドとして HTTP サーバーを起動する。

```mermaid
graph TD
    subgraph Core [Rust Backend]
        Main[Main Thread]
        Monitor[Input Monitor Thread]
        SharedState["Shared State (ArcSwap)"]
        
        subgraph Server ["OBS Integration Module"]
            HttpServer[Axum HTTP Server]
        end
    end
    
    subgraph External [External Consumers]
        OBS["OBS Studio (Browser Source)"]
        Browser[Web Browser]
    end

    Monitor -->|Write| SharedState
    HttpServer -->|Read| SharedState
    Main -->|Start/Stop| HttpServer
    
    OBS -- "GET / (HTML)" --> HttpServer
    OBS -- "Poll /api/stats (JSON)" --> HttpServer
    Browser -- "GET ..." --> HttpServer

```

### 2.2 コンポーネント設計

#### A. HTTP Server (Backend)
*   **Framework**: `axum` (軽量・高速・非同期)
*   **Binding**: `127.0.0.1` (ローカルホストのみ)
*   **Port**: デフォルト `36000` (設定により変更可能)
*   **Lifecycle**:
    *   アプリ起動時: 設定 (`obs_enabled`) に基づき自動起動。
    *   設定変更時: ON/OFF 切り替えやポート変更時に再起動。

#### B. Endpoints

| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/` | オーバーレイ表示用の HTML ファイルを返す。 |
| `GET` | `/api/stats` | 現在のセッション統計とメタデータ（推奨ポーリング間隔など）を含む JSON データを返す。 |
| `GET` | `/overlay.css` | (Optional) スタイルシート。HTMLにインライン化しても良い。 |
| `GET` | `/overlay.js` | (Optional) フロントエンドロジック。HTMLにインライン化しても良い。 |

#### C. Overlay Frontend (HTML/TypeScript)
*   **Technology**: TypeScript (Vanilla) + Vite
*   **Reason**: 配列や型定義(`SharedState`の型など)をメインアプリと共有し、開発効率と保守性を高めるため。Reactは使用せず、DOM操作を直接行うことで軽量さを維持する。
*   **Behavior**:
    1.  ロード時に静的な HTML 構造を描画。
    2.  初回ロード時、およびレスポンスヘッダやJSON内の指定 (`poll_interval_ms`) に基づく間隔で `/api/stats` をポーリング。
    3.  **Visual**: 単なる数値リストではなく、キーごとの押下回数を積み上げ棒グラフ（Bar Chart）等で視覚的に表現する。
        *   各バーはセッション内の最大値、あるいは特定スケールに対して相対的に伸縮するアニメーションを行う。

### 2.3 データフロー
1.  **Monitor Thread** が入力を検知し、`SharedState` を更新する (1ms 周期)。
2.  **OBS Browser Source** が `/api/stats` をリクエストする (例: 1000ms 周期)。
3.  **HTTP Server** が `SharedState` の最新スナップショットを取得し、JSON レスポンスを生成する。
    *   **Locking Strategy**: `ArcSwap` の `load()` を使用するため、Monitor Thread の書き込みをブロックしない (Wait-free reader)。

## 3. 設定項目 (AppConfig)

`AppConfig` 構造体に以下のフィールドを追加する。

```rust
struct AppConfig {
    // ...existing fields
    
    // OBS Integration
    pub obs_enabled: bool,          // default: false
    pub obs_port: u16,              // default: 36000
    pub obs_poll_interval_ms: u64,  // default: 1000, range: 100-5000
}
```

## 4. UI/UX (SwitchLifeManager)

### Settings 画面
「OBS Integration」セクションを追加。

*   **Switch**: Enable OBS Server (ON/OFF)
*   **Number Input**: Port (36000)
    *   ON の状態でも変更可能だが、変更適用時にサーバー再起動が発生する、もしくは「Apply」ボタンで反映する。
*   **Slider / Input**: Refresh Rate (Poll Interval)
    *   Range: 100ms (High) - 5000ms (Low).
*   **Status Indicator**: Running (Green) / Stopped (Gray) / Error (Red)
*   **Source URL Info**: `http://localhost:36000` (クリックでコピー)

## 5. セキュリティとパフォーマンス

*   **セキュリティ**:
    *   `127.0.0.1` にバインドすることで、外部ネットワークからのアクセスを遮断する。
    *   HTTPS は使用しない (OBS の内部ブラウザとの互換性・証明書管理の手間を回避)。
*   **パフォーマンス**:
    *   `/api/stats` は単なるメモリアクセスと JSON シリアライズのみであり、計算コストは無視できるレベル。
    *   ポーリング頻度はクライアント (OBS) 依存だが、1秒間隔程度を想定。

## 6. 実装フェーズ

1.  **Backend**: `axum` 依存関係の追加、`ObsServer` 構造体の実装。
2.  **Backend**: `SharedState` へのアクセスと JSON レスポンスの実装。
3.  **Overlay**: 静的 HTML/CSS/JS の作成と埋め込み (Rust バイナリに `include_str!` 等で埋め込むか、アセットとして配置)。
4.  **Frontend**: 設定画面の UI 実装と IPC 連携。
