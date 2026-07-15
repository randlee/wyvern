(function () {
  "use strict";

  const dialogEl = document.getElementById("dialog");
  const titleEl = document.getElementById("title");
  const messageEl = document.getElementById("message");
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

  function renderButtons(list, defaultIndex) {
    buttonsEl.innerHTML = "";
    list.forEach(function (btn, index) {
      const el = document.createElement("button");
      el.type = "button";
      el.textContent = btn.label;
      el.dataset.testid = "btn-" + btn.id;
      el.setAttribute("data-testid", "btn-" + btn.id);
      el.id = "btn-" + btn.id;
      if (index === (defaultIndex == null ? list.length - 1 : defaultIndex)) {
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
      if (payload.type !== "message") {
        throw new Error("expected message dialog, got " + payload.type);
      }
      document.title = payload.title || "Wyvern Message";
      titleEl.textContent = payload.title || "";
      messageEl.textContent = payload.message || "";
      const list = Array.isArray(payload.button_list) ? payload.button_list : [];
      renderButtons(list, payload.default_button);
      dialogEl.hidden = false;
      WyvernApi.applyDialogLayout(payload);
      window.addEventListener("beforeunload", onBeforeUnload);
    })
    .catch(showError);
})();
