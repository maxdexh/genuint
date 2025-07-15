use std::{env, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or(std::env::VarError::NotPresent)?;
    let dest_path = Path::new(&out_dir).join("consts.rs");

    let mut out = String::new();
    write_small(&mut out)?;
    write_big(
        &mut out,
        env::var("CARGO_CFG_TARGET_POINTER_WIDTH")?.parse()?,
    )?;
    std::fs::write(&dest_path, out)?;

    Ok(())
}

fn write_small(mut out: impl std::fmt::Write) -> std::fmt::Result {
    for i in 2..255 {
        writeln!(out, "bisect!(U{i}, {i}, U{h}, U{p});", h = i / 2, p = i % 2)?;
    }
    Ok(())
}
fn write_big(mut out: impl std::fmt::Write, tpw: usize) -> std::fmt::Result {
    writeln!(
        out,
        "pub type UsizeMax = big!({bits});",
        bits = "U1 ".repeat(tpw)
    )
}
