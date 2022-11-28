use crate::args::{LogArgs, Verbosity};

static mut COLOURISE: bool = true;
static mut FATAL_WARNINGS: bool = false;
static mut VERBOSITY: Verbosity = Verbosity::Terse;

pub fn init(args: LogArgs) {
    unsafe {
        COLOURISE = args.colour;
        FATAL_WARNINGS = args.fatal_warnings;
        VERBOSITY = args.verbosity;
    }
}
