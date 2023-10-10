use std::{
    ffi::{OsStr, OsString},
    os::windows::prelude::OsStringExt,
};

fn main() {
    let x = String::from_utf16(&[0xFFEF, 0x0041, 0x0042, 0x0043]).unwrap();
    println!("{:?}", x);
}
