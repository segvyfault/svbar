use std::sync::{Arc, Mutex};
use libpulse_binding::{
    mainloop::standard::Mainloop,
    proplist::Proplist,
    operation::State as OperationState,
    context::{Context, FlagSet, State},
    def::Retval
};

type VolumeResult = Result<u8, String>;
pub struct VolumeContext {
    mainloop: Mainloop,
    context: Context,
}

impl VolumeContext {
    pub fn new() -> Result<Self, String> { 
        let mut mainloop = match Mainloop::new() {
            Some(ml) => ml,
            None => return Err("Failed to create a mainloop".into())
        };

        let proplist = match Proplist::new() {
            Some(p) => p,
            None => return Err("Failed to construct a proplist".into())
        };

        let mut context = match Context::new_with_proplist(&mainloop, "CBAR_VOLUME", &proplist) {
            Some(c) => c,
            None => return Err("Failed to create a pulseaudio context".into())
        };

        if let Err(why) = context.connect(None, FlagSet::NOFLAGS, None) {
            return Err(format!("Couldn't connect to pulseaudio server: {why}"));
        }

        loop {
            match context.get_state() {
                State::Failed | State::Terminated => {
                    return Err("Context has failed or was terminated".into());
                }
                State::Ready => break,
                _ => { 
                    mainloop.iterate(false);
                }
            }
        }

        Ok(Self {
            mainloop,
            context
        })
    }

    pub fn get(&mut self) -> VolumeResult {
        let volume = Arc::new(Mutex::new(Err("Value wasn't changed".to_string()) as VolumeResult));
        let copy = volume.clone();

        let op = self.context.introspect().get_sink_info_list(move |info| {
            match info {
                libpulse_binding::callbacks::ListResult::Item(sink) => {
                    if sink.mute {
                        *copy.lock().unwrap() = Ok(0);
                    }
                    else {
                        let mut raw_volume = sink.volume.avg().print().trim().to_string();
                        raw_volume.remove(raw_volume.len() - 1);

                        let new_value = match raw_volume.parse::<u8>() {
                            Ok(volume) => Ok(volume),
                            Err(why) => Err(why.to_string())
                        };

                        *copy.lock().unwrap() = new_value;
                    }
                }
                _ => {}
            }
        });

        while op.get_state() == OperationState::Running {
            self.mainloop.iterate(false);
        }

        volume.lock().unwrap().clone()
    }

    pub fn exit(&mut self) {
        self.context.disconnect();
        self.mainloop.quit(Retval(0));
    }
}

#[test]
fn get_volume() {
    let mut context = VolumeContext::new().unwrap();

    match context.get() {
        Ok(value) => println!("{value}"),
        Err(why) => panic!("{why}")
    }
}
