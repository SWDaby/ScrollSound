//! Platform-neutral core: all decision logic, zero Win32 imports.
//! See openspec change rewrite-in-rust design D10/D11.

pub mod autostart;
pub mod events;
pub mod settings;
pub mod table;