use std::env;
use std::path::{Path, PathBuf};

use autocxx_bindgen::Builder as BindgenBuilder;

fn configure_and_generate(builder: BindgenBuilder, ida: &Path, output: impl AsRef<Path>) {
    let rs = PathBuf::from(env::var("OUT_DIR").unwrap()).join(output.as_ref());
    let bindings = builder
        .clang_arg("-xc++")
        .clang_arg(format!("-I{}", ida.display()))
        .clang_args(
            #[cfg(target_os = "linux")]
            &["-std=c++17", "-D__LINUX__=1", "-D__EA64__=1"],
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            &["-std=c++17", "-D__MACOS__=1", "-D__ARM__=1", "-D__EA64__=1"],
            #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
            &["-std=c++17", "-D__MACOS__=1", "-D__EA64__=1"],
            #[cfg(target_os = "windows")]
            &["-std=c++17", "-D__NT__=1", "-D__EA64__=1"],
        )
        .respect_cxx_access_specs(true)
        .generate()
        .expect("generate bindings");

    bindings.write_to_file(rs).expect("write bindings");
}

fn main() {
    let sdk_path = PathBuf::from(env::var("IDASDKDIR").expect("IDASDKDIR should be set"));
    let ida = sdk_path.join("include");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = manifest_dir.join("src");
    
    cxx_build::CFG.exported_header_dirs.push(&ida);
    cxx_build::CFG.exported_header_dirs.push(&src_dir);

    // Build the CXX bridge for type builders
    let mut cxx_bridge = cxx_build::bridge("src/types_bridge.rs");
    cxx_bridge
        .include(&ida)
        .include(&src_dir)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-D__MACOS__=1")
        .flag_if_supported("-D__EA64__=1")
        .flag_if_supported("-D__ARM__=1");

    // Add IDA version define if ida92 feature is enabled
    if env::var("CARGO_FEATURE_IDA92").is_ok() {
        cxx_bridge.flag_if_supported("-DIDA_SDK_VERSION_OVERRIDE=920");
    }

    cxx_bridge.compile("types_bridge");

    let ffi_path = PathBuf::from("src");

    let mut builder = autocxx_build::Builder::new(ffi_path.join("lib.rs"), [&ffi_path, &ida])
        .extra_clang_args(
            #[cfg(target_os = "linux")]
            &["-std=c++17", "-D__LINUX__=1", "-D__EA64__=1"],
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            &["-std=c++17", "-D__MACOS__=1", "-D__ARM__=1", "-D__EA64__=1"],
            #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
            &["-std=c++17", "-D__MACOS__=1", "-D__EA64__=1"],
            #[cfg(target_os = "windows")]
            &["-std=c++17", "-D__NT__=1", "-D__EA64__=1"],
        )
        .build()
        .expect("parsed correctly");

    #[cfg(target_os = "linux")]
    {
        builder
            .cargo_warnings(false)
            .flag_if_supported("-std=c++17")
            .define("__LINUX__", "1")
            .define("__EA64__", "1")
            .compile("libida-stubs");
    }

    #[cfg(target_os = "macos")]
    {
        let mut b = builder
            .cargo_warnings(false)
            .flag_if_supported("-std=c++17")
            .define("__MACOS__", "1")
            .define("__EA64__", "1");

        #[cfg(target_arch = "aarch64")]
        {
            b = b.define("__ARM__", "1");
        }

        // Add IDA version override if ida92 feature is enabled
        if env::var("CARGO_FEATURE_IDA92").is_ok() {
            b = b.define("IDA_SDK_VERSION_OVERRIDE", "920");
        }

        b.compile("libida-stubs");
    }

    #[cfg(target_os = "windows")]
    {
        builder
            .cargo_warnings(false)
            .cpp(true)
            .std("c++17")
            .define("__NT__", "1")
            .define("__EA64__", "1")
            .compile("libida-stubs");
    }

    let pod = autocxx_bindgen::builder()
        .header(ida.join("pro.h").to_str().expect("path is valid string"))
        .header(ida.join("ua.hpp").to_str().expect("path is valid string"))
        .allowlist_type("insn_t")
        .allowlist_type("op_t")
        .allowlist_type("optype_t")
        .allowlist_item("OF_.*");

    configure_and_generate(pod, &ida, "pod.rs");

    let idp = autocxx_bindgen::builder()
        .header(ida.join("pro.h").to_str().expect("path is valid string"))
        .header(ida.join("idp.hpp").to_str().expect("path is valid string"))
        .allowlist_item("PLFM_.*");

    configure_and_generate(idp, &ida, "idp.rs");

    let inf = autocxx_bindgen::builder()
        .header(ida.join("pro.h").to_str().expect("path is valid string"))
        .header(ida.join("ida.hpp").to_str().expect("path is valid string"))
        .header(
            ida.join("typeinf.hpp")
                .to_str()
                .expect("path is valid string"),
        )
        .allowlist_item("AF_.*")
        .allowlist_item("AF2_.*")
        .allowlist_item("CM_.*")
        .allowlist_item("COMP_.*")
        .allowlist_item("INFFL_.*")
        .allowlist_item("LFLG_.*")
        .allowlist_item("STT_.*")
        .allowlist_item("SW_.*")
        .allowlist_item("compiler_info_t");

    configure_and_generate(inf, &ida, "inf.rs");

    let insn_consts = [
        ("ARM_.*", "insn_arm.rs"),
        ("NN_.*", "insn_x86.rs"),
        ("MIPS_.*", "insn_mips.rs"),
    ];

    for (prefix, output) in insn_consts {
        let arch = autocxx_bindgen::builder()
            .header(ida.join("pro.h").to_str().expect("path is valid string"))
            .header(
                ida.join("allins.hpp")
                    .to_str()
                    .expect("path is a valid string"),
            )
            .clang_arg("-fshort-enums")
            .allowlist_item(prefix);

        configure_and_generate(arch, &ida, output);
    }

    let hexrays = autocxx_bindgen::builder()
        .header(ida.join("pro.h").to_str().expect("path is valid string"))
        .header(
            ida.join("hexrays.hpp")
                .to_str()
                .expect("path is valid string"),
        )
        .opaque_type("std::.*")
        .opaque_type("carglist_t")
        .allowlist_item("cfunc_t")
        .allowlist_item("citem_t")
        .allowlist_item("cexpr_t")
        .allowlist_item("cinsn_t")
        .allowlist_item("cblock_t")
        .allowlist_item("cswitch_t")
        .allowlist_item("ctry_t")
        .allowlist_item("cthrow_t")
        .allowlist_item("cnumber_t")
        .allowlist_item("lvar_t")
        .allowlist_item("lvar_locator_t")
        .allowlist_item("vdloc_t")
        .allowlist_item("CV_.*")
        .allowlist_item("DECOMP_.*");

    configure_and_generate(hexrays, &ida, "hexrays.rs");

    println!(
        "cargo::rerun-if-changed={}",
        ffi_path.join("lib.rs").display()
    );
}
