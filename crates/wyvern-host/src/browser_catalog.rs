//! Hardcoded browser catalog — id → display name + per-OS discovery recipes.

use std::path::{Path, PathBuf};

use crate::options::BrowserId;

/// A catalog browser that may be discovered and cached in the registry.
#[derive(Debug, Clone, Copy)]
pub struct CatalogBrowser {
    /// Stable registry id (`chrome`, `firefox`, …).
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Env override for an explicit executable path (`WYVERN_CHROME_PATH`, …).
    pub path_env: &'static str,
}

/// Full catalog (includes ids beyond the c.15 `--viewer` enum).
pub fn catalog() -> &'static [CatalogBrowser] {
    &[
        CatalogBrowser {
            id: "chrome",
            name: "Google Chrome",
            path_env: "WYVERN_CHROME_PATH",
        },
        CatalogBrowser {
            id: "edge",
            name: "Microsoft Edge",
            path_env: "WYVERN_EDGE_PATH",
        },
        CatalogBrowser {
            id: "firefox",
            name: "Mozilla Firefox",
            path_env: "WYVERN_FIREFOX_PATH",
        },
        CatalogBrowser {
            id: "safari",
            name: "Safari",
            path_env: "WYVERN_SAFARI_PATH",
        },
        CatalogBrowser {
            id: "brave",
            name: "Brave",
            path_env: "WYVERN_BRAVE_PATH",
        },
        CatalogBrowser {
            id: "chromium",
            name: "Chromium",
            path_env: "WYVERN_CHROMIUM_PATH",
        },
        CatalogBrowser {
            id: "opera",
            name: "Opera",
            path_env: "WYVERN_OPERA_PATH",
        },
        CatalogBrowser {
            id: "vivaldi",
            name: "Vivaldi",
            path_env: "WYVERN_VIVALDI_PATH",
        },
    ]
}

/// Look up a catalog entry by id.
#[cfg_attr(not(test), allow(dead_code))]
pub fn find(id: &str) -> Option<&'static CatalogBrowser> {
    catalog().iter().find(|b| b.id == id)
}

/// Catalog entry for a [`BrowserId`] (c.15 `--viewer` named set).
pub fn for_browser_id(id: BrowserId) -> &'static CatalogBrowser {
    match id {
        BrowserId::Chrome => &catalog()[0],
        BrowserId::Edge => &catalog()[1],
        BrowserId::Firefox => &catalog()[2],
        BrowserId::Safari => &catalog()[3],
    }
}

/// Discover an executable for `browser` on this platform, if installed.
pub fn discover(browser: &CatalogBrowser) -> Option<PathBuf> {
    if let Ok(path) = std::env::var(browser.path_env) {
        let p = PathBuf::from(&path);
        // Explicit override: honor the path when present; do not fall through to
        // platform discovery when the override is set but missing (http-viewer-contract).
        return executable_exists(&p).then_some(p);
    }
    discover_platform(browser.id)
}

fn executable_exists(path: &Path) -> bool {
    path.is_file()
}

#[cfg(target_os = "macos")]
fn discover_platform(id: &str) -> Option<PathBuf> {
    let candidates: &[&str] = match id {
        "chrome" => &[
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome for Testing",
        ],
        "edge" => &["/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"],
        "firefox" => &["/Applications/Firefox.app/Contents/MacOS/firefox"],
        "safari" => &["/Applications/Safari.app/Contents/MacOS/Safari"],
        "brave" => &["/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"],
        "chromium" => &["/Applications/Chromium.app/Contents/MacOS/Chromium"],
        "opera" => &["/Applications/Opera.app/Contents/MacOS/Opera"],
        "vivaldi" => &["/Applications/Vivaldi.app/Contents/MacOS/Vivaldi"],
        _ => &[],
    };
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|p| executable_exists(p))
}

#[cfg(target_os = "windows")]
fn discover_platform(id: &str) -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let program_files = std::env::var_os("PROGRAMFILES").map(PathBuf::from);
    let program_files_x86 = std::env::var_os("PROGRAMFILES(X86)").map(PathBuf::from);

    let mut candidates: Vec<PathBuf> = Vec::new();
    match id {
        "chrome" => {
            for root in [&local, &program_files, &program_files_x86]
                .into_iter()
                .flatten()
            {
                candidates.push(
                    root.join("Google")
                        .join("Chrome")
                        .join("Application")
                        .join("chrome.exe"),
                );
            }
        }
        "edge" => {
            for root in [&program_files, &program_files_x86, &local]
                .into_iter()
                .flatten()
            {
                candidates.push(
                    root.join("Microsoft")
                        .join("Edge")
                        .join("Application")
                        .join("msedge.exe"),
                );
            }
        }
        "firefox" => {
            for root in [&program_files, &program_files_x86].into_iter().flatten() {
                candidates.push(root.join("Mozilla Firefox").join("firefox.exe"));
            }
        }
        "brave" => {
            for root in [&local, &program_files].into_iter().flatten() {
                candidates.push(
                    root.join("BraveSoftware")
                        .join("Brave-Browser")
                        .join("Application")
                        .join("brave.exe"),
                );
            }
        }
        "chromium" => {
            if let Some(root) = &local {
                candidates.push(root.join("Chromium").join("Application").join("chrome.exe"));
            }
        }
        "opera" => {
            if let Some(root) = &local {
                candidates.push(root.join("Programs").join("Opera").join("opera.exe"));
            }
        }
        "vivaldi" => {
            if let Some(root) = &local {
                candidates.push(root.join("Vivaldi").join("Application").join("vivaldi.exe"));
            }
        }
        "safari" => {}
        _ => {}
    }
    candidates.into_iter().find(|p| executable_exists(p))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn discover_platform(id: &str) -> Option<PathBuf> {
    let names: &[&str] = match id {
        "chrome" => &[
            "google-chrome",
            "google-chrome-stable",
            "chromium-browser",
            "chromium",
        ],
        "edge" => &["microsoft-edge", "microsoft-edge-stable", "msedge"],
        "firefox" => &["firefox"],
        "brave" => &["brave-browser", "brave"],
        "chromium" => &["chromium", "chromium-browser"],
        "opera" => &["opera"],
        "vivaldi" => &["vivaldi"],
        "safari" => &[],
        _ => &[],
    };
    for name in names {
        if let Some(path) = which(name) {
            return Some(path);
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if executable_exists(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_viewer_enum_ids() {
        for id in ["chrome", "safari", "edge", "firefox"] {
            assert!(find(id).is_some(), "missing {id}");
        }
    }

    #[test]
    fn for_browser_id_matches_wire_name() {
        assert_eq!(for_browser_id(BrowserId::Chrome).id, "chrome");
        assert_eq!(for_browser_id(BrowserId::Safari).id, "safari");
    }
}
