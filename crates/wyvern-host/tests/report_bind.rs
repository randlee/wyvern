//! Bind discriminant: `require_report_page` is not packaged `ui/report/index.html`.

mod support;
use support::http::wait_for_url_file;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use wyvern_host::{begin, HostError, HostOptions, ViewerMode};
use wyvern_schema::{Command, ReportCommand, ReportMode, ReportPagePath, ReportTitle};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_ui_root() -> PathBuf {
    workspace_root().join("ui")
}

fn unique_path(prefix: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()))
}

fn report_command(page: &str) -> Command {
    Command::Report(ReportCommand {
        title: ReportTitle::new("bind"),
        page: ReportPagePath::new(page),
        mode: ReportMode::View,
        panels: None,
        width: None,
        height: None,
    })
}

fn host_options(ui_root: PathBuf, url_file: PathBuf) -> HostOptions {
    HostOptions {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        ui_root,
        shared_ui_root: workspace_ui_root(),
        viewer: ViewerMode::None,
        dialog_url_env: true,
        dialog_url_file: Some(url_file),
        allow_non_loopback: false,
        session_timeout: Duration::from_secs(15),
        mock_picker: None,
    }
}

#[test]
fn report_bind_rejects_packaged_report_index_layout() {
    let root = unique_path("wyvern-report-bind-packaged");
    std::fs::create_dir_all(root.join("report")).expect("report dir");
    std::fs::write(root.join("report").join("index.html"), "<html>nope</html>").expect("index");
    let err = begin(
        report_command("pages/view.xhtml"),
        host_options(root.clone(), unique_path("wyvern-report-bind-url")),
    )
    .expect_err("packaged report/index.html must not satisfy require_report_page");
    match err {
        HostError::UiNotFound { path, .. } => {
            assert!(
                path.ends_with("pages/view.xhtml"),
                "must fail on command page path, got {}",
                path.display()
            );
        }
        other => panic!("expected UiNotFound, got {other:?}"),
    }
}

#[test]
fn report_bind_accepts_page_without_report_index() {
    let root = unique_path("wyvern-report-bind-page");
    std::fs::create_dir_all(root.join("pages")).expect("pages");
    std::fs::write(
        root.join("pages").join("view.xhtml"),
        "<section>ok</section>",
    )
    .expect("page");
    assert!(
        !root.join("report").join("index.html").is_file(),
        "fixture must not include packaged report/index.html"
    );
    let url_file = unique_path("wyvern-report-bind-ok-url");
    let handle = begin(
        report_command("pages/view.xhtml"),
        host_options(root, url_file.clone()),
    )
    .expect("page file is enough");
    let dialog_url = wait_for_url_file(&url_file);
    assert!(
        dialog_url.contains("/report/pages/view.xhtml"),
        "{dialog_url}"
    );
    assert!(!dialog_url.contains("/wizard/"), "{dialog_url}");
    let _ = handle.viewer_exited_without_result();
    let _ = std::fs::remove_file(url_file);
}
