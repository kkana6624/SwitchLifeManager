use crate::domain::models::{ButtonStats, LogicalKey};
use std::collections::HashMap;

#[derive(Debug, Default)]
struct ButtonMonitorState {
    last_release_at: Option<u64>,
    chatter_cooldown_until: u64,
    is_pressed: bool,
    // Tracks if we have already counted a "chatter release" for the current release cycle to avoid double counting
    // Actually the requirement says: "If this is the first chatter after release... total_chatter_releases += 1"
    // So we need to track if we've already attributed a chatter-group to the last release.
    has_counted_chatter_release: bool,
}

pub struct ChatterDetector {
    // Map from LogicalKey to its monitoring state
    states: HashMap<LogicalKey, ButtonMonitorState>,
    chatter_threshold_ms: u64,
}

impl ChatterDetector {
    pub fn new(chatter_threshold_ms: u64) -> Self {
        Self {
            states: HashMap::new(),
            chatter_threshold_ms,
        }
    }

    /// Process a new state for a specific button.
    ///
    /// # Arguments
    /// * `key` - The logical key being updated
    /// * `is_pressed_now` - The current physical state of the button
    /// * `now_ms` - Current timestamp in milliseconds
    /// * `stats` - The stats object to update
    /// * `is_session_active` - Whether the game session is currently active (game running)
    pub fn process_button(
        &mut self,
        key: &LogicalKey,
        is_pressed_now: bool,
        now_ms: u64,
        stats: &mut ButtonStats,
        is_session_active: bool,
    ) {
        let state = self
            .states
            .entry(key.clone())
            .or_insert_with(ButtonMonitorState::default);

        // Edge detection
        if is_pressed_now && !state.is_pressed {
            // Rising Edge (Press)

            // Check cooldown (multi-bounce suppression)
            if now_ms < state.chatter_cooldown_until {
                // Ignored (debouncing/cooldown active)
                // Do not update state.is_pressed?
                // Architecture says: "now < chatter_cooldown_until の間は、押下・チャタリングのどちらも数えない"
                // But we must update is_pressed to true so we detect release later.
                state.is_pressed = true;
                return;
            }

            let is_chatter = if let Some(last_release) = state.last_release_at {
                now_ms - last_release < self.chatter_threshold_ms
            } else {
                false
            };

            if is_chatter {
                stats.total_chatters += 1;
                if is_session_active {
                    stats.last_session_chatters += 1;
                }

                if !state.has_counted_chatter_release {
                    stats.total_chatter_releases += 1;
                    if is_session_active {
                        stats.last_session_chatter_releases += 1;
                    }
                    state.has_counted_chatter_release = true;
                }

                // Set cooldown
                state.chatter_cooldown_until = now_ms + self.chatter_threshold_ms;
                // DO NOT increment total_presses
            } else {
                // Normal Press
                stats.total_presses += 1;
                if is_session_active {
                    stats.last_session_presses += 1;
                }
            }

            state.is_pressed = true;
        } else if !is_pressed_now && state.is_pressed {
            // Falling Edge (Release)
            stats.total_releases += 1;
            state.last_release_at = Some(now_ms);
            state.has_counted_chatter_release = false; // Reset for next cycle
            state.is_pressed = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_press_release() {
        let mut detector = ChatterDetector::new(15);
        let mut stats = ButtonStats::default();
        let key = LogicalKey::Key1;

        // Press at 100ms
        detector.process_button(&key, true, 100, &mut stats, true);
        assert_eq!(stats.total_presses, 1);
        assert_eq!(stats.total_releases, 0);
        assert_eq!(stats.total_chatters, 0);

        // Release at 150ms
        detector.process_button(&key, false, 150, &mut stats, true);
        assert_eq!(stats.total_presses, 1);
        assert_eq!(stats.total_releases, 1);
        assert_eq!(stats.total_chatters, 0);
    }

    #[test]
    fn test_chattering_press() {
        let mut detector = ChatterDetector::new(15);
        let mut stats = ButtonStats::default();
        let key = LogicalKey::Key1;

        // 1. Normal Press at 100ms
        detector.process_button(&key, true, 100, &mut stats, true);
        // 2. Release at 150ms
        detector.process_button(&key, false, 150, &mut stats, true);

        // 3. Chatter Press at 155ms (5ms after release < 15ms)
        detector.process_button(&key, true, 155, &mut stats, true);

        assert_eq!(stats.total_presses, 1); // Should not increase
        assert_eq!(stats.total_releases, 1);
        assert_eq!(stats.total_chatters, 1);
        assert_eq!(stats.total_chatter_releases, 1);

        // 4. Release at 160ms
        detector.process_button(&key, false, 160, &mut stats, true);
        assert_eq!(stats.total_releases, 2);
    }

    #[test]
    fn test_multi_bounce_suppression() {
        // Architecture: "now < chatter_cooldown_until の間は、押下・チャタリングのどちらも数えない"
        let mut detector = ChatterDetector::new(15);
        let mut stats = ButtonStats::default();
        let key = LogicalKey::Key1;

        // 1. Normal Press
        detector.process_button(&key, true, 100, &mut stats, true);
        detector.process_button(&key, false, 150, &mut stats, true); // Release at 150

        // 2. Chatter Press at 155 (set cooldown until 155+15 = 170)
        detector.process_button(&key, true, 155, &mut stats, true);
        assert_eq!(stats.total_chatters, 1);

        // 3. Quick Release at 156
        detector.process_button(&key, false, 156, &mut stats, true);

        // 4. Another Chatter Press at 158 (still < 170) -> Should be ignored
        detector.process_button(&key, true, 158, &mut stats, true);
        assert_eq!(
            stats.total_chatters, 1,
            "Should ignore press during cooldown"
        );

        // 5. Release at 160
        detector.process_button(&key, false, 160, &mut stats, true);

        // 6. Press at 180 ( > 170) -> Should be treated as new event (but check if chatter or normal)
        // Last release was at 160. 180 - 160 = 20 > 15. So Normal Press.
        detector.process_button(&key, true, 180, &mut stats, true);
        assert_eq!(stats.total_presses, 2);
        assert_eq!(stats.total_chatters, 1);
    }

    #[test]
    fn test_multiple_chatters_single_release_group() {
        // If we have multiple chatters that ARE counted (i.e., not suppressed by cooldown? wait, cooldown suppresses them).
        // Wait, the logic says: "chatter_cooldown_until = now + chatter_threshold_ms".
        // So effectively, once a chatter is detected, we ignore everything for `threshold` ms.
        // So we can't really have "multiple chatters" for the SAME release event unless the bouncing continues for longer than threshold?

        // Scenario:
        // Release at 100.
        // Press at 105 (Chatter 1). Cooldown -> 120.
        // Release at 106.
        // Press at 125. (125 - 106 = 19 > 15). This is Normal Press.

        // What if threshold is small? Say 10ms.
        // Release at 100.
        // Press at 102 (Chatter). Cooldown -> 112.
        // Release at 103.
        // Press at 105 (Ignored due to cooldown 112).
        // ...

        // It seems the "total_chatter_releases" logic handles the case where maybe the logic allowed multiple chatters.
        // But with the cooldown logic, we mostly just catch the first one.
        // However, if the cooldown expires and we are still bouncing?

        // Release at 100. Threshold 10.
        // Press at 102 (Chatter). Cooldown -> 112.
        // Release at 103.
        // ... time passes ...
        // Press at 115. (115 - 103 = 12 > 10). Normal Press?
        // Or if 115 - 100 (original release)? No, last release was 103.

        // So "total_chatter_releases" tracks "how many times did a release event result in subsequent chattering".
        // Since we count the first chatter, and set flag, subsequent chatters (if any logic permitted them) wouldn't increment it.
        // But with cooldown, subsequent chatters are likely ignored or treated as new presses if slow enough.

        let mut detector = ChatterDetector::new(10);
        let mut stats = ButtonStats::default();
        let key = LogicalKey::Key1;

        detector.process_button(&key, true, 100, &mut stats, true);
        detector.process_button(&key, false, 150, &mut stats, true); // Release

        detector.process_button(&key, true, 152, &mut stats, true); // Chatter 1
        assert_eq!(stats.total_chatter_releases, 1);
        assert_eq!(stats.total_chatters, 1);

        // Cooldown is 162.
        detector.process_button(&key, false, 153, &mut stats, true); // Release

        // Press at 155. Inside cooldown (155 < 162). Ignored.
        detector.process_button(&key, true, 155, &mut stats, true);
        assert_eq!(stats.total_chatters, 1);

        // Release at 156.
        detector.process_button(&key, false, 156, &mut stats, true);
    }
}
