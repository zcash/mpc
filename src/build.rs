extern crate gcc;

fn main() {
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:rustc-link-lib=gmpxx");
    println!("cargo:rustc-link-lib=sodium");

    let mut cfg = gcc::Config::new();

     cfg.cpp(true)
        .define("NO_PROCPS", None)
        .define("STATIC", None)
        .define("CURVE_ALT_BN128", None)
        .define("MONTGOMERY_OUTPUT", None)
        .define("USE_ASM", None)
        .define("NO_PT_COMPRESSION", None)
        .define("BINARY_OUTPUT", None)
        .flag("-std=c++11")
        .include("libsnark/src")
        .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_g1.cpp")
        .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_g2.cpp")
        .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_init.cpp")
        .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_pairing.cpp")
        .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_pp.cpp")
        .file("libsnark/src/common/utils.cpp")
        .file("libsnark/src/common/profiling.cpp")
        .file("src/bnwrap.cpp")
    ;
    
    cfg.compile("libbnwrap.a");
}
