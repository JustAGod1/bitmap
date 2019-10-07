extern crate cc;

use std::fs::File;
use std::ops::Shl;
use std::io::Write;
use std::ptr::{null_mut, null};

fn main() {
    cc::Build::new()
        .file("qdbmp/qdbmp.c")
        .include("qdbmp")
        .compile("qdbmp");

}