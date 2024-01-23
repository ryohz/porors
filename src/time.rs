use std::{
    io::Write,
    path::PathBuf,
    str::FromStr,
    sync::{Condvar, Mutex},
};

use anyhow::{Context, Result};
use once_cell::sync::Lazy;

use crate::{config::CONFIG, output::print_top};

pub static STOP_STATE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
pub static STOP_STATE_CONDVAR: Lazy<Mutex<Condvar>> = Lazy::new(|| Mutex::new(Condvar::new()));

pub fn parse_as_sec(raw: &str) -> Result<u64> {
    let with_unit = |unit: &str, string: &str| -> Result<(u64, String)> {
        if string.contains(unit) {
            let v = string.split(unit).collect::<Vec<&str>>();
            let num_str = v.get(0).unwrap();
            let surplus = match v.get(1) {
                Some(item) => item,
                None => "",
            };
            let num = num_str
                .parse::<u64>()
                .with_context(|| format!("failed to convert \"{}\" to integer", num_str))?;

            return Ok((num, surplus.to_string()));
        } else {
            return Ok((0, string.to_string()));
        }
    };

    let mut sec_sum = 0;

    let (h, ms_str) = with_unit("h", raw).context("parse error")?;
    sec_sum += h * 3600;

    let (m, s_str) = with_unit("m", ms_str.as_str()).context("parse error")?;
    sec_sum += m * 60;

    let (s, _) = with_unit("s", s_str.as_str()).context("parse error")?;
    sec_sum += s;

    Ok(sec_sum)
}

pub fn timer(session_sec: u64, break_sec: u64, number: u32) {
    for i in 0..number {
        session(session_sec, i, number);
        std::thread::spawn(move || crate::audio::nodify_session_end());
        break_(break_sec, i, number);
        std::thread::spawn(move || crate::audio::nodify_break_end());
    }
}

fn setup_counter(
    sec: u64,
) -> (
    std::sync::Arc<std::sync::Mutex<u64>>,
    std::sync::Arc<std::sync::Condvar>,
) {
    let (progress_shared, progress_condvar) = (
        std::sync::Arc::new(std::sync::Mutex::new(sec)),
        std::sync::Arc::new(std::sync::Condvar::new()),
    );

    let (progress_shared_clone, progress_condvar_clone) =
        (progress_shared.clone(), progress_condvar.clone());

    tokio::spawn(async move {
        counter(sec, progress_shared_clone, progress_condvar_clone);
    });

    (progress_shared, progress_condvar)
}

fn session(sec: u64, count: u32, max: u32) {
    let aafont = CONFIG.aafont.clone();
    let (progress_shared, progress_condvar) = setup_counter(sec);

    let mut progress = sec;
    while progress > 0 {
        let stop_state = STOP_STATE.lock().unwrap();
        if *stop_state {
            let _ = STOP_STATE_CONDVAR.lock().unwrap().wait(stop_state);
        }
        let new_progress = progress_condvar
            .wait(progress_shared.lock().unwrap())
            .unwrap();
        progress = *new_progress;

        let h = progress / 3600;
        let m = (progress % 3600) / 60;
        let s = progress % 60;
        let output = format!("{}:{}:{}", h, m, s);
        let output = aafont.get_string(output.as_str());
        let (output, x_pad, y_pad) = output.center_aligned();
        crate::output::clear();
        println!(
            "{}\x1b[35m{}\x1b[0m{} working...\n",
            " ".repeat(x_pad),
            "▄".repeat(count as usize + 1),
            "▄".repeat(max as usize - count as usize - 1)
        );
        println!("\x1b[35m{}\x1b[0m", output);
    }
}

fn break_(sec: u64, count: u32, max: u32) {
    let aafont = CONFIG.aafont.clone();
    let (progress_shared, progress_condvar) = setup_counter(sec);

    let mut progress = sec;
    while progress > 0 {
        let new_progress = progress_condvar
            .wait(progress_shared.lock().unwrap())
            .unwrap();
        progress = *new_progress;
        println!("{}", progress);
        let h = progress / 3600;
        let m = (progress % 3600) / 60;
        let s = progress % 60;
        let output = format!("{}:{}:{}", h, m, s);
        let output = aafont.get_string(output.as_str());
        let (output, x_pad, y_pad) = output.center_aligned();
        crate::output::clear();
        println!(
            "{}\x1b[32m{}\x1b[0m{} breaking...\n",
            " ".repeat(x_pad),
            "▄".repeat(count as usize + 1),
            "▄".repeat(max as usize - count as usize - 1)
        );
        println!("\x1b[32m{}\x1b[0m", output);
    }
}

fn counter(
    sec: u64,
    progress_shared: std::sync::Arc<std::sync::Mutex<u64>>,
    condvar: std::sync::Arc<std::sync::Condvar>,
) {
    let delta = std::time::Duration::from_secs(1);
    let mut progress = sec;

    while 0 < progress {
        std::thread::sleep(delta);
        progress -= 1;
        *progress_shared.lock().unwrap() = progress;
        condvar.notify_all();
    }
}

#[test]
fn parse_as_sec_test() {
    let h = 5 * 3600;
    let m = 3 * 60;
    let s = 8;
    let sec = h + m + s;

    let sec_generated = parse_as_sec("5h3m8s").unwrap();
    assert_eq!(sec, sec_generated)
}
