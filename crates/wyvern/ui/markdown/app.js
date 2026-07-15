(function () {
  "use strict";

  const dialogEl = document.getElementById("dialog");
  const titleEl = document.getElementById("title");
  const statusEl = document.getElementById("status");
  const bodyEl = document.getElementById("markdown-body");
  const buttonsEl = document.getElementById("buttons");
  const errorEl = document.getElementById("error");

  let submitted = false;

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

  function renderButtons(list) {
    buttonsEl.innerHTML = "";
    list.forEach(function (btn, index) {
      const el = document.createElement("button");
      el.type = "button";
      el.textContent = btn.label;
      el.setAttribute("data-testid", "btn-" + btn.id);
      el.id = "btn-" + btn.id;
      if (index === 0) {
        el.classList.add("primary");
      }
      el.addEventListener("click", function () {
        submit(btn.id);
      });
      buttonsEl.appendChild(el);
    });
  }

  function onBeforeUnload() {
    if (submitted) return;
    WyvernApi.postResultBeacon({ button: "dismissed" });
  }

  WyvernApi.fetchDialog()
    .then(function (payload) {
      if (payload.type !== "markdown") {
        throw new Error("expected markdown dialog, got " + payload.type);
      }
      document.title = payload.title || "Wyvern Markdown";
      titleEl.textContent = payload.title || "";
      // content_html is server-sanitized (pulldown-cmark + ammonia).
      bodyEl.innerHTML = payload.content_html || "";
      if (payload.status) {
        statusEl.hidden = false;
        statusEl.textContent = payload.status;
      } else {
        statusEl.hidden = true;
        statusEl.textContent = "";
      }
      const list = Array.isArray(payload.button_list) ? payload.button_list : [];
      renderButtons(list);
      dialogEl.hidden = false;
      WyvernApi.applyDialogLayout(payload);
      window.addEventListener("beforeunload", onBeforeUnload);
    })
    .catch(showError);
})();
