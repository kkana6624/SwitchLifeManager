# SwitchLifeManager

SwitchLifeManagerは、リズムゲーム（beatmania IIDX等）用コントローラー（PhoenixWANや公式コン等）のマイクロスイッチ寿命、チャタリング（多重反応）、打鍵統計を管理・可視化するためのツールです。

**Tauri v2 + React によるモダンで軽量な設計にリニューアルされました。**

## 主な機能

*   **寿命の可視化**: 累計打鍵数をカウントし、マイクロスイッチの定格寿命（オムロンD2MVシリーズ等）に基づいた残寿命をプログレスバーで表示します。
*   **チャタリング検出**: 設定した閾値に基づき、意図しない多重入力を検出し記録します。
*   **リアルタイム・セッション統計**: **beatmania IIDX INFINITAS** (`bm2dx.exe`) の起動を自動検知し、そのゲームセッション中のみの打鍵数やチャタリング率をリアルタイムに集計します。
*   **テスター**: 実機同様の1-7鍵盤配置（ピアノレイアウト）で直感的に入力確認が可能です。
*   **メンテナンス支援**:
    *   **一括操作**: チェックボックスでキーを複数選択し、モデル変更や統計リセットを一括で行えます。
    *   **個別リセット**: スイッチ交換時に特定のキーの統計のみをリセットできます。
    *   **履歴管理 (History)**: スイッチの交換やリセットの履歴を記録し、いつでも閲覧可能です。
*   **タスクトレイ常駐**: ウィンドウを閉じるとタスクトレイに格納され、バックグラウンドで監視を継続します。**ダブルクリック**でウィンドウを復帰します。
*   **マルチ入力方式対応**: DirectInput (HID / 最大32ボタン) および XInput の両方のコントローラーに対応しています。

## 使用方法

### 1. 起動と接続確認
アプリを実行します。コントローラーが接続されていれば自動的に認識されます。
*   **Status**: 接続中であれば緑色で "Connected" と表示されます。
*   **Game**: INFINITASが起動していれば青色で "In Game" と表示されます。

### 2. キーコンフィグ (Settingsタブ)
**Settings** タブで、コントローラーのボタンをアプリ上の論理キー（1鍵〜7鍵、E1〜E4等）に割り当てます。
1.  対象キーの横にある **Set** をクリック。
2.  コントローラーの対応するボタンを押します。
3.  設定は自動的に保存されます。（重複がある場合は自動的に古い割り当てが解除されます）

### 3. スイッチ管理 (Dashboardタブ)
メイン画面であるDashboardでは以下の操作が可能です。
*   **モデル選択**: 現在使用しているマイクロスイッチのモデル（例: "Omron D2MV-01-1C3 (50g)"）を選択すると、正確な寿命推計が表示されます。
*   **一括変更 (Bulk Actions)**: キー一覧のチェックボックスを選択すると、上部にアクションバーが表示され、モデルの一括変更や統計の一括リセットが可能です。
*   **統計リセット**: スイッチを物理的に交換した際、"Reset" をクリックすることで、そのキーのカウンタのみをリセットして新調した個体として再カウントできます。

### 4. 入力テスト (Testerタブ)
**Input Tester** タブで入力確認が行えます。
*   **ピアノレイアウト**: 1〜7鍵が実機と同様の交互配置で表示されます。
*   **システムボタン**: E1〜E4は対応する列の上部に表示されます。

## インストール・開発方法

### 必要環境
*   **Windows 10/11** (WebView2 ランタイムが必要ですが、通常は標準搭載されています)
*   開発を行う場合:
    *   [Rust](https://www.rust-lang.org/tools/install)
    *   [Node.js](https://nodejs.org/) (npm)

### ソースコードからの実行
1.  リポジトリをクローンします。
2.  フロントエンドの依存関係をインストールします:
    ```bash
    cd src-ui
    npm install
    ```
3.  開発サーバーを起動します:
    ```bash
    # プロジェクトルートで実行
    npm run tauri dev
    # または
    cargo tauri dev
    ```

### リリースビルド
```bash
npm run tauri build
```
実行ファイルは `src-tauri/target/release/` に生成されます。

## データ保存場所
ユーザー設定やログは以下に保存されます:
*   **Windows**: `%LOCALAPPDATA%\SwitchLifeManager\`
    *   `profile.json`: 設定と統計データ。
    *   `app.log`: アプリケーションログ。

## 将来の計画

*   **ダブルプレイ (DP) 対応**: 2台のコントローラーの同時監視（詳細は `docs/roadmap_dp_support.md` を参照）。
*   監視対象プロセスのカスタマイズ（BMSプレイヤー等への対応）。
*   チャタリング発生傾向のグラフ表示。

---

# SwitchLifeManager (English)

SwitchLifeManager is a specialized utility tool designed for rhythm game controllers (specifically Beatmania IIDX controllers like PhoenixWAN or Official Controllers). It helps you visualize microswitch lifespan, detect chattering (double-clicking), and manage maintenance cycles.

**Now powered by Tauri v2 + React for a modern, lightweight, and robust experience.**

## Key Features

*   **Lifespan Visualization**: Tracks total key presses and displays remaining lifespan based on the rated spec of your microswitches (e.g., Omron D2MV series).
*   **Chatter Detection**: Intelligently detects unintended double-clicks (chattering) using configurable thresholds.
*   **Real-time Session Stats**: Automatically detects when **beatmania IIDX INFINITAS** (`bm2dx.exe`) is running and tracks statistics specifically for that game session.
*   **Tester**: A visual input tester featuring a 1-7 piano-style key layout for intuitive checkups.
*   **Maintenance Support**:
    *   **Bulk Actions**: Apply switch models or reset statistics for multiple keys at once using checkboxes.
    *   **Individual Reset**: Reset counters for specific keys after replacement.
    *   **History**: View logs of switch replacements and statistic resets.
*   **System Tray Resident**: Minimizes to the system tray to run quietly in the background. **Double-click** the tray icon to restore the window.
*   **Universal Input Support**: Supports both DirectInput (HID) (up to 32 buttons) and XInput controllers.

## How to Use

### 1. Launch & Connection
Run the application. It will automatically detect your connected controller.
*   **Status**: Shows "Connected" in green.
*   **Game**: Shows "In Game" in blue if INFINITAS is active.

### 2. Key Configuration (Settings Tab)
Go to the **Settings** tab to map your controller's buttons to the application's logical keys (Key 1-7, E1-E4).
1.  Click **Set** next to a key.
2.  Press the corresponding button on your controller.
3.  The mapping is saved automatically.
    *   *Note: If a button is already mapped, the old mapping is automatically removed to prevent conflicts.*

### 3. Switch Management (Dashboard Tab)
The Dashboard is the main view.
*   **Model Selection**: Select the microswitch model you are currently using (e.g., "Omron D2MV-01-1C3 (50g)") to see accurate lifespan estimates.
*   **Bulk Actions**: Select multiple keys using checkboxes. The "Bulk Actions" bar will appear, allowing you to apply a model or reset stats for all selected keys.
*   **Reset Stats**: When you physically replace a switch, click "Reset" to restart the counter for that key.

### 4. Input Tester (Tester Tab)
Use the **Input Tester** to verify button inputs.
*   **Piano Layout**: Keys are arranged in a 1-7 interleaved layout mimicking the actual controller.
*   **System Buttons**: E1-E4 are displayed above the corresponding columns.

## Installation / Development

### Prerequisites
*   **Windows 10/11** (WebView2 Runtime is usually installed by default).
*   For development:
    *   [Rust](https://www.rust-lang.org/tools/install)
    *   [Node.js](https://nodejs.org/) (npm)

### Running from Source
1.  Clone the repository.
2.  Install frontend dependencies:
    ```bash
    cd src-ui
    npm install
    ```
3.  Run the development server:
    ```bash
    # From project root
    npm run tauri dev
    # OR
    cargo tauri dev
    ```

### Building for Release
```bash
npm run tauri build
```
The executable will be located in `src-tauri/target/release/`.

## Data Location
User profiles and logs are saved in:
*   **Windows**: `%LOCALAPPDATA%\SwitchLifeManager\`
    *   `profile.json`: User settings and statistics.
    *   `app.log`: Application logs.

## Future Plans

*   **Double Play (DP) Support**: Simultaneous monitoring of two controllers (see `docs/roadmap_dp_support.md` (Japanese)).
*   Configurable process name for other games (e.g., BMS players).
*   Advanced chatter analysis graphs.