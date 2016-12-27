mod s_type;
mod s_function;
mod s_memory;
mod s_export;
mod s_code;

pub use self::s_type::*;
pub use self::s_function::*;
pub use self::s_memory::*;
pub use self::s_export::*;
pub use self::s_code::*;

use buf::Buf;
use Error;

pub struct NameSection<'a>(pub &'a [u8]);
pub struct ImportSection<'a>(pub &'a [u8]);
pub struct TableSection<'a>(pub &'a [u8]);
pub struct GlobalSection<'a>(pub &'a [u8]);
pub struct StartSection<'a>(pub &'a [u8]);
pub struct ElementSection<'a>(pub &'a [u8]);
pub struct DataSection<'a>(pub &'a [u8]);
