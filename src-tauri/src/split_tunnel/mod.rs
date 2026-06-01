//! Split tunneling — per-app VPN routing с OS-level enforcement.

pub mod defaults;
pub mod detector;
pub mod engine;
pub mod log;
pub mod model;
pub mod platform;
pub mod singbox_rules;
pub mod store;

pub use engine::SplitTunnelEngine;
