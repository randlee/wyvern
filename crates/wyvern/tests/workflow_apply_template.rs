//! Dry-run, apply, tagged re-apply, untagged fail, and `--force` (g.6).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde_json::{json, Value};
use wyvern::{Allowlist, WorkflowRunner, WORKFLOW_SCRIPT_TIMEOUT};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_share() -> PathBuf {
    workspace_root().join("share/wyvern")
}

fn script(name: &str) -> PathBuf {
    workspace_root().join("scripts/ext").join(name)
}

fn resolve_python() -> &'static str {
    for name in ["python3", "py", "python"] {
        if std::process::Command::new(name)
            .arg("-c")
            .arg("import sys")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return name;
        }
    }
    panic!("python3, py, or python is required for template apply tests");
}

fn runner_for(tmp: &Path, repo: &Path) -> WorkflowRunner {
    WorkflowRunner {
        allowlist: Allowlist {
            share_root: workspace_share(),
            cwd: workspace_root(),
            wizard_dir: tmp.to_path_buf(),
        },
        timeout: WORKFLOW_SCRIPT_TIMEOUT,
        extra_env: vec![(
            OsString::from("WYVERN_REPO_ROOT"),
            repo.as_os_str().to_os_string(),
        )],
    }
}

fn post_spec() -> wyvern_schema::WorkflowSpec {
    wyvern_schema::WorkflowSpec {
        pre: None,
        post: Some(wyvern_schema::WorkflowPath::new(
            script("apply-template.py").to_string_lossy().into_owned(),
        )),
    }
}

fn finish_template(template_id: &str, output_path: &str, variables: Value) -> Value {
    json!({
        "button": "finish",
        "data": {
            "template_id": template_id,
            "variables": variables,
            "output_path": output_path
        },
        "stack": []
    })
}

fn sidecar_for(dest: &Path) -> PathBuf {
    let name = format!(
        "{}.wyvern.json",
        dest.file_name().unwrap().to_string_lossy()
    );
    dest.with_file_name(name)
}

fn assert_tagged_sidecar(dest: &Path, template_id: &str) {
    let sidecar = sidecar_for(dest);
    assert!(sidecar.is_file(), "missing sidecar {}", sidecar.display());
    let value: Value = serde_json::from_str(&std::fs::read_to_string(&sidecar).unwrap()).unwrap();
    assert_eq!(value["managed_by"], "wyvern:template");
    assert_eq!(value["template_id"], template_id);
    assert_eq!(value["version"], 1);
}

fn run_script(repo: &Path, extra_args: &[&str], finish: &Value) -> std::process::Output {
    let mut child = std::process::Command::new(resolve_python())
        .arg(script("apply-template.py"))
        .args(extra_args)
        .env("WYVERN_SHARE", workspace_share())
        .env("WYVERN_REPO_ROOT", repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn apply-template.py");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(finish.to_string().as_bytes())
        .expect("write finish");
    child.wait_with_output().expect("wait")
}

#[test]
fn shipped_wizard_declares_post_and_seven_templates() {
    let wizard_path = workspace_share().join("examples/template-picker/wizard.json");
    let wizard: Value =
        serde_json::from_str(&std::fs::read_to_string(wizard_path).unwrap()).unwrap();
    wyvern_schema::validate(&wizard).expect("wizard schema");
    assert_eq!(wizard["page"]["id"], "pick");
    assert_eq!(
        wizard["workflow"]["post"],
        "{wyvern_share}/scripts/ext/apply-template.py"
    );
    let templates = wizard["config"]["templates"].as_array().expect("templates");
    let ids: Vec<&str> = templates
        .iter()
        .map(|row| row["id"].as_str().expect("id"))
        .collect();
    assert_eq!(
        ids,
        [
            "pytest",
            "github-workflow",
            "nunit",
            "xunit",
            "benchmark-dotnet",
            "wizard/minimal",
            "wizard/two-step"
        ]
    );
    let app =
        std::fs::read_to_string(workspace_share().join("examples/template-picker/app.js")).unwrap();
    assert!(app.contains("config.templates") || app.contains("config && config.templates"));
    assert!(
        !app.contains("readDir") && !app.contains("readdir"),
        "page JS must not scan the catalog directory"
    );

    for id in ids {
        let manifest = workspace_share()
            .join("templates")
            .join(id)
            .join("template.manifest.json");
        assert!(manifest.is_file(), "missing {}", manifest.display());
        let parsed: Value =
            serde_json::from_str(&std::fs::read_to_string(&manifest).unwrap()).unwrap();
        assert_eq!(parsed["id"], id);
        assert!(
            parsed["files"]
                .as_array()
                .is_some_and(|files| !files.is_empty()),
            "manifest files for {id}"
        );
    }
}

#[test]
fn dry_run_prints_plan_and_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path(), &repo);
    let dest = repo.join("tests/test_example.py");
    let finish = finish_template(
        "pytest",
        "tests/test_example.py",
        json!({"module_name": "widgets"}),
    );

    runner
        .run_post(&post_spec(), &finish, true)
        .expect("dry-run post");
    assert!(!dest.exists(), "dry-run must not write {}", dest.display());
    assert!(
        !sidecar_for(&dest).exists(),
        "dry-run must not write sidecar"
    );

    let output = run_script(&repo, &["--dry-run"], &finish);
    assert!(
        output.status.success(),
        "apply-template --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("copy ")
            && stdout.contains(" -> ")
            && stdout.contains("test_example.py")
            && stdout.contains("(create)"),
        "dry-run stdout must include copy-plan lines: {stdout}"
    );
    assert!(
        stdout.contains("sidecar ") && stdout.contains("test_example.py.wyvern.json"),
        "dry-run stdout must include sidecar plan: {stdout}"
    );
    assert!(
        !dest.exists(),
        "script --dry-run must not write {}",
        dest.display()
    );
    assert!(
        !sidecar_for(&dest).exists(),
        "script --dry-run must not write sidecar"
    );
}

#[test]
fn apply_writes_substituted_file_and_sidecar() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path(), &repo);
    let dest = repo.join("tests/test_example.py");

    runner
        .run_post(
            &post_spec(),
            &finish_template(
                "pytest",
                "tests/test_example.py",
                json!({"module_name": "widgets"}),
            ),
            false,
        )
        .expect("apply post");

    let body = std::fs::read_to_string(&dest).unwrap();
    assert!(body.contains("test_widgets"), "{body}");
    assert!(!body.contains("{module_name}"), "{body}");
    assert_tagged_sidecar(&dest, "pytest");
}

#[test]
fn tagged_reapply_overwrites() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path(), &repo);
    let dest = repo.join("tests/test_example.py");
    let finish = finish_template(
        "pytest",
        "tests/test_example.py",
        json!({"module_name": "first"}),
    );
    runner
        .run_post(&post_spec(), &finish, false)
        .expect("first apply");
    std::fs::write(&dest, "stale\n").unwrap();

    let again = finish_template(
        "pytest",
        "tests/test_example.py",
        json!({"module_name": "second"}),
    );
    runner
        .run_post(&post_spec(), &again, false)
        .expect("tagged re-apply");
    let body = std::fs::read_to_string(&dest).unwrap();
    assert!(body.contains("test_second"), "{body}");
    assert_tagged_sidecar(&dest, "pytest");
}

#[test]
fn untagged_collision_fails_and_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let dest = repo.join("tests/test_example.py");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::write(&dest, "user-owned\n").unwrap();
    let runner = runner_for(tmp.path(), &repo);

    let err = runner
        .run_post(
            &post_spec(),
            &finish_template(
                "pytest",
                "tests/test_example.py",
                json!({"module_name": "x"}),
            ),
            false,
        )
        .expect_err("untagged must fail");
    assert!(
        matches!(err, wyvern::WorkflowError::NonZero { .. }),
        "{err:?}"
    );
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "user-owned\n");
    assert!(!sidecar_for(&dest).exists());
}

#[test]
fn force_overwrites_untagged_as_script_only_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let dest = repo.join("tests/test_example.py");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::write(&dest, "user-owned\n").unwrap();

    let output = run_script(
        &repo,
        &["--force"],
        &finish_template(
            "pytest",
            "tests/test_example.py",
            json!({"module_name": "forced"}),
        ),
    );
    assert!(
        output.status.success(),
        "force failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body = std::fs::read_to_string(&dest).unwrap();
    assert!(body.contains("test_forced"), "{body}");
    assert_tagged_sidecar(&dest, "pytest");
}

#[test]
fn apply_directory_skeleton_and_reject_escape() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let runner = runner_for(tmp.path(), &repo);
    runner
        .run_post(
            &post_spec(),
            &finish_template("wizard/minimal", "wizard-minimal/", json!({})),
            false,
        )
        .expect("skeleton apply");
    let wizard = repo.join("wizard-minimal/wizard.json");
    let page = repo.join("wizard-minimal/pages/home.html");
    assert!(wizard.is_file(), "{}", wizard.display());
    assert!(page.is_file(), "{}", page.display());
    assert_tagged_sidecar(&wizard, "wizard/minimal");
    assert_tagged_sidecar(&page, "wizard/minimal");

    let output = run_script(
        &repo,
        &[],
        &finish_template("../etc/passwd", "out.py", json!({})),
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("template_id") || stderr.contains(".."),
        "{stderr}"
    );
}
