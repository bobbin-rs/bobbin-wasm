use module::Module;

use core::fmt::Write;

pub fn dump_module<W: Write>(w: W, m: &Module) {
    writeln!(w, "Dumping Module:");
}