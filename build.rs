fn main() {
    // Only apply Windows subsystem when GUI feature is enabled and target is Windows
    #[cfg(all(windows, feature = "gui"))]
    {
        // Windows resource compilation
        #[cfg(windows)]
        {
            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/pipe.ico");
            res.set("ProductName", "Pipe Network Firestarter");
            res.set("FileDescription", "Pipe Network Storage Client");
            res.set("CompanyName", "Pipe Network");
            res.set("LegalCopyright", "Copyright (C) 2024 Pipe Network");
            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to compile Windows resources: {}", e);
            }
        }

        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }
}
