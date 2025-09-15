#![expect(missing_docs)]

#[test]
#[cfg(not(miri))] // Miri does not work with `walkdir`.
fn trybuild() -> std::io::Result<()> {
    let t = trybuild::TestCases::new();

    for entry in walkdir::WalkDir::new("tests/ui") {
        let entry = entry?;
        if entry.path().extension().unwrap_or_default() == "rs" {
            // To add a new compile-fail test, write the test `tests/ui/foo.rs` file then `touch tests/ui/foo.stderr` to
            // mark it as compile-fail and `TRYBUILD=overwrite cargo test -p veecle-os-runtime-macros` to fill the expected
            // messages.
            if entry.path().with_extension("stderr").exists() {
                t.compile_fail(entry.path());
            } else {
                t.pass(entry.path());
            }
        }
    }

    Ok(())
}
