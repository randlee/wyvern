//! Agent DAG post export contract; assert no execute/spawn API (g.7).

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
    panic!("python3, py, or python is required for agent-dag export tests");
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
            script("export-agent-dag.py").to_string_lossy().into_owned(),
        )),
    }
}

fn pair_dag() -> Value {
    json!({
        "layout_id": "pair",
        "nodes": [
            { "id": "node-1", "name": "planner", "role": "plan" },
            { "id": "node-2", "name": "reviewer", "role": "review" }
        ],
        "edges": [
            ["node-1", "node-2"]
        ]
    })
}

fn finish_dag(dag: &Value) -> Value {
    json!({
        "button": "finish",
        "data": { "dag": dag },
        "stack": []
    })
}

fn assert_dag_wire_shape(dag: &Value) {
    assert!(dag["layout_id"].as_str().is_some_and(|id| !id.is_empty()));
    let nodes = dag["nodes"].as_array().expect("nodes");
    assert!(!nodes.is_empty());
    for node in nodes {
        assert!(node["id"].as_str().is_some_and(|id| !id.is_empty()));
        assert!(node["name"].as_str().is_some_and(|name| !name.is_empty()));
        assert!(node["role"].as_str().is_some_and(|role| !role.is_empty()));
        assert_eq!(node.as_object().map(|obj| obj.len()), Some(3));
    }
    let edges = dag["edges"].as_array().expect("edges");
    assert!(!edges.is_empty());
    for edge in edges {
        let pair = edge.as_array().expect("[from, to]");
        assert_eq!(pair.len(), 2);
        assert!(pair[0].as_str().is_some_and(|from| !from.is_empty()));
        assert!(pair[1].as_str().is_some_and(|to| !to.is_empty()));
    }
}

fn run_script(repo: &Path, extra_args: &[&str], finish: &Value) -> std::process::Output {
    let mut child = std::process::Command::new(resolve_python())
        .arg(script("export-agent-dag.py"))
        .args(extra_args)
        .env("WYVERN_REPO_ROOT", repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn export-agent-dag.py");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(finish.to_string().as_bytes())
        .expect("write finish");
    child.wait_with_output().expect("wait")
}

fn assert_no_execute_hook(source: &str, label: &str) {
    let lowered = source.to_ascii_lowercase();
    for needle in [
        "\"execute\"",
        "workflow.execute",
        "subprocess",
        "popen",
        "task delegation",
        "spawn_agent",
        "rust dag",
    ] {
        assert!(
            !lowered.contains(needle),
            "{label} must not contain execute/spawn hook {needle:?}"
        );
    }
}

#[test]
fn shipped_wizard_declares_post_and_has_no_execute_hook() {
    let wizard_path = workspace_share().join("examples/agent-dag/wizard.json");
    let raw = std::fs::read_to_string(&wizard_path).unwrap();
    let wizard: Value = serde_json::from_str(&raw).unwrap();
    wyvern_schema::validate(&wizard).expect("wizard schema");
    assert_eq!(wizard["page"]["id"], "canvas");
    assert_eq!(wizard["page"]["layout"], "workspace");
    assert_eq!(
        wizard["workflow"]["post"],
        "{wyvern_share}/scripts/ext/export-agent-dag.py"
    );
    assert!(wizard["workflow"].get("pre").is_none() || wizard["workflow"]["pre"].is_null());
    assert!(wizard.get("execute").is_none());
    assert!(wizard["workflow"].get("execute").is_none());
    let layouts = wizard["config"]["layouts"].as_array().expect("layouts");
    let ids: Vec<&str> = layouts
        .iter()
        .map(|row| row["id"].as_str().expect("id"))
        .collect();
    assert_eq!(ids, ["solo", "pair", "trio"]);
    assert_eq!(layouts[0]["agents"], 1);
    assert_eq!(layouts[1]["agents"], 2);
    assert_eq!(layouts[2]["agents"], 3);

    let example = workspace_share().join("examples/agent-dag");
    let canvas = std::fs::read_to_string(example.join("pages/canvas.html")).unwrap();
    assert!(
        canvas.contains("turbo-flow-canvas") && canvas.contains("/wizard/dist/canvas.js"),
        "agent-dag must embed the turbo-flow canvas bundle"
    );
    assert!(example.join("dist/canvas.js").is_file());
    assert!(example.join("stack-merge.js").is_file());

    let app = std::fs::read_to_string(example.join("app.js")).unwrap();
    assert!(app.contains("config.layouts") || app.contains("config && config.layouts"));
    assert!(
        app.contains("assembleDag") && (app.contains("data.dag") || app.contains("dag:")),
        "page JS must assemble data.dag from the turbo-flow graph"
    );
    assert_no_execute_hook(&raw, "wizard.json");
    assert_no_execute_hook(&app, "app.js");
    let script_src = std::fs::read_to_string(script("export-agent-dag.py")).unwrap();
    assert_no_execute_hook(&script_src, "export-agent-dag.py");
    assert!(
        !script_src.contains("subprocess") && !script_src.contains("Popen"),
        "export script must not spawn processes"
    );
}

#[test]
fn dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let dest = repo.join("wyvern-dag-export.json");
    let runner = runner_for(tmp.path(), &repo);
    let finish = finish_dag(&pair_dag());

    runner
        .run_post(&post_spec(), &finish, true)
        .expect("dry-run post");
    assert!(
        !dest.exists(),
        "--workflow-dry-run must not write {}",
        dest.display()
    );

    let output = run_script(&repo, &["--dry-run"], &finish);
    assert!(
        output.status.success(),
        "export-agent-dag --dry-run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !dest.exists(),
        "script --dry-run must not write {}",
        dest.display()
    );
}

#[test]
fn post_writes_default_export_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let dest = repo.join("wyvern-dag-export.json");
    let runner = runner_for(tmp.path(), &repo);
    let dag = pair_dag();

    runner
        .run_post(&post_spec(), &finish_dag(&dag), false)
        .expect("export post");

    assert!(dest.is_file(), "missing {}", dest.display());
    let exported: Value = serde_json::from_str(&std::fs::read_to_string(&dest).unwrap()).unwrap();
    assert_eq!(exported, dag);
    assert_dag_wire_shape(&exported);
}

#[test]
fn output_flag_is_script_only() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let dest = repo.join("custom-dag.json");
    let finish = finish_dag(&pair_dag());
    let output = run_script(&repo, &["-o", dest.to_str().expect("utf8 dest")], &finish);
    assert!(
        output.status.success(),
        "export -o failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dest.is_file(), "missing {}", dest.display());
    assert!(!repo.join("wyvern-dag-export.json").exists());
    let exported: Value = serde_json::from_str(&std::fs::read_to_string(&dest).unwrap()).unwrap();
    assert_eq!(exported, pair_dag());
}
