//! Timeless XI: the Timeless interface's own features (Config > Settings > Client > Interface = Timeless), kept in
//! their own files so upstream merges stay small.

pub mod angler;
pub mod book;
pub mod fishing;
pub mod hdicon;
pub mod hotbar;
pub mod icons;
pub mod manifest;
pub mod ring;
pub mod setup;
pub mod title;
// TimelessXI: the standalone build's server side (Timeless DAT patches, the allowed game version, the required
// mods at first start), reached as client::timeless::<fn> as before it moved in here.
mod server;
pub use server::*;
pub mod look_files;
pub mod looks;
