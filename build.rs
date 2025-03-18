fn main() {
    println!("cargo:rustc-link-lib=virt");

    // // Try using pkg-config to find libvirt
    // match pkg_config::probe_library("libvirt") {
    //     Ok(_) => println!("Found libvirt via pkg-config"),
    //     Err(e) => println!("cargo:warning=Could not find libvirt via pkg-config: {}", e),
    // }
}
