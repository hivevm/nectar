//! The core reads no environment variable itself; only what its host hands over counts. This test
//! sets one in its own process, so it lives in a test binary of its own.

use nectar_core::{MemoryFiles, Project};

#[test]
fn only_the_variables_handed_over_reach_the_documents() {
    // SAFETY: the only test of this binary, so no other thread reads the environment meanwhile.
    unsafe { std::env::set_var("NECTAR_VERSION", "2.0") };
    let files: MemoryFiles = [
        (
            "nectar.toml",
            "[[document]]\nsource = \"main.md\"\n[vars]\nVERSION = \"1.0\"\n",
        ),
        ("main.md", "# Main\n\nVersion {{VERSION}}.\n"),
    ]
    .into_iter()
    .collect();
    let version = |project: &Project| {
        let documents = project.documents();
        format!("{:?}", documents.documents[0].root.blocks)
    };

    let mut project = Project::load(Box::new(files)).unwrap();
    assert!(
        version(&project).contains("Version 1.0."),
        "nothing handed over"
    );
    project.set_environment([("NECTAR_VERSION".to_string(), "3.0".to_string())]);
    assert!(
        version(&project).contains("3.0"),
        "what the host hands over"
    );
}
