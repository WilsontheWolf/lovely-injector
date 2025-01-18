fn main() {
    let theos = env!("THEOS");
    let rootless = option_env!("ROOTLESS").is_some();
    if rootless {
        println!("cargo:rustc-link-search=framework={}/vendor/lib/iphone/rootless", theos);
        println!("cargo:rustc-link-arg=-Wl,-rpath,/var/jb/Library/Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,/var/jb/usr/lib");
    } else {
        println!("cargo:rustc-link-search=framework={}/vendor/lib/", theos);
    }
}
