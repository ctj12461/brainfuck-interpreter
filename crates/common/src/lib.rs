//! # brainfuck-interpreter
//! A fast, powerful and configurable interpreter written in Rust,
//! which allows various options to meet different demends, including
//! memory (tape) length configuration, EOF handling configuration and
//! so on.
//!
//! Licensed under MIT.
//!
//! Copyright (C) 2023 Justin Chen (ctj12461)
//!

#![allow(
    clippy::collapsible_else_if,
    clippy::new_without_default,
    clippy::comparison_chain
)]

pub mod compiler;
pub mod execution;
