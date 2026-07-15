(function () {
  "use strict";

  const dialogEl = document.getElementById("dialog");
  const titleEl = document.getElementById("title");
  const messageEl = document.getElementById("message");
  const fieldWrap = document.getElementById("field-wrap");
  const buttonsEl = document.getElementById("buttons");
  const errorEl = document.getElementById("error");

  let submitted = false;
  let payload = null;
  let fieldEl = null;

  function showError(err) {
    errorEl.hidden = false;
    errorEl.textContent = String(err && err.message ? err.message : err);
  }

  function isPickerMode() {
    return payload && (payload.mode === "file" || payload.mode === "folder");
  }

  function inputValueForSubmit() {
    const raw = fieldEl ? fieldEl.value : "";
    if (payload.mode === "file" && payload.multiple) {
      return raw
        .split(/\n/)
        .map(function (s) {
          return s.trim();
        })
        .filter(Boolean);
    }
    return raw;
  }

  function bindEnterToSubmit(el) {
    el.addEventListener("keydown", function (ev) {
      if (ev.key === "Enter" && !payload.multiline && !ev.shiftKey) {
        ev.preventDefault();
        submit("ok");
      }
    });
  }

  function renderPickerField() {
    fieldWrap.hidden = false;
    const row = document.createElement("div");
    row.className = "path-row";

    fieldEl = document.createElement(
      payload.mode === "file" && payload.multiple ? "textarea" : "input",
    );
    if (fieldEl.tagName === "TEXTAREA") {
      fieldEl.rows = 2;
      fieldEl.className = "path-multi";
    } else {
      fieldEl.type = "text";
    }
    fieldEl.id = "input-field";
    fieldEl.setAttribute("data-testid", "input-field");
    fieldEl.value = payload.default || "";
    if (payload.placeholder) {
      fieldEl.placeholder = payload.placeholder;
    }
    row.appendChild(fieldEl);

    const browse = document.createElement("button");
    browse.type = "button";
    browse.className = "browse-btn";
    browse.setAttribute("data-testid", "btn-browse");
    browse.setAttribute(
      "aria-label",
      payload.mode === "folder" ? "Choose folder" : "Choose file",
    );
    browse.textContent = "…";
    browse.addEventListener("click", function () {
      openPicker();
    });
    row.appendChild(browse);

    fieldWrap.appendChild(row);
    fieldEl.focus();
    if (fieldEl.value) {
      fieldEl.select();
    }
    bindEnterToSubmit(fieldEl);
  }

  function renderTextField() {
    fieldWrap.hidden = false;
    if (payload.multiline) {
      fieldEl = document.createElement("textarea");
      fieldEl.className = "multi-line";
      fieldEl.value = payload.default || "";
    } else {
      fieldEl = document.createElement("input");
      fieldEl.type = payload.password ? "password" : "text";
      fieldEl.value = payload.default || "";
    }
    fieldEl.id = "input-field";
    fieldEl.setAttribute("data-testid", "input-field");
    if (payload.placeholder) {
      fieldEl.placeholder = payload.placeholder;
    }
    fieldWrap.appendChild(fieldEl);
    fieldEl.focus();
    if (typeof fieldEl.select === "function" && fieldEl.value) {
      fieldEl.select();
    }
    bindEnterToSubmit(fieldEl);
  }

  function renderField() {
    fieldWrap.innerHTML = "";
    fieldEl = null;
    if (isPickerMode()) {
      renderPickerField();
      return;
    }
    renderTextField();
  }

  async function openPicker() {
    if (!fieldEl) return;
    errorEl.hidden = true;
    try {
      const pickerBody = {};
      let picked;
      if (payload.mode === "file") {
        if (Array.isArray(payload.filter)) pickerBody.filter = payload.filter;
        if (payload.multiple) pickerBody.multiple = true;
        if (payload.start_path) pickerBody.start_path = payload.start_path;
        picked = await WyvernApi.postPickerFile(pickerBody);
      } else {
        if (payload.start_path) pickerBody.start_path = payload.start_path;
        picked = await WyvernApi.postPickerFolder(pickerBody);
      }
      if (!picked.ok || picked.cancelled) {
        return;
      }
      const paths = picked.paths || [];
      if (payload.mode === "file" && payload.multiple) {
        fieldEl.value = paths.join("\n");
      } else {
        fieldEl.value = paths[0] || "";
      }
      fieldEl.focus();
    } catch (err) {
      showError(err);
    }
  }

  async function submit(buttonId) {
    if (submitted) return;
    if (buttonId === "cancel" || buttonId === "dismissed") {
      submitted = true;
      try {
        await WyvernApi.postResult({ button: buttonId });
      } catch (err) {
        submitted = false;
        showError(err);
      }
      return;
    }

    submitted = true;
    try {
      await WyvernApi.postResult({
        button: buttonId,
        input: inputValueForSubmit(),
      });
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
    .then(function (data) {
      if (data.type !== "input") {
        throw new Error("expected input dialog, got " + data.type);
      }
      payload = data;
      if (data.mode === "file" || data.mode === "folder") {
        dialogEl.classList.add("dialog--fill");
      }
      document.title = data.title || "Wyvern Input";
      titleEl.textContent = data.title || "";
      messageEl.textContent = data.message || "";
      renderField();
      const list = Array.isArray(data.button_list) ? data.button_list : [];
      renderButtons(list);
      dialogEl.hidden = false;
      WyvernApi.applyDialogLayout(payload);
      window.addEventListener("beforeunload", onBeforeUnload);
    })
    .catch(showError);
})();
