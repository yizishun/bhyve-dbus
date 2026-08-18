use zbus::interface;
use crate::console::Console;

pub struct MultiTouchInterface {
    console: Console    
}
impl MultiTouchInterface {
    pub fn new(console: Console) -> Self {
	    Self { console }
    }
}
#[interface(name = "org.qemu.Display1.MultiTouch")]
impl MultiTouchInterface {
    async fn send_event(&self, kind: u32, num_slot: u64, x: f64, y: f64) {
    }

    #[zbus(property)]
    fn max_slots(&self) -> i32 {
        0
    }
}
