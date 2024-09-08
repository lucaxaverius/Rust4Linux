savedcmd_/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.o := RUST_MODFILE=/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test rustc --edition=2021 -Zbinary_dep_depinfo=y -Dunsafe_op_in_unsafe_fn -Dnon_ascii_idents -Wrust_2018_idioms -Wunreachable_pub -Wmissing_docs -Wrustdoc::missing_crate_level_docs -Wclippy::all -Wclippy::mut_mut -Wclippy::needless_bitwise_bool -Wclippy::needless_continue -Wclippy::no_mangle_with_rust_abi -Wclippy::dbg_macro -Cpanic=abort -Cembed-bitcode=n -Clto=n -Cforce-unwind-tables=n -Ccodegen-units=1 -Csymbol-mangling-version=v0 -Crelocation-model=static -Zfunction-sections=n -Wclippy::float_arithmetic --target=./scripts/target.json -Ctarget-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2 -Ztune-cpu=generic -Cno-redzone=y -Ccode-model=kernel -Copt-level=2 -Cdebug-assertions=n -Coverflow-checks=y -Cforce-frame-pointers=y -Zdwarf-version=5 -Cdebuginfo=2  --cfg MODULE  @./include/generated/rustc_cfg -Zallow-features=new_uninit -Zcrate-attr=no_std -Zcrate-attr='feature(new_uninit)' -Zunstable-options --extern force:alloc --extern kernel --crate-type rlib -L ./rust/ --crate-name jiffies_test --sysroot=/dev/null --out-dir /home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/ --emit=dep-info=/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/.jiffies_test.o.d --emit=obj=/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.o /home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.rs

source_/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.o := /home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.rs

deps_/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.o := \
  /home/rustxave/src/kernel/linux/rust/libcore.rmeta \
  /home/rustxave/src/kernel/linux/rust/libcompiler_builtins.rmeta \
  /home/rustxave/src/kernel/linux/rust/libkernel.rmeta \
  /home/rustxave/src/kernel/linux/rust/liballoc.rmeta \
  /home/rustxave/src/kernel/linux/rust/libmacros.so \
  /home/rustxave/src/kernel/linux/rust/libbindings.rmeta \
  /home/rustxave/src/kernel/linux/rust/libuapi.rmeta \
  /home/rustxave/src/kernel/linux/rust/libbuild_error.rmeta \

/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.o: $(deps_/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.o)

$(deps_/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/jiffies/jiffies_test.o):
