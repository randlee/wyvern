/**
 * Review-mode finish — POST /api/report/finish once per Approve/Cancel click.
 *
 * Reads authoritative panels from `#manifest-data`. Disables both buttons after
 * the first submit. Does not retry on 409 / 5xx (session is already terminal).
 */
(function () {
  "use strict";

  var submitted = false;

  function readManifestPanels() {
    var el = document.getElementById("manifest-data");
    if (!el) {
      return [];
    }
    try {
      var data = JSON.parse(el.textContent || "{}");
      return Array.isArray(data.panels) ? data.panels : [];
    } catch (err) {
      return [];
    }
  }

  function disableButtons() {
    var buttons = document.querySelectorAll("[data-report-approve], [data-report-cancel]");
    for (var i = 0; i < buttons.length; i += 1) {
      buttons[i].disabled = true;
    }
  }

  function commentsValue() {
    var el = document.getElementById("review-comments");
    return el && typeof el.value === "string" ? el.value : "";
  }

  function finish(approved) {
    if (submitted) {
      return;
    }
    submitted = true;
    disableButtons();
    var body = JSON.stringify({
      approved: approved,
      comments: commentsValue(),
      panels: readManifestPanels(),
    });
    fetch("/api/report/finish", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: body,
    }).catch(function () {
      // No automatic retry: a 409/5xx means the session already completed or
      // the host is gone; a second POST would race SessionState::complete.
    });
  }

  function onReady() {
    var approve = document.querySelector("[data-report-approve]");
    var cancel = document.querySelector("[data-report-cancel]");
    if (approve) {
      approve.addEventListener("click", function () {
        finish(true);
      });
    }
    if (cancel) {
      cancel.addEventListener("click", function () {
        finish(false);
      });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", onReady);
  } else {
    onReady();
  }
})();
