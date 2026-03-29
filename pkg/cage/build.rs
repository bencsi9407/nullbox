fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    if target.contains("musl") {
        // libkrun is a dynamic library — cage must link dynamically
        println!("cargo:rustc-link-lib=dylib=krun");
        println!("cargo:rustc-link-search=native=/usr/lib");
    } else {
        pkg_config::probe_library("libkrun").expect(
            "libkrun not found. Install: pacman -S libkrun libkrunfw",
        );
    }
}
