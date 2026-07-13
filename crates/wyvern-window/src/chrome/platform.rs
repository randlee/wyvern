//! Platform-specific chrome layout for the HTML title bar (ADR-0010 / ADR-0010a).

/// Dialog / chrome command kinds that drive [`PlatformChrome`] flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandKind {
    /// Non-modal status chrome (`chrome` command).
    Chrome,
    /// Modal message dialog.
    Message,
    /// Modal input dialog.
    Input,
    /// Modal markdown viewer.
    Markdown,
    /// Modal question dialog.
    Question,
}

/// Title-bar layout flags injected into HTML templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformChrome {
    /// macOS only: reserve 72px left padding for traffic lights (ADR-0010).
    pub macos_safe_zone: bool,
    /// Win/Linux non-modal: show HTML minimize button.
    pub show_minimize: bool,
    /// Win/Linux: render HTML close/minimize block in title bar.
    pub show_window_controls: bool,
}

/// Build [`PlatformChrome`] for the given command kind on the current OS.
pub fn platform_chrome_for(command: CommandKind) -> PlatformChrome {
    #[cfg(target_os = "macos")]
    {
        let _ = command;
        PlatformChrome {
            macos_safe_zone: true,
            show_minimize: false,
            show_window_controls: false,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let modal = matches!(
            command,
            CommandKind::Message
                | CommandKind::Input
                | CommandKind::Markdown
                | CommandKind::Question
        );
        PlatformChrome {
            macos_safe_zone: false,
            show_minimize: !modal,
            show_window_controls: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_chrome_macos_hides_html_controls() {
        #[cfg(target_os = "macos")]
        {
            let chrome = platform_chrome_for(CommandKind::Chrome);
            assert!(chrome.macos_safe_zone);
            assert!(!chrome.show_minimize);
            assert!(!chrome.show_window_controls);

            let modal = platform_chrome_for(CommandKind::Message);
            assert!(modal.macos_safe_zone);
            assert!(!modal.show_window_controls);
        }
    }

    #[test]
    fn platform_chrome_non_macos_modal_omits_minimize() {
        #[cfg(not(target_os = "macos"))]
        {
            let chrome = platform_chrome_for(CommandKind::Chrome);
            assert!(!chrome.macos_safe_zone);
            assert!(chrome.show_minimize);
            assert!(chrome.show_window_controls);

            let modal = platform_chrome_for(CommandKind::Message);
            assert!(!modal.macos_safe_zone);
            assert!(!modal.show_minimize);
            assert!(modal.show_window_controls);
        }
    }
}
