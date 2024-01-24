use std::{path::PathBuf, str::FromStr};

use once_cell::sync::Lazy;

use crate::font::AaFont;

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let bind = Bind {
        stop_start: String::from("s"),
    };
    let nav_msg = bind.nav_msg();
    let font_path = PathBuf::from_str("/home/ryohz/src/porors/fonts/ansi_shadow").unwrap();
    let aafont = AaFont::new(&font_path, true).unwrap();
    let session_end_audio = PathBuf::from_str("/home/ryohz/src/porors/sounds/alarm.mp3").unwrap();
    let break_end_audio = PathBuf::from_str("/home/ryohz/src/porors/sounds/success.mp3").unwrap();
    Config {
        nav_msg,
        bind,
        aafont,
        session_end_audio,
        break_end_audio,
    }
});

pub struct Config {
    pub nav_msg: String,
    pub bind: Bind,
    pub aafont: AaFont,
    pub session_end_audio: PathBuf,
    pub break_end_audio: PathBuf,
    pub info_saved: PathBuf,
}

pub struct Bind {
    pub stop_start: String,
}

impl Bind {
    pub fn nav_msg(&self) -> String {
        let msg = format!(
            "stop: {stop}, start: {start}",
            stop = self.stop_start,
            start = self.stop_start
        );
        msg
    }

    pub fn handle(input: &str) {
        if input == CONFIG.bind.stop_start {
            let mut stop_state = crate::time::STOP_STATE.lock().unwrap();
            *stop_state = !*stop_state;
            let stop_state_condvar = crate::time::STOP_STATE_CONDVAR.lock().unwrap();
            stop_state_condvar.notify_all();
        }
    }
}
