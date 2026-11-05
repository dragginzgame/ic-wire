use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/diagnostic-discovery-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn case(&self, name: &str, expected: Option<&[u8]>) {
        let path = self.0.join(name);
        fs::create_dir(&path).unwrap();
        if let Some(bytes) = expected {
            fs::write(path.join("expected.txt"), bytes).unwrap();
        }
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn missing_required_expectation_is_an_error() {
    let directory = Directory::new();
    directory.case("first", Some(b"first diagnostic\n"));
    directory.case("missing", None);
    assert_eq!(
        diagnostic_cases(&directory.0).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn rejects_empty_inventory_and_invalid_case_content() {
    let directory = Directory::new();
    directory.case("valid", None);
    directory.case("support", None);
    assert_eq!(
        diagnostic_cases(&directory.0).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    directory.case("bad", Some(&[255]));
    assert_eq!(
        diagnostic_cases(&directory.0).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn sorts_cases_and_skips_only_explicit_wiring_and_controls() {
    let directory = Directory::new();
    directory.case("z", Some(b"z diagnostic\n"));
    directory.case("a", Some(b"a diagnostic\nsecond line\n"));
    directory.case("valid", None);
    directory.case("support", None);
    fs::write(directory.0.join("mod.rs"), b"// root").unwrap();
    assert_eq!(
        diagnostic_cases(&directory.0).unwrap(),
        vec![
            (
                "a".into(),
                vec!["a diagnostic".into(), "second line".into()]
            ),
            ("z".into(), vec!["z diagnostic".into()]),
        ]
    );
}
