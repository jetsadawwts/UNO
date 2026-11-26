//! HTML views, charts, assets and the local live server.
mod assets;
mod charts;
mod html;
mod manual;
mod panes;
mod server;
mod views;

pub(crate) use assets::*;
pub(crate) use charts::*;
pub(crate) use html::*;
pub use manual::manual_html;
pub use panes::{diff_message, render_panes, Panes};
pub use server::Server;
pub use views::{render_page, PageMode};
