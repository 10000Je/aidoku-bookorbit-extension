#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod models;
pub mod reader;

#[cfg(target_arch = "wasm32")]
mod source;
