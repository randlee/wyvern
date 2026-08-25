/**
 * Vanilla-chrome two-step skeleton.
 * Page JS collects opaque finish data only — no disk I/O (REQ-0124 / REQ-0125).
 */
(function (global) {
  "use strict";

  function emptySelection() {
    return { label: "", file_path: "" };
  }

  function selectionFromPrior() {
    var stack = (global.wyvern && Array.isArray(global.wyvern.stack)
      ? global.wyvern.stack
      : []) || [];
    for (var i = stack.length - 1; i >= 0; i -= 1) {
      var data = stack[i] && stack[i].data;
      if (data && (data.label || data.file_path)) {
        return {
          label: data.label ? String(data.label) : "",
          file_path: data.file_path ? String(data.file_path) : ""
        };
      }
    }
    var pageData = global.wyvern && global.wyvern.page_data;
    if (pageData && (pageData.label || pageData.file_path)) {
      return {
        label: pageData.label ? String(pageData.label) : "",
        file_path: pageData.file_path ? String(pageData.file_path) : ""
      };
    }
    return emptySelection();
  }

  function pathFromFileInput(el) {
    if (!el || !el.files || !el.files[0]) {
      return "";
    }
    var file = el.files[0];
    return file.path || file.name || "";
  }

  function collectFormSelection() {
    var prior = selectionFromPrior();
    var labelEl = document.querySelector("[data-testid='field-label']");
    var pathEl = document.querySelector("[data-testid='field-file-path']");
    var fileEl = document.querySelector("[data-testid='field-file']");
    var filePath = pathEl ? String(pathEl.value || "") : prior.file_path;
    var fromPicker = pathFromFileInput(fileEl);
    if (fromPicker) {
      filePath = fromPicker;
    }
    return {
      label: labelEl ? String(labelEl.value || "") : prior.label,
      file_path: filePath
    };
  }

  function fillForm() {
    var labelEl = document.querySelector("[data-testid='field-label']");
    if (!labelEl) {
      return;
    }
    var prior = selectionFromPrior();
    labelEl.value = prior.label;
    var pathEl = document.querySelector("[data-testid='field-file-path']");
    if (pathEl) {
      pathEl.value = prior.file_path;
    }
    var fileEl = document.querySelector("[data-testid='field-file']");
    if (fileEl && pathEl) {
      fileEl.addEventListener("change", function () {
        var picked = pathFromFileInput(fileEl);
        if (picked) {
          pathEl.value = picked;
        }
      });
    }
  }

  function fillReview() {
    var labelEl = document.querySelector("[data-testid='review-label']");
    if (!labelEl) {
      return;
    }
    var data = selectionFromPrior();
    labelEl.textContent = data.label || "—";
    var pathEl = document.querySelector("[data-testid='review-file-path']");
    if (pathEl) {
      pathEl.textContent = data.file_path || "—";
    }
  }

  global.collectCurrentPageData = function () {
    if (document.querySelector("[data-testid='field-label']")) {
      return collectFormSelection();
    }
    return selectionFromPrior();
  };

  if (document.querySelector("[data-testid='field-label']")) {
    global.wizardNextDescriptor = {
      id: "two",
      title: "Review",
      html: "pages/two.html"
    };
  }

  function boot() {
    if (typeof global.wyvernWizardState === "function") {
      return global.wyvernWizardState().then(function () {
        fillForm();
        fillReview();
      });
    }
    fillForm();
    fillReview();
    return Promise.resolve();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      boot();
    });
  } else {
    boot();
  }
})(typeof window !== "undefined" ? window : globalThis);
