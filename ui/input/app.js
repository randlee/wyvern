(function () {
  "use strict";

  const dialogEl = document.getElementById("dialog");
  const titleEl = document.getElementById("title");
  const messageEl = document.getElementById("message");
  const fieldWrap = document.getElementById("field-wrap");
  const pickedEl = document.getElementById("picked");
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

  function renderField() {
    fieldWrap.innerHTML = "";
    fieldEl = null;
    if (isPickerMode()) {
      fieldWrap.hidden = true;
      return;
    }
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
    fieldEl.addEventListener("keydown", function (ev) {
      if (ev.key === "Enter" && !payload.multiline && !ev.shiftKey) {
        ev.preventDefault();
        submit("ok");
      }
    });
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

    if (isPickerMode()) {
      try {
        const pickerBody = {};
        if (payload.mode === "file") {
          if (Array.isArray(payload.filter)) pickerBody.filter = payload.filter;
          if (payload.multiple) pickerBody.multiple = true;
          if (payload.start_path) pickerBody.start_path = payload.start_path;
          const picked = await WyvernApi.postPickerFile(pickerBody);
          if (!picked.ok || picked.cancelled) {
            return;
          }
          const paths = picked.paths || [];
          pickedEl.hidden = false;
          pickedEl.textContent = paths.join(", ");
          submitted = true;
          const input =
            payload.multiple || paths.length > 1 ? paths : paths[0] || "";
          await WyvernApi.postResult({ button: "ok", input: input });
        } else {
          if (payload.start_path) pickerBody.start_path = payload.start_path;
          const picked = await WyvernApi.postPickerFolder(pickerBody);
          if (!picked.ok || picked.cancelled) {
            return;
          }
          const paths = picked.paths || [];
          pickedEl.hidden = false;
          pickedEl.textContent = paths.join(", ");
          submitted = true;
          await WyvernApi.postResult({
            button: "ok",
            input: paths[0] || "",
          });
        }
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
        input: fieldEl ? fieldEl.value : "",
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
      document.title = data.title || "Wyvern Input";
      titleEl.textContent = data.title || "";
      messageEl.textContent = data.message || "";
      renderField();
      const list = Array.isArray(data.button_list) ? data.button_list : [];
      renderButtons(list);
      dialogEl.hidden = false;
      window.addEventListener("beforeunload", onBeforeUnload);
    })
    .catch(showError);
})();
