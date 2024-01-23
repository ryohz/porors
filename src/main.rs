mod audio;
mod config;
mod font;
mod output;
mod time;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, short)]
    session: String,
    #[clap(long, short)]
    break_: String,
    #[clap(long, short)]
    number: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let session_sec = time::parse_as_sec(&args.session).unwrap();
    let break_sec = time::parse_as_sec(&args.break_).unwrap();
    let number = args.number;

    time::timer(session_sec, break_sec, number);
}
