//! The Windows exe: its icon (assets/vana-client.rc) linked into the binaries, through rc.exe (MSVC) or windres (MinGW,
//! cross builds too; RC_<target> or RC names another compiler), and a 16 MB main-thread stack.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Windows gives the main thread 1 MB of stack (Linux 8): a debug build's frames overflow it at start.
    match std::env::var("CARGO_CFG_TARGET_ENV").as_deref() {
        Ok("msvc") => println!("cargo:rustc-link-arg-bins=/STACK:16777216"),
        Ok("gnu") => println!("cargo:rustc-link-arg-bins=-Wl,--stack,16777216"),
        _ => {}
    }
    println!("cargo:rerun-if-changed=assets/vana-client.rc");
    println!("cargo:rerun-if-changed=assets/vana-client.ico");
    // Timeless XI: the exe carries the Timeless hourglass icon (assets/timeless/timeless.ico).
    println!("cargo:rerun-if-changed=assets/timeless/timeless.ico");
    // rc.exe looks for the icon in its include dirs, not beside the .rc.
    let assets = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
    // Timeless XI: the version resource, so Windows names the app "Timeless XI" (taskbar, Task Manager, Open with,
    // notifications) whatever the exe file is called (the release asset is vana-client-windows-x86_64.exe).
    let ver = std::env::var("CARGO_PKG_VERSION").unwrap();
    let (major, minor, patch) = (
        std::env::var("CARGO_PKG_VERSION_MAJOR").unwrap(),
        std::env::var("CARGO_PKG_VERSION_MINOR").unwrap(),
        std::env::var("CARGO_PKG_VERSION_PATCH").unwrap(),
    );
    let build: String = ver.rsplit('.').next().unwrap_or("0").chars().filter(char::is_ascii_digit).collect();
    let nums = format!("{major},{minor},{patch},{}", if build.is_empty() { "0" } else { &build });
    let rc = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("timeless-xi.rc");
    std::fs::write(
        &rc,
        format!(
            r#"1 ICON "timeless/timeless.ico"
1 VERSIONINFO
FILEVERSION {nums}
PRODUCTVERSION {nums}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "Timeless XI"
      VALUE "FileDescription", "Timeless XI"
      VALUE "ProductName", "Timeless XI"
      VALUE "InternalName", "TimelessXI"
      VALUE "OriginalFilename", "TimelessXI.exe"
      VALUE "FileVersion", "{ver}"
      VALUE "ProductVersion", "{ver}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
        ),
    )
    .unwrap();
    if let Err(e) = embed_resource::compile(&rc, embed_resource::ParamsIncludeDirs([assets])).manifest_optional() {
        println!("cargo:warning=no exe icon: {e}");
    }
}
