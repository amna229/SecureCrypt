//! Application HTTP handlers.
//!
//! This module groups handlers according to their responsibility,
//! including page rendering, transfer management, TLS handshake
//! processing, and dashboard notifications.

pub mod handshake;
pub mod notification;
pub mod page;
pub mod transfer;
