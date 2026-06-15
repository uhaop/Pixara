use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::build();

    #[cfg(target_os = "windows")]
    if env::var("CARGO_FEATURE_HEIC").is_ok() && env::var("PIXARA_PUBLIC").is_err() {
        copy_heic_runtime_dlls();
    }
}

#[cfg(target_os = "windows")]
fn copy_heic_runtime_dlls() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = manifest_dir.join("target").join(profile);

    let Some(vcpkg_root) = resolve_vcpkg_root() else {
        println!(
            "cargo:warning=HEIC runtime DLLs were not copied. Set VCPKG_ROOT or install vcpkg under %USERPROFILE%\\vcpkg."
        );
        return;
    };

    let bin_dir = vcpkg_root.join("installed").join("x64-windows").join("bin");
    if !bin_dir.is_dir() {
        println!(
            "cargo:warning=HEIC runtime DLLs were not copied. Missing vcpkg bin directory: {}",
            bin_dir.display()
        );
        return;
    }

    let required = ["heif.dll", "libde265.dll", "libx265.dll"];
    if let Err(err) = fs::create_dir_all(&target_dir) {
        println!("cargo:warning=Failed to create target dir {}: {err}", target_dir.display());
        return;
    }

    for dll in required {
        let source = bin_dir.join(dll);
        if !source.is_file() {
            println!(
                "cargo:warning=HEIC runtime DLL missing in vcpkg: {}",
                source.display()
            );
            continue;
        }
        let destination = target_dir.join(dll);
        if let Err(err) = fs::copy(&source, &destination) {
            println!(
                "cargo:warning=Failed to copy {} to {}: {err}",
                source.display(),
                destination.display()
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn resolve_vcpkg_root() -> Option<PathBuf> {
    if let Ok(root) = env::var("VCPKG_ROOT") {
        let path = PathBuf::from(root);
        if path.is_dir() {
            return Some(path);
        }
    }

    env::var("USERPROFILE")
        .ok()
        .map(|home| PathBuf::from(home).join("vcpkg"))
        .filter(|path| path.is_dir())
}
