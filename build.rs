fn main() {
    // Only apply Windows subsystem when GUI feature is enabled and target is Windows
    #[cfg(all(windows, feature = "gui"))]
    {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }
}
