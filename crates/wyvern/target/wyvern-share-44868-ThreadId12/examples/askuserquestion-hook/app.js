/**
 * AskUserQuestion hook installer — collect `data.hook_config` only.
 * Page JS must not read or write Claude Code hook files (REQ-0124 / REQ-0125).
 * Paths come from `config.hook_state` after workflow pre.
 */
(function (global) {
  "use strict";

  function hookStateFromConfig(config) {
    var hs = (config && config.hook_state) || {};
    function scope(name) {
      var row = hs[name] || {};
      return {
        enabled: !!row.enabled,
        settings_path: typeof row.settings_path === "string" ? row.settings_path : ""
      };
    }
    return { global: scope("global"), repo: scope("repo") };
  }

  function hookConfigFromPrior() {
    var stack = (global.wyvern && Array.isArray(global.wyvern.stack)
      ? global.wyvern.stack
      : []) || [];
    for (var i = stack.length - 1; i >= 0; i -= 1) {
      var data = stack[i] && stack[i].data;
      if (data && data.hook_config) {
        return data.hook_config;
      }
    }
    var pageData = global.wyvern && global.wyvern.page_data;
    if (pageData && pageData.hook_config) {
      return pageData.hook_config;
    }
    var state = hookStateFromConfig(global.wyvern && global.wyvern.config);
    return {
      global: { enabled: state.global.enabled },
      repo: { enabled: state.repo.enabled }
    };
  }

  function collectHookConfig() {
    var globalEl = document.querySelector("[data-testid='toggle-global']");
    var repoEl = document.querySelector("[data-testid='toggle-repo']");
    if (globalEl && repoEl) {
      return {
        global: { enabled: !!globalEl.checked },
        repo: { enabled: !!repoEl.checked }
      };
    }
    return hookConfigFromPrior();
  }

  function bindToggles() {
    var initial = hookConfigFromPrior();
    var globalEl = document.querySelector("[data-testid='toggle-global']");
    var repoEl = document.querySelector("[data-testid='toggle-repo']");
    if (globalEl) {
      globalEl.checked = !!(initial.global && initial.global.enabled);
    }
    if (repoEl) {
      repoEl.checked = !!(initial.repo && initial.repo.enabled);
    }
  }

  function fillPath(testId, path) {
    var el = document.querySelector("[data-testid='" + testId + "']");
    if (el && path) {
      el.textContent = path;
    }
  }

  function fillScopePaths() {
    var state = hookStateFromConfig(global.wyvern && global.wyvern.config);
    fillPath("path-global", state.global.settings_path);
    fillPath("path-repo", state.repo.settings_path);
    fillPath("review-path-global", state.global.settings_path);
    fillPath("review-path-repo", state.repo.settings_path);
  }

  function fillReview() {
    var cfg = hookConfigFromPrior();
    var globalEl = document.querySelector("[data-testid='review-global']");
    var repoEl = document.querySelector("[data-testid='review-repo']");
    if (globalEl) {
      globalEl.textContent = cfg.global && cfg.global.enabled ? "Enable" : "Disable";
    }
    if (repoEl) {
      repoEl.textContent = cfg.repo && cfg.repo.enabled ? "Enable" : "Disable";
    }
  }

  global.collectCurrentPageData = function () {
    return { hook_config: collectHookConfig() };
  };

  if (document.querySelector("[data-testid='toggle-global']")) {
    global.wizardNextDescriptor = {
      id: "review",
      title: "Review hook scopes",
      html: "pages/review.html"
    };
  }

  function boot() {
    if (typeof global.wyvernWizardState === "function") {
      return global.wyvernWizardState().then(function () {
        bindToggles();
        fillScopePaths();
        fillReview();
      });
    }
    bindToggles();
    fillScopePaths();
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
