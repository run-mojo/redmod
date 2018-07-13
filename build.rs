//extern crate cpp_build;
extern crate gcc;

fn main() {
    // Build a Redis pseudo-library so that we have symbols that we can link
    // against while building Rust code.
    //
    // include/redismodule.h is just vendored in from the Redis project and
    // src/redismodule.c is just a stub that includes it and plays a few other
    // tricks that we need to complete the build.
    gcc::Build::new()
//        .file("src/redismodule.c")
        .file("c/endianconv.c")
        .file("c/zmalloc.c")
        .file("c/sds.c")
        .file("c/sds_ext.c")
        .file("c/siphash.c")
        .file("c/sha1.c")
        .file("c/dict.c")
        .file("c/util.c")
        .file("c/server.c")

        .file("c/listpack.c")
        .file("c/rax.c")
        .file("c/rax_ext.c")
        .file("c/stream.c")
        .file("c/stream_ext.c")

        .file("src/redismodule.c")
        .include("c/")
        .compile("libredismodule.a");

//    gcc::compile_library()

//    gcc::Build::new()
//        .file("src/listpack.c")
//        .include("include/")
//        .compile("liblistpack.a");
    // The GCC module emits `rustc-link-lib=static=redismodule` for us.

//    cpp_build::build("src/lib.rs");
}
