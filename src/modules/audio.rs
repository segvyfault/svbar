use super::ModuleInfo;
use volume::VolumeContext;

pub struct AudioModule {
    context: VolumeContext,
    previous: u8,
}
impl AudioModule {
    pub fn new() -> Self { 
        Self { 
            context: VolumeContext::new().expect("Failed to get volume context"),
            previous: 0 
        } 
    } 
}

impl ModuleInfo for AudioModule {
    fn display(&mut self) -> String {
        match self.context.get() {
            Ok(new) => {
                self.previous = new;
                new.to_string()
            }
            Err(why) => {
                println!("Failed to get volume: {why}");
                self.previous.to_string()
            }
        }
    }

    fn clean_up(&mut self) {
        let _ = self.context.exit();
    }
}
