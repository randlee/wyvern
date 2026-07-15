//! One-shot dialog session state and result channel.
//!
//! [`SessionState`] is cheaply cloned into every axum handler. Handlers need
//! shared access to the validated [`Command`] and a one-shot completion sender,
//! so the inner state lives behind `Arc<tokio::sync::Mutex<_>>`.
//!
//! - `Arc` clones across concurrent GET `/api/dialog`, POST `/api/result`, and
//!   picker routes without moving ownership of the command or channel.
//! - `Mutex` ensures `complete` takes the result-submit capability exactly once
//!   while serializing that take with reads of `command`. Splitting immutable
//!   command state from a completion flag would work but adds pieces for a
//!   one-shot session; the mutex keeps command and completion co-located.
//! - `tokio::sync::Mutex` fits async handlers (picker/result paths already await
//!   in this context and may grow awaits under the lock later).
//!
//! ## Lifecycle capabilities (RBP-F001 / RBP-F002)
//!
//! Session phases are gated by capability tokens rather than ad-hoc `Option`
//! presence checks alone:
//! - [`BoundOrigin`] — issued once after TCP bind; navigate URL builders require it.
//! - [`ResultSubmitToken`] — issued at session creation; `complete` consumes it.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{oneshot, Mutex, OwnedSemaphorePermit, Semaphore};
use wyvern_schema::{
    ButtonLabel, ChromeResult, Command, CommandResult, InputResult, MarkdownResult, MessageResult,
    WizardPageDescriptor, WizardResult, WizardStackEntry, WizardTerminalButton,
};
use wyvern_wizard::{NavigateOutcome, WizardError, WizardSession, WizardSnapshot};

use crate::picker::MockPickerConfig;

/// Max time a native `rfd` picker may block a `spawn_blocking` worker.
pub(crate) const PICKER_TIMEOUT: Duration = Duration::from_secs(300);

/// Capability proving the HTTP listener bound and the public origin is known.
///
/// Issued once via [`SessionState::set_public_origin`] after bind (RBP-F001).
#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundOrigin(String);

impl BoundOrigin {
    fn as_str(&self) -> &str {
        &self.0
    }
}

/// One-shot capability to submit the dialog result (RBP-F002).
///
/// Created at session construction; taken exactly once by [`SessionState::complete`].
struct ResultSubmitToken(oneshot::Sender<CommandResult>);

impl ResultSubmitToken {
    fn submit(self, result: CommandResult) {
        let _ = self.0.send(result);
    }
}

/// Shared state for the active one-shot dialog.
#[derive(Clone)]
pub(crate) struct SessionState {
    inner: Arc<Mutex<SessionInner>>,
    /// Caps concurrent native pickers (one dialog → one picker at a time).
    picker_slots: Arc<Semaphore>,
    /// Optional in-process picker mock (tests); env mock remains for CLI/e2e.
    mock_picker: Option<MockPickerConfig>,
}

struct SessionInner {
    /// Shared command — `Arc` so `/api/dialog` and picker routes avoid deep clones.
    command: Arc<Command>,
    /// Present when the active command is [`Command::Wizard`].
    wizard: Option<WizardSession>,
    /// Bind-phase capability; `None` until [`SessionState::set_public_origin`].
    bound_origin: Option<BoundOrigin>,
    /// Complete-phase capability; `None` after the one-shot result is submitted.
    result_token: Option<ResultSubmitToken>,
}

impl SessionState {
    /// Create a session that will deliver the result once via `result_tx`.
    pub(crate) fn new(
        command: Command,
        result_tx: oneshot::Sender<wyvern_schema::CommandResult>,
        mock_picker: Option<MockPickerConfig>,
    ) -> Self {
        let wizard = match &command {
            Command::Wizard(wizard_cmd) => Some(WizardSession::new(wizard_cmd)),
            _ => None,
        };
        Self {
            inner: Arc::new(Mutex::new(SessionInner {
                command: Arc::new(command),
                wizard,
                bound_origin: None,
                result_token: Some(ResultSubmitToken(result_tx)),
            })),
            // Serialize picker spawn_blocking so repeated POSTs cannot exhaust
            // the blocking pool (RSH-006).
            picker_slots: Arc::new(Semaphore::new(1)),
            mock_picker,
        }
    }

    /// Issue the bind-phase [`BoundOrigin`] capability used to build absolute URLs.
    pub(crate) async fn set_public_origin(&self, origin: String) {
        self.inner.lock().await.bound_origin = Some(BoundOrigin(origin));
    }

    /// Shared handle to the active command for `/api/dialog` (cheap `Arc` clone).
    pub(crate) async fn command(&self) -> Arc<Command> {
        Arc::clone(&self.inner.lock().await.command)
    }

    /// Snapshot of the wizard session, if this is a wizard dialog.
    ///
    /// Builds the snapshot under the mutex without cloning [`WizardSession`]
    /// (RBP-F005) — `snapshot()` already copies only the wire-facing fields.
    pub(crate) async fn wizard_snapshot(&self) -> Result<WizardSnapshot, WizardError> {
        let guard = self.inner.lock().await;
        Ok(guard
            .wizard
            .as_ref()
            .ok_or(WizardError::SessionNotInitialized)?
            .snapshot())
    }

    /// Run `navigate_next` on the live wizard session.
    pub(crate) async fn wizard_navigate_next(
        &self,
        data: serde_json::Value,
        next: WizardPageDescriptor,
    ) -> Result<(NavigateOutcome, String), WizardError> {
        let mut guard = self.inner.lock().await;
        require_open_result_token(&guard)?;
        let origin = require_bound_origin(&guard)?.to_string();
        let session = guard
            .wizard
            .as_mut()
            .ok_or(WizardError::SessionNotInitialized)?;
        let outcome = session.navigate_next(data, next)?;
        let url = wizard_page_url(&origin, outcome.page.html.as_str());
        Ok((outcome, url))
    }

    /// Run `navigate_back` on the live wizard session.
    pub(crate) async fn wizard_navigate_back(
        &self,
        data: serde_json::Value,
    ) -> Result<(NavigateOutcome, String), WizardError> {
        let mut guard = self.inner.lock().await;
        require_open_result_token(&guard)?;
        let origin = require_bound_origin(&guard)?.to_string();
        let session = guard
            .wizard
            .as_mut()
            .ok_or(WizardError::SessionNotInitialized)?;
        let outcome = session.navigate_back(data)?;
        let url = wizard_page_url(&origin, outcome.page.html.as_str());
        Ok((outcome, url))
    }

    /// Run `finish` and complete the one-shot session with the derived result.
    ///
    /// Holds the session mutex through result derivation and
    /// [`ResultSubmitToken`] consumption so a concurrent navigate cannot mutate
    /// wizard state after the terminal result is fixed but before the session is
    /// marked complete (QA-001). The oneshot is sent only after the lock is
    /// released.
    ///
    /// Returns `Ok(None)` when a result was already submitted (HTTP 409).
    pub(crate) async fn wizard_finish(
        &self,
        button: WizardTerminalButton,
        data: serde_json::Value,
        stack: Vec<WizardStackEntry>,
    ) -> Result<Option<WizardResult>, WizardError> {
        let prepared = {
            let mut guard = self.inner.lock().await;
            let session = guard
                .wizard
                .as_ref()
                .ok_or(WizardError::SessionNotInitialized)?;
            let result = session.finish(button, data, stack)?;
            guard.result_token.take().map(|token| (result, token))
        };
        match prepared {
            Some((result, token)) => {
                token.submit(CommandResult::Wizard(result.clone()));
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// REQ-0097 fallback when the viewer exits or the session times out.
    ///
    /// Wizard sessions derive dismissed stdout via
    /// [`WizardSession::finish`]`(Dismissed, page_data, full_visited_stack)` —
    /// same d.2 algorithm as an explicit finish POST (stdout `data` is `{}`).
    pub(crate) async fn dismissed_on_exit_or_timeout(&self) -> CommandResult {
        let guard = self.inner.lock().await;
        match (guard.command.as_ref(), guard.wizard.as_ref()) {
            (Command::Wizard(_), Some(session)) => {
                let snap = session.snapshot();
                let page_id = snap.page.id.as_str().to_string();
                let mut derived = snap.stack;
                derived.push(WizardStackEntry {
                    page: snap.page,
                    data: snap.page_data.clone(),
                });
                match session.finish(
                    WizardTerminalButton::Dismissed,
                    snap.page_data,
                    derived.clone(),
                ) {
                    Ok(result) => CommandResult::Wizard(result),
                    Err(err) => {
                        // RBP-F002: keep the manually derived full visited stack rather
                        // than degrading to WizardResult::dismissed() (empty stack).
                        tracing::error!(
                            error = %err,
                            page_id = %page_id,
                            stack_len = derived.len(),
                            "wizard dismissed fallback finish failed; returning derived stack"
                        );
                        CommandResult::Wizard(WizardResult {
                            button: ButtonLabel::dismissed(),
                            data: serde_json::json!({}),
                            stack: derived,
                        })
                    }
                }
            }
            (command, _) => dismissed_for_command(command),
        }
    }

    /// In-process mock picker config, if any.
    pub(crate) fn mock_picker(&self) -> Option<&MockPickerConfig> {
        self.mock_picker.as_ref()
    }

    /// Acquire the single picker permit for this session.
    ///
    /// Returns an [`OwnedSemaphorePermit`] held by the async handler. On HTTP
    /// timeout the handler drops the permit so a subsequent picker can proceed
    /// (RSH-002); the detached `spawn_blocking` task may still finish later.
    pub(crate) async fn acquire_picker_slot(
        &self,
    ) -> Result<OwnedSemaphorePermit, HostSessionClosed> {
        Arc::clone(&self.picker_slots)
            .acquire_owned()
            .await
            .map_err(|_| HostSessionClosed)
    }

    /// Deliver a validated result by consuming the [`ResultSubmitToken`] (idempotent).
    pub(crate) async fn complete(&self, result: wyvern_schema::CommandResult) -> bool {
        let mut guard = self.inner.lock().await;
        if let Some(token) = guard.result_token.take() {
            token.submit(result);
            true
        } else {
            false
        }
    }
}

fn require_open_result_token(guard: &SessionInner) -> Result<(), WizardError> {
    if guard.result_token.is_none() {
        Err(WizardError::ResultAlreadySubmitted)
    } else {
        Ok(())
    }
}

fn require_bound_origin(guard: &SessionInner) -> Result<&str, WizardError> {
    guard
        .bound_origin
        .as_ref()
        .map(BoundOrigin::as_str)
        .ok_or(WizardError::PublicOriginNotSet)
}

fn wizard_page_url(origin: &str, html: &str) -> String {
    let html = html.trim_start_matches('/');
    format!("{origin}/wizard/{html}")
}

/// REQ-0097 dismissed shape for non-wizard command types (blocking dialogs).
fn dismissed_for_command(command: &Command) -> CommandResult {
    match command {
        Command::Message { .. } => CommandResult::Message(MessageResult {
            button: ButtonLabel::dismissed(),
        }),
        Command::Input { .. } => CommandResult::Input(InputResult {
            button: ButtonLabel::dismissed(),
            input: None,
        }),
        Command::Markdown { .. } => CommandResult::Markdown(MarkdownResult {
            button: ButtonLabel::dismissed(),
        }),
        Command::Chrome { .. } => CommandResult::Chrome(ChromeResult {
            button: ButtonLabel::dismissed(),
        }),
        Command::Question { questions_raw, .. } => CommandResult::Question(
            wyvern_schema::QuestionResult::dismissed(questions_raw.clone()),
        ),
        Command::Wizard(_) => CommandResult::Wizard(WizardResult::dismissed()),
    }
}

/// Session was dropped while waiting for a picker slot (should not happen in-process).
#[derive(Debug)]
pub(crate) struct HostSessionClosed;

impl std::fmt::Display for HostSessionClosed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("dialog session closed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyvern_schema::{
        WizardCommand, WizardPageDescriptor, WizardPageHtml, WizardPageId, WizardPageTitle,
    };

    fn wizard_cmd() -> Command {
        Command::Wizard(WizardCommand {
            page: WizardPageDescriptor {
                id: WizardPageId::new("a"),
                title: WizardPageTitle::new("a"),
                html: WizardPageHtml::new("pages/a.html"),
                layout: None,
            },
            config: serde_json::json!({}),
            width: None,
            height: None,
        })
    }

    #[tokio::test]
    async fn navigate_after_result_submitted_returns_conflict_error() {
        let (tx, _rx) = oneshot::channel();
        let session = SessionState::new(wizard_cmd(), tx, None);
        session.set_public_origin("http://127.0.0.1:9".into()).await;

        assert!(
            session
                .complete(CommandResult::Wizard(WizardResult::dismissed()))
                .await
        );

        let err = session
            .wizard_navigate_next(
                serde_json::json!({}),
                WizardPageDescriptor {
                    id: WizardPageId::new("b"),
                    title: WizardPageTitle::new("b"),
                    html: WizardPageHtml::new("pages/b.html"),
                    layout: None,
                },
            )
            .await
            .expect_err("should reject post-complete navigate");
        assert_eq!(err, WizardError::ResultAlreadySubmitted);

        let err = session
            .wizard_navigate_back(serde_json::json!({}))
            .await
            .expect_err("back should also reject");
        assert_eq!(err, WizardError::ResultAlreadySubmitted);
    }

    #[tokio::test]
    async fn navigate_without_bound_origin_fails() {
        let (tx, _rx) = oneshot::channel();
        let session = SessionState::new(wizard_cmd(), tx, None);
        let err = session
            .wizard_navigate_back(serde_json::json!({}))
            .await
            .expect_err("origin required");
        assert_eq!(err, WizardError::PublicOriginNotSet);
    }

    /// Finish with a stack matching the initial page only; navigate tries to leave
    /// that page. With an atomic finish+complete path, the outcomes are mutually
    /// exclusive: either finish wins (`ResultAlreadySubmitted` on navigate) or
    /// navigate wins first (`StackMismatch` on finish). Both succeeding would mean
    /// navigate ran in the gap between result derivation and token consumption.
    #[tokio::test]
    async fn concurrent_navigate_and_finish_are_mutually_exclusive() {
        let page_a = WizardPageDescriptor {
            id: WizardPageId::new("a"),
            title: WizardPageTitle::new("a"),
            html: WizardPageHtml::new("pages/a.html"),
            layout: None,
        };
        let page_b = WizardPageDescriptor {
            id: WizardPageId::new("b"),
            title: WizardPageTitle::new("b"),
            html: WizardPageHtml::new("pages/b.html"),
            layout: None,
        };
        let finish_data = serde_json::json!({});
        let finish_stack = vec![WizardStackEntry {
            page: page_a,
            data: finish_data.clone(),
        }];

        for _ in 0..200 {
            let (tx, _rx) = oneshot::channel();
            let session = SessionState::new(wizard_cmd(), tx, None);
            session.set_public_origin("http://127.0.0.1:9".into()).await;

            let session_finish = session.clone();
            let session_nav = session.clone();
            let stack = finish_stack.clone();
            let data = finish_data.clone();
            let next = page_b.clone();

            let (finish_res, nav_res) = tokio::join!(
                async move {
                    session_finish
                        .wizard_finish(WizardTerminalButton::Finish, data, stack)
                        .await
                },
                async move {
                    session_nav
                        .wizard_navigate_next(serde_json::json!({}), next)
                        .await
                }
            );

            match (finish_res, nav_res) {
                (Ok(Some(_)), Err(WizardError::ResultAlreadySubmitted)) => {}
                (Err(WizardError::StackMismatch { .. }), Ok(_)) => {}
                (Ok(Some(_)), Ok(_)) => {
                    panic!(
                        "navigate must not succeed alongside successful finish \
                         (QA-001 race between derive and token take)"
                    );
                }
                (finish, nav) => {
                    panic!("unexpected concurrent outcomes: finish={finish:?} nav={nav:?}");
                }
            }
        }
    }
}
