/**
 * Shared wizard chrome — back / next / finish wiring via wyvern-api.js.
 *
 * Opt-in: `<script src="/shared/wizard-nav.js" data-wizard-chrome></script>`
 *
 * Normative contracts (sprint d.7):
 * - Terminal page root sets `data-wizard-terminal="true"` (attribute only).
 * - Back: `wyvernWizardBack()` with no arg, or page-supplied opaque blob.
 * - Next: `wyvernWizardNext(collectCurrentPageData(), nextDescriptor)`.
 * - Finish: full visited stack = `window.wyvern.stack` + `{ page, data }`.
 * - Empty / missing page data → `{}` (never `undefined` access in helpers).
 */
(function (global) {
  "use strict";

  function normalizeData(data) {
    if (data === undefined || data === null) {
      return {};
    }
    return data;
  }

  function showError(message) {
    var el = document.querySelector("[data-testid='wizard-error']");
    if (el) {
      el.hidden = false;
      el.textContent = message;
    } else if (typeof console !== "undefined" && console.error) {
      console.error(message);
    }
  }

  /**
   * Opaque current-page blob. Prefer page-author `collectCurrentPageData()`;
   * missing / null / undefined → `{}`.
   */
  function collectPageData() {
    if (typeof global.collectCurrentPageData === "function") {
      try {
        return normalizeData(global.collectCurrentPageData());
      } catch (err) {
        showError(String(err && err.message ? err.message : err));
        return {};
      }
    }
    return {};
  }

  /** Page-author next descriptor: function or plain object on `wizardNextDescriptor`. */
  function resolveNextDescriptor() {
    var supplied = global.wizardNextDescriptor;
    if (typeof supplied === "function") {
      return supplied();
    }
    if (supplied && typeof supplied === "object") {
      return supplied;
    }
    return null;
  }

  function isTerminalPage() {
    return !!document.querySelector("[data-wizard-terminal='true']");
  }

  /** First page ≈ cursor 0: empty prior `stack` (REQ-0024). */
  function isFirstPage() {
    var stack = (global.wyvern && Array.isArray(global.wyvern.stack)
      ? global.wyvern.stack
      : []) || [];
    return stack.length === 0;
  }

  function buildFinishStack(data) {
    var prior = (
      global.wyvern && Array.isArray(global.wyvern.stack) ? global.wyvern.stack : []
    ).slice();
    prior.push({
      page: global.wyvern ? global.wyvern.page : null,
      data: normalizeData(data),
    });
    return prior;
  }

  function findNavRoot() {
    return document.querySelector("[data-wizard-nav]") || document;
  }

  function findBackButton(root) {
    return (
      root.querySelector("[data-wizard-back]") ||
      root.querySelector("[data-testid='wizard-back']")
    );
  }

  function findNextButton(root) {
    return (
      root.querySelector("[data-wizard-next]") ||
      root.querySelector("[data-testid='wizard-next']")
    );
  }

  function applyChromeState(root) {
    var back = findBackButton(root);
    var next = findNextButton(root);

    if (back) {
      if (isFirstPage()) {
        back.hidden = true;
        back.disabled = true;
        back.setAttribute("aria-hidden", "true");
      } else {
        back.hidden = false;
        back.disabled = false;
        back.removeAttribute("aria-hidden");
      }
    }

    if (next && isTerminalPage()) {
      next.textContent = "Finish";
      next.setAttribute("data-wizard-action", "finish");
    } else if (next) {
      if (!next.getAttribute("data-wizard-next-label")) {
        next.setAttribute("data-wizard-next-label", next.textContent || "Next");
      }
      next.textContent = next.getAttribute("data-wizard-next-label") || "Next";
      next.setAttribute("data-wizard-action", "next");
    }
  }

  async function handleBack() {
    if (typeof global.wyvernWizardBack !== "function") {
      throw new Error("wizard-nav: wyvernWizardBack is not available");
    }
    // Page-supplied opaque blob, else no-arg (meaningful-payload preserve).
    if (typeof global.wizardBackData === "function") {
      await global.wyvernWizardBack(normalizeData(global.wizardBackData()));
      return;
    }
    if (global.wizardBackData !== undefined) {
      await global.wyvernWizardBack(normalizeData(global.wizardBackData));
      return;
    }
    await global.wyvernWizardBack();
  }

  async function handleNextOrFinish() {
    var data = collectPageData();
    if (isTerminalPage()) {
      if (typeof global.wyvernWizardFinish !== "function") {
        throw new Error("wizard-nav: wyvernWizardFinish is not available");
      }
      await global.wyvernWizardFinish({
        button: "finish",
        data: data,
        stack: buildFinishStack(data),
      });
      return;
    }
    if (typeof global.wyvernWizardNext !== "function") {
      throw new Error("wizard-nav: wyvernWizardNext is not available");
    }
    var next = resolveNextDescriptor();
    if (!next) {
      throw new Error(
        "wizard-nav: define window.wizardNextDescriptor for non-terminal Next"
      );
    }
    await global.wyvernWizardNext(data, next);
  }

  function wireButtons(root) {
    var back = findBackButton(root);
    var next = findNextButton(root);

    if (back && !back.__wyvernWizardNavWired) {
      back.__wyvernWizardNavWired = true;
      back.addEventListener("click", function () {
        handleBack().catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }

    if (next && !next.__wyvernWizardNavWired) {
      next.__wyvernWizardNavWired = true;
      next.addEventListener("click", function () {
        handleNextOrFinish().catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }
  }

  async function ensureState() {
    if (typeof global.wyvernWizardState === "function") {
      await global.wyvernWizardState();
    }
    if (!global.wyvern) {
      global.wyvern = { config: {}, page: null, page_data: {}, stack: [] };
    }
    if (!Array.isArray(global.wyvern.stack)) {
      global.wyvern.stack = [];
    }
    if (global.wyvern.page_data === undefined || global.wyvern.page_data === null) {
      global.wyvern.page_data = {};
    }
  }

  async function boot() {
    try {
      await ensureState();
      var root = findNavRoot();
      applyChromeState(root);
      wireButtons(root);
    } catch (err) {
      showError(String(err && err.message ? err.message : err));
    }
  }

  function scriptOptedIn() {
    var script = document.currentScript;
    if (script && script.hasAttribute("data-wizard-chrome")) {
      return true;
    }
    var scripts = document.querySelectorAll('script[src*="wizard-nav.js"]');
    for (var i = 0; i < scripts.length; i++) {
      if (scripts[i].hasAttribute("data-wizard-chrome")) {
        return true;
      }
    }
    return false;
  }

  global.WyvernWizardNav = {
    boot: boot,
    collectPageData: collectPageData,
    isTerminalPage: isTerminalPage,
    isFirstPage: isFirstPage,
    applyChromeState: applyChromeState,
    normalizeData: normalizeData,
  };

  if (scriptOptedIn()) {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", boot);
    } else {
      boot();
    }
  }
})(typeof window !== "undefined" ? window : globalThis);
