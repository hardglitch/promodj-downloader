use crate::data::consts::FLASH_DURATION_FRAMES;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ButtonState {
    pub current_flash_frame: usize,
}
impl ButtonState {
    pub fn new() -> Self {
        ButtonState {
            current_flash_frame: FLASH_DURATION_FRAMES,
        }
    }
}