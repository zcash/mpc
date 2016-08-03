extern crate gcc;

const USE_ATE_PAIRING: bool = false;

fn main() {
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:rustc-link-lib=gmpxx");
    println!("cargo:rustc-link-lib=sodium");

    if USE_ATE_PAIRING {
        let mut cfg = gcc::Config::new();

         cfg.cpp(true)
            .opt_level(2)
            .define("BN_SUPPORT_SNARK", None)
            .include("ate-pairing/include")
            .include("xbyak")
            .file("ate-pairing/src/zm.cpp")
            .file("ate-pairing/src/zm2.cpp")
            .compile("libzm.a");
    }

    let mut cfg = gcc::Config::new();

    let cfg = cfg.cpp(true)
             .opt_level(2)
             .define("NO_PROCPS", None)
             .define("STATIC", None)
             .define("MONTGOMERY_OUTPUT", None)
             .define("USE_ASM", None)
             .define("NO_PT_COMPRESSION", None)
             .define("BINARY_OUTPUT", None)
             .flag("-std=c++11")
             .include("libsnark/src")
             .file("libsnark/src/common/utils.cpp")
             .file("libsnark/src/common/profiling.cpp")
             .file("src/libsnarkwrap.cpp");

    if USE_ATE_PAIRING {
        let cfg = cfg.define("CURVE_BN128", None)
                 .define("BN_SUPPORT_SNARK", None)
                 .include("ate-pairing/include")
                 .file("libsnark/src/algebra/curves/bn128/bn128_g1.cpp")
                 .file("libsnark/src/algebra/curves/bn128/bn128_g2.cpp")
                 .file("libsnark/src/algebra/curves/bn128/bn128_gt.cpp")
                 .file("libsnark/src/algebra/curves/bn128/bn128_init.cpp")
                 .file("libsnark/src/algebra/curves/bn128/bn128_pairing.cpp")
                 .file("libsnark/src/algebra/curves/bn128/bn128_pp.cpp");

        cfg.compile("libsnarkwrap.a");
    } else {
        let cfg = cfg.define("CURVE_ALT_BN128", None)
                 .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_g1.cpp")
                 .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_g2.cpp")
                 .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_init.cpp")
                 .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_pairing.cpp")
                 .file("libsnark/src/algebra/curves/alt_bn128/alt_bn128_pp.cpp");

        cfg.compile("libsnarkwrap.a");
    }
}
