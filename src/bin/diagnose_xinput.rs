use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting XInput Diagnostic Tool...");
    println!("Checking all 4 XInput slots (User Index 0-3)...");
    println!("Press Ctrl+C to exit.");

    loop {
        let mut connected_count = 0;
        println!("\n--- Scan at {:?} ---", std::time::SystemTime::now());
        
        for i in 0..4 {
            let mut state = XINPUT_STATE::default();
            let result = unsafe { XInputGetState(i, &mut state) };

            if result == 0 { // ERROR_SUCCESS
                connected_count += 1;
                let buttons = state.Gamepad.wButtons.0;
                println!("  [Slot {}] CONNECTED - Buttons: {:016b} (Decimal: {})", i, buttons, buttons);
            } else if result == 1167 { // ERROR_DEVICE_NOT_CONNECTED
                // println!("  [Slot {}] Not Connected", i);
            } else {
                println!("  [Slot {}] Error Code: {}", i, result);
            }
        }

        if connected_count == 0 {
            println!("  No XInput devices found.");
        }

        thread::sleep(Duration::from_secs(1));
    }
}
