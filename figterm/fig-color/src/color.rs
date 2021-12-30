//! `color.h` bindings

#![allow(
    clippy::all,
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    deref_nullptr,
    unaligned_references,
    unused
)]

include!(concat!(env!("OUT_DIR"), "/color.rs"));
