use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use gilrs::{Gilrs, GamepadId};

/// DirectInput/HID input source using the `gilrs` crate.
pub struct GilrsInputSource {
    gilrs: Gilrs,
}

impl GilrsInputSource {
    pub fn new() -> Self {
        Self {
            gilrs: Gilrs::new().unwrap(), // In a real app, handle error
        }
    }

    /// Helper to map a stable index (user setting) to a runtime GamepadId
    fn get_gamepad_by_index(&mut self, index: u32) -> Option<GamepadId> {
        // Simple mapping: Iterate all connected gamepads and pick the nth one.
        // This is not stable across reboots if IDs change, but standard for basic support.
        let mut count = 0;
        for (id, _gamepad) in self.gilrs.gamepads() {
            if count == index {
                return Some(id);
            }
            count += 1;
        }
        None
    }
}

impl InputSource for GilrsInputSource {
    fn get_state(&mut self, controller_index: u32) -> Result<u32, InputError> {
        // 1. Pump events to update state
        while let Some(_) = self.gilrs.next_event() {}

        // 2. Find target gamepad
        let gamepad_id = match self.get_gamepad_by_index(controller_index) {
            Some(id) => id,
            None => return Err(InputError::Disconnected),
        };

        // 3. Get state
        // Gilrs doesn't give a "bitmap" directly like XInput. We must construct it.
        // Or we can map specific buttons.
        // For IIDX controllers via generic HID, buttons are usually 0..N.
        // We need to map these to our u16 bitmask.
        //
        // Assumption: The bitmask in this app is treated as "Bit N corresponds to Button N".
        // XInput: wButtons (A=bit12, B=bit13 etc).
        // HID: Generic buttons 1-16.
        //
        // To maintain compatibility, we should define a mapping.
        // However, the architecture says "ButtonMap: Physical(Bitmask) -> LogicalKey".
        // So as long as we produce a consistent Bitmask for the device, the user can remap it.
        //
        // Strategy: Map Gilrs buttons to bits.
        // South -> Bit 0, East -> Bit 1... or just use raw button indices if available?
        // Gilrs provides `is_pressed(Button)`.
        // It also provides access to raw button code via `state().button_data(code)`.

        let gamepad = self.gilrs.gamepad(gamepad_id);
        let mut bitmap: u32 = 0;

        // Try standard buttons first (common for HID mappings)
        // This might need adjustment for specific controllers.
        // DJ DAO usually maps to generic buttons.
        // Let's iterate over a range of raw codes if possible, or standard buttons.
        // Gilrs abstracts this.
        //
        // Critical: User says "DirectInput (HID Standard)".
        // We should try to capture "all pressed buttons" into a bitmap.
        // Since `gilrs` is high-level, let's check standard buttons.

        use gilrs::Button;
        let buttons = [
            (Button::South, 0), (Button::East, 1), (Button::North, 2), (Button::West, 3),
            (Button::C, 4), (Button::Z, 5), // 6-button pads
            (Button::LeftTrigger, 6), (Button::LeftTrigger2, 7),
            (Button::RightTrigger, 8), (Button::RightTrigger2, 9),
            (Button::Select, 10), (Button::Start, 11),
            (Button::Mode, 12), (Button::LeftThumb, 13), (Button::RightThumb, 14),
            // dpad
            (Button::DPadUp, 15), (Button::DPadDown, 16), (Button::DPadLeft, 17), (Button::DPadRight, 18),
        ];

        // Construct a u32 bitmap.
        // Note: u16 is small for modern controllers (can have 32+ buttons).
        // XInput wButtons is u16.
        // If the user's controller has buttons mapping to indices > 15, we might lose them with u16.
        // Architecture uses u16. For Rev 2, maybe we should have upgraded to u32/u64?
        // For now, adhere to u16.

        // Let's assume standard mapping for now.
        // If the controller is generic, Gilrs might map Button 1 to South, Button 2 to East, etc.

        for (btn, bit_pos) in buttons.iter() {
            if *bit_pos < 32 && gamepad.is_pressed(*btn) {
                bitmap |= 1 << bit_pos;
            }
        }

        Ok(bitmap)
    }

    fn enumerate_controllers(&mut self) -> Result<Vec<crate::domain::models::ControllerInfo>, InputError> {
        let mut controllers = Vec::new();
        for (_id, gamepad) in self.gilrs.gamepads() {
            let uuid_bytes = gamepad.uuid();
            let uuid_str = uuid_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            controllers.push(crate::domain::models::ControllerInfo {
                id: uuid_str,
                name: gamepad.name().to_string(),
            });
        }
        Ok(controllers)
    }
}
