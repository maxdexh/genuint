use std::{env, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or(std::env::VarError::NotPresent)?;
    let dest_path = Path::new(&out_dir).join("consts.rs");

    let mut out = String::new();
    write_small(&mut out)?;
    std::fs::write(&dest_path, out)?;

    Ok(())
}

fn write_small(mut out: impl std::fmt::Write) -> std::fmt::Result {
    for i in 2..=256 {
        writeln!(out, "bisect!(_{i}, {i}, _{h}, _{p});", h = i / 2, p = i % 2)?;
    }
    Ok(())
}
