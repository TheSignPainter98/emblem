mod args;

use args::{Args};

fn main() {
    let args = Args::parse();
    println!("{:#?}", args);
}
