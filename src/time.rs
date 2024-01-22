use anyhow::{Context, Result};

pub fn parse_duration(raw: &str) -> Result<std::time::Duration> {
    let with_unit = |unit: &str, string: &str| -> Result<(u64, String)> {
        if string.contains(unit) {
            let v = string.split(unit).collect::<Vec<&str>>();
            let num_str = v.get(0).unwrap();
            println!("{}", string);
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

    let mut second_sum = 0;

    let (h, ms_str) = with_unit("h", raw).context("parse error")?;
    second_sum += h * 3600;

    let (m, s_str) = with_unit("m", ms_str.as_str()).context("parse error")?;
    second_sum += m * 60;

    let (s, _) = with_unit("s", s_str.as_str()).context("parse error")?;
    second_sum += s;

    let dur = std::time::Duration::new(second_sum, 0);
    Ok(dur)
}

#[test]
fn parse_duration_test() {
    let h = 5 * 3600;
    let m = 3 * 60;
    let s = 8;
    let ss = h + m + s;

    let dur = std::time::Duration::new(ss, 0);
    let dur_generated = parse_duration("5h3m8s").unwrap();
    assert_eq!(dur, dur_generated)
}
