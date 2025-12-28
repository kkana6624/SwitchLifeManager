# SwitchLifeManager

SwitchLifeManagerは、リズムゲーム（beatmania IIDX等）用コントローラー（PhoenixWANや公式コン等）のマイクロスイッチ寿命、チャタリング（多重反応）、打鍵統計を管理・可視化するためのツールです。

## 主な機能

*   **寿命の可視化**: 累計打鍵数をカウントし、マイクロスイッチの定格寿命（オムロンD2MVシリーズ等）に基づいた残寿命をプログレスバーで表示します。
*   **チャタリング検出**: 設定した閾値に基づき、意図しない多重入力を検出し、統計として記録します。
*   **リアルタイム・セッション統計**: **beatmania IIDX INFINITAS** (`bm2dx.exe`) の起動を自動検知し、そのゲームセッション中のみの打鍵数やチャタリング率をリアルタイムに集計します。
*   **メンテナンス支援**: スイッチ交換時に特定のキーの統計のみをリセットできます。
*   **マルチ入力方式対応**: DirectInput (HID) および XInput の両方のコントローラーに対応しています。

## 使用方法

### 1. 起動と接続確認
`SwitchLifeManager.exe` を実行します。コントローラーが接続されていれば自動的に認識されます。
*   **Status**: 接続中であれば緑色で "Connected" と表示されます。
*   **Game**: INFINITASが起動していれば青色で "Running" と表示されます。

### 2. キーコンフィグ (Settingsタブ)
**Settings** タブで、コントローラーのボタンをアプリ上の論理キー（1鍵〜7鍵、E1〜E4等）に割り当てます。
1.  対象キーの横にある **Set** をクリック。
2.  コントローラーの対応するボタンを押します。
3.  設定は自動的に保存されます。

### 3. スイッチ管理 (Dashboardタブ)
メイン画面であるDashboardでは以下の操作が可能です。
*   **モデル選択**: 現在使用しているマイクロスイッチのモデル（例: "Omron D2MV-01-1C3 (50g)"）を選択すると、正確な寿命推計が表示されます。
*   **一括変更**: 上部のパネルから、選択した複数のキーに一括でスイッチモデルを適用できます。
*   **統計リセット**: スイッチを物理的に交換した際、"Reset Stats" をクリックすることで、そのキーのカウンタのみをリセットして新調した個体として再カウントできます。

### 4. セッション統計 (Session Statsタブ)
現在の（または直近の）プレイセッションの詳細データを確認できます。
*   **リアルタイム更新**: プレイ中の打鍵数やチャタリング率がリアルタイムで更新されます。
*   **自動リセット**: このタブの統計は、ゲーム (`bm2dx.exe`) を起動するたびに自動的にリセットされます。

## 現在の制限事項・注意点

*   **バックグラウンド動作**: 現在、本アプリは **ウィンドウを開いたまま** にする必要があります。タスクトレイへの格納（常駐化）には **まだ対応していません**。ウィンドウを閉じるとアプリも終了します。この機能は将来のアップデートで対応予定です。
*   **プロセス検知**: セッション統計は `bm2dx.exe` というプロセス名を監視して動作します。

## インストール方法

1.  最新のリリースからZIPファイルをダウンロードします。
2.  任意のフォルダに展開します。
3.  `SwitchLifeManager.exe` を実行します。
    *   *補足: 設定ファイルはローカルのアプリケーションデータフォルダ (`%LOCALAPPDATA%\SwitchLifeManager`) に保存されます。*

## 将来の計画

*   タスクトレイ常駐機能（バックグラウンド動作）。
*   監視対象プロセスのカスタマイズ（BMSプレイヤー等への対応）。
*   チャタリング発生傾向のグラフ表示。

---

# SwitchLifeManager (English)

SwitchLifeManager is a specialized utility tool designed for rhythm game controllers (specifically Beatmania IIDX controllers like PhoenixWAN or Official Controllers). It helps you visualize microswitch lifespan, detect chattering (double-clicking), and manage maintenance cycles.

## Key Features

*   **Lifespan Visualization**: Tracks total key presses and displays remaining lifespan based on the rated spec of your microswitches (e.g., Omron D2MV series).
*   **Chatter Detection**: Intelligently detects unintended double-clicks (chattering) using configurable thresholds.
*   **Real-time Session Stats**: Automatically detects when **beatmania IIDX INFINITAS** (`bm2dx.exe`) is running and tracks statistics specifically for that game session.
*   **Maintenance Support**: Allows you to reset statistics for specific keys when you replace switches.
*   **Universal Input Support**: Supports both DirectInput (HID) and XInput controllers.

## How to Use

### 1. Launch & Connection
Simply run `SwitchLifeManager.exe`. The application will automatically detect your connected controller.
*   **Status**: Should show "Connected" in green.
*   **Game**: Shows "Running" in blue if INFINITAS is active.

### 2. Key Configuration (Settings Tab)
Go to the **Settings** tab to map your controller's buttons to the application's logical keys (Key 1-7, E1-E4).
1.  Click **Set** next to a key.
2.  Press the corresponding button on your controller.
3.  The mapping is saved automatically.

### 3. Switch Management (Dashboard Tab)
The Dashboard is the main view.
*   **Model Selection**: Select the microswitch model you are currently using (e.g., "Omron D2MV-01-1C3 (50g)") to see accurate lifespan estimates.
*   **Bulk Actions**: Use the top panel to apply a switch model to multiple keys at once.
*   **Reset Stats**: When you physically replace a switch, click "Reset Stats" for that key to restart the counter.

### 4. Session Statistics (Session Stats Tab)
This tab shows performance data for your current (or last) gaming session.
*   **Real-time**: Watch press counts and chatter rates update as you play.
*   **Reset**: Statistics in this tab are automatically reset when you launch the game (`bm2dx.exe`).

## Current Limitations & Notes

*   **Background Operation**: Currently, the application **must remain open** to track inputs. Minimizing to the system tray (background residency) is **NOT yet supported**. Closing the window will terminate the application. This feature is planned for a future release.
*   **Process Detection**: Session tracking relies on detecting the process name `bm2dx.exe`.

## Installation

1.  Download the latest release ZIP file.
2.  Extract the contents to a folder of your choice.
3.  Run `SwitchLifeManager.exe`.
    *   *Note: Settings are saved in your local application data folder (`%LOCALAPPDATA%\SwitchLifeManager`).*

## Future Plans

*   System Tray support (minimize to background).
*   Configurable process name for other games (e.g., BMS players).
*   Advanced chatter analysis graphs.