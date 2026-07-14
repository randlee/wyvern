(function () {
  "use strict";

  var titlebarEl = document.getElementById("titlebar");
  var titlebarTitleEl = document.getElementById("titlebar-title");
  var titlebarControlsEl = document.getElementById("titlebar-controls");
  var dialogEl = document.getElementById("dialog");
  var titleEl = document.getElementById("title");
  var statusEl = document.getElementById("status");
  var buttonsEl = document.getElementById("buttons");
  var errorEl = document.getElementById("error");

  var submitted = false;

  function showError(err) {
    errorEl.hidden = false;
    errorEl.textContent = String(err && err.message ? err.message : err);
  }

  async function submit(buttonId) {
    if (submitted) return;
    submitted = true;
    try {
      await WyvernApi.postResult({ button: buttonId });
    } catch (err) {
      submitted = false;
      showError(err);
    }
  }

  function onBeforeUnload() {
    if (submitted) return;
    WyvernApi.postResultBeacon({ button: "dismissed" });
  }

  /**
   * Returns true when running on Windows or Linux (not macOS).
   *
   * On macOS the OS provides native window chrome; on Win/Linux the browser
   * tab has no native close/minimize so we render HTML controls.
   */
  function needsHtmlChrome() {
    var ua = (navigator.userAgent || "").toLowerCase();
    return !(ua.indexOf("mac os x") !== -1 || ua.indexOf("macintosh") !== -1);
  }

  /**
   * Render HTML close + minimize buttons for Win/Linux.
   *
   * Close sends `dismissed`; minimize is cosmetic only (no wyvern-window IPC
   * in the HTTP host — it posts no result and leaves the dialog open).
   */
  function renderTitlebarControls(title) {
    if (!needsHtmlChrome()) return;

    titlebarTitleEl.textContent = title || "";
    titlebarEl.hidden = false;

    var minimizeBtn = document.createElement("button");
    minimizeBtn.className = "titlebar-btn minimize";
    minimizeBtn.setAttribute("aria-label", "Minimize");
    minimizeBtn.setAttribute("data-testid", "btn-minimize");
    minimizeBtn.textContent = "\u2013"; // –
    minimizeBtn.addEventListener("click", function () {
      // Cosmetic only in HTTP mode; no host IPC for minimize.
    });

    var closeBtn = document.createElement("button");
    closeBtn.className = "titlebar-btn close";
    closeBtn.setAttribute("aria-label", "Close");
    closeBtn.setAttribute("data-testid", "btn-close-chrome");
    closeBtn.textContent = "\u00d7"; // ×
    closeBtn.addEventListener("click", function () {
      submit("dismissed");
    });

    titlebarControlsEl.appendChild(minimizeBtn);
    titlebarControlsEl.appendChild(closeBtn);
  }

  WyvernApi.fetchDialog()
    .then(function (payload) {
      if (payload.type !== "chrome") {
        throw new Error("expected chrome dialog, got " + payload.type);
      }

      var title = payload.title || "Wyvern";
      document.title = title;
      titleEl.textContent = title;

      if (payload.status) {
        statusEl.textContent = payload.status;
        statusEl.hidden = false;
      }

      renderTitlebarControls(title);

      // Chrome has a single hardcoded OK button — no ButtonsPreset on the wire.
      var okBtn = document.createElement("button");
      okBtn.type = "button";
      okBtn.textContent = "OK";
      okBtn.classList.add("primary");
      okBtn.id = "btn-ok";
      okBtn.setAttribute("data-testid", "btn-ok");
      okBtn.addEventListener("click", function () {
        submit("ok");
      });
      buttonsEl.appendChild(okBtn);

      dialogEl.hidden = false;
      window.addEventListener("beforeunload", onBeforeUnload);
    })
    .catch(showError);
})();
