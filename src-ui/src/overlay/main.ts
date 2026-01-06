import './style.css';

// Define types matching the backend JSON response
interface AppConfig {
    obs_poll_interval_ms: number;
}

interface ButtonStats {
    last_session_presses: number;
}

interface SwitchData {
    switch_model_id: string;
    stats: ButtonStats;
}

interface MonitorSharedState {
    config: AppConfig;
    switches: Record<string, SwitchData>;
    is_game_running: boolean;
    profile_name: string;
}

const API_URL = '/api/stats';

class OverlayApp {
    private container: HTMLElement;
    private pollInterval: number = 1000;
    // Dynamic scale: Max presses seen in this session (starts at small value to avoid 0 div)
    private maxPressesInSession: number = 10;

    constructor() {
        this.container = document.getElementById('app')!;
        this.initUI();
        this.poll();
    }

    initUI() {
        this.container.innerHTML = `
      <div class="title">Session Stats</div>
      <div class="stats-container" id="stats-list">
        <!-- Bars will be injected here -->
      </div>
    `;
    }

    async poll() {
        try {
            // In dev, we might need full URL if served from Vite but API is on port 36000
            // But for production use case, this script is served from localhost:36000/
            const res = await fetch(API_URL);
            if (res.ok) {
                const data: MonitorSharedState = await res.json();
                this.updateUI(data);

                // Update interval if changed
                if (data.config && data.config.obs_poll_interval_ms !== this.pollInterval) {
                    this.pollInterval = data.config.obs_poll_interval_ms;
                }
            }
        } catch (e) {
            console.error("Polling failed", e);
        } finally {
            window.setTimeout(() => this.poll(), this.pollInterval);
        }
    }

    updateUI(data: MonitorSharedState) {
        const list = document.getElementById('stats-list');
        if (!list) return;

        // Track max for scaling (using last_session_presses)
        // Only update max if game is running or we have data. 
        // If new session (reset), maxPresses might decrease? 
        // "Bar Chart" usually has fixed scale or dynamic. Dynamic is better for "Session".
        // Let's calculate max from current data.
        let currentMax = 10;
        const entries = Object.entries(data.switches).filter(([key]) => !key.startsWith("Other")); // Filter relevant keys

        // Sort keys: Key1..7, E1..4
        // Simple sort by string for now
        entries.sort((a, b) => a[0].localeCompare(b[0], undefined, { numeric: true }));

        for (const [_, sw] of entries) {
            if (sw.stats.last_session_presses > currentMax) {
                currentMax = sw.stats.last_session_presses;
            }
        }
        this.maxPressesInSession = currentMax;

        // Build or Update rows
        // To match DOM elements without React, we can use ID based lookups
        entries.forEach(([key, sw]) => {
            let row = document.getElementById(`row-${key}`);
            const count = sw.stats.last_session_presses;
            const widthPercent = (count / this.maxPressesInSession) * 100;

            if (!row) {
                // Create
                row = document.createElement('div');
                row.className = 'key-row';
                row.id = `row-${key}`;
                row.innerHTML = `
                <div class="key-label">${key}</div>
                <div class="bar-container">
                    <div class="bar-fill" style="width: 0%"></div>
                    <div class="count-label">0</div>
                </div>
            `;
                list.appendChild(row);
            }

            // Update
            const fill = row.querySelector('.bar-fill') as HTMLElement;
            const label = row.querySelector('.count-label') as HTMLElement;

            if (fill) fill.style.width = `${widthPercent}%`;
            if (label) label.textContent = count.toString();
        });
    }
}

new OverlayApp();
