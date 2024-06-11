// Issue #66530: We would ICE if someone compiled with `-o /dev/null`,
// because we would try to generate auxiliary files in `/dev/` (which
// at least the OS X file system rejects).
//
// An attempt to `-Ztemps-dir` into a directory we cannot write into should
// indeed be an error; but not an ICE.
//
// However, some folks run tests as root, which can write `/dev/` and end
// up clobbering `/dev/null`. Instead we'll use an inaccessible path, which
// also used to ICE, but even root can't magically write there.
//
// Note that `-Ztemps-dir` uses `create_dir_all` so it is not sufficient to
// use a directory with non-existing parent like `/does-not-exist/output`.
// See https://github.com/rust-lang/rust/issues/66530

//@ only-linux
// Reason: set_mode is only available on Unix

//@ ignore-arm
// Reason: linker error on `armhf-gnu`

use run_make_support::{fs_wrapper, rustc};

fn main() {
    // Create an inaccessible directory.
    fs_wrapper::create_dir("inaccessible");
    let meta = fs_wrapper::metadata("inaccessible");
    let mut perms = meta.permissions();
    perms.set_mode(0o000); // Lock down the directory.
    fs_wrapper::set_permissions("inaccessible", perms);

    // Run rustc with `-Z temps-dir` set to a directory *inside* the inaccessible one,
    // so that it can't create `tmp`.
    rustc()
        .input("program.rs")
        .arg("-Ztemps-dir=inaccessible/tmp")
        .run_fail()
        .assert_stderr_contains(
            "failed to find or create the directory specified by `--temps-dir`",
        );

    perms.set_mode(0o666); // Unlock the directory, so that compiletest can delete it.
    fs_wrapper::set_permissions("inaccessible", perms);
}
