fn main() {
    let theos = env!("THEOS");
    let rootless = option_env!("ROOTLESS").is_some();
    if (rootless) {
        println!("cargo:rustc-link-search=framework={}/vendor/lib/iphone/rootless", theos);
        println!("cargo:rustc-link-arg=-rpath");
        println!("cargo:rustc-link-arg=-/var/jb/Library/Frameworks");
        println!("cargo:rustc-link-arg=-rpath");
        println!("cargo:rustc-link-arg=-/Library/Frameworks");
    } else {
        println!("cargo:rustc-link-search=framework={}/vendor/lib/", theos);
    }
}

