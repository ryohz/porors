mod time;

use clap::Parser;

use crate::time::parse_duration;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, short)]
    session: String,
    #[clap(long, short)]
    break_: String,
}

fn main() {
    let args = Args::parse();
    let session_dur = parse_duration(&args.session).unwrap();
    let break_dur = parse_duration(&args.break_).unwrap();
}
