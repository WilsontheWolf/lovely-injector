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
    let dst = cmake::build("capstone");
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=capstone");

    cc::Build::new()
        .file("symbolfinder.c")
        .flag("-fvisibility=default")
        .include("./capstone/include/")
        .compile("symbolfinder");
}
