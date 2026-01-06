use crate::domain::models::{ButtonStats, LogicalKey};
use crate::usecase::input_monitor::ChatterDetector;

#[test]
fn test_input_outside_session_updates_total_only() {
    let mut detector = ChatterDetector::new(15);
    let mut stats = ButtonStats::default();
    let key = LogicalKey::Key1;

    // Simulate input while session is INACTIVE (is_session_active = false)
    detector.process_button(&key, true, 100, &mut stats, false);

    // Total should increase
    assert_eq!(stats.total_presses, 1);
    // Session stats should NOT increase
    assert_eq!(stats.last_session_presses, 0);

    // Release (also outside session)
    detector.process_button(&key, false, 150, &mut stats, false);
    assert_eq!(stats.total_releases, 1);
    // Note: total_releases currently does not have a session counterpart in ButtonStats struct?
    // Wait, let's check ButtonStats definition in models.rs.
    // It has: last_session_presses, last_session_chatters, last_session_chatter_releases.
    // It does NOT seem to have last_session_releases. If not required by UI, it's fine.
}

#[test]
fn test_input_inside_session_updates_both() {
    let mut detector = ChatterDetector::new(15);
    let mut stats = ButtonStats::default();
    let key = LogicalKey::Key1;

    // Simulate input while session is ACTIVE (is_session_active = true)
    detector.process_button(&key, true, 100, &mut stats, true);

    // Both should increase
    assert_eq!(stats.total_presses, 1);
    assert_eq!(stats.last_session_presses, 1);
}

#[test]
fn test_chatter_inside_and_outside_session() {
    let mut detector = ChatterDetector::new(15);
    let mut stats = ButtonStats::default();
    let key = LogicalKey::Key1;

    // 1. Chatter OUTSIDE session
    // Press -> Release -> Chatter Press
    detector.process_button(&key, true, 100, &mut stats, false);
    detector.process_button(&key, false, 150, &mut stats, false);
    detector.process_button(&key, true, 155, &mut stats, false); // Chatter

    assert_eq!(stats.total_chatters, 1);
    assert_eq!(stats.last_session_chatters, 0);

    // Cooldown...
    detector.process_button(&key, false, 200, &mut stats, false);

    // 2. Chatter INSIDE session
    detector.process_button(&key, true, 300, &mut stats, true);
    detector.process_button(&key, false, 350, &mut stats, true);
    detector.process_button(&key, true, 355, &mut stats, true); // Chatter

    assert_eq!(stats.total_chatters, 2);
    assert_eq!(stats.last_session_chatters, 1);
}
