(function (global) {
  "use strict";

  function pageId() {
    return (global.wyvern && global.wyvern.page && global.wyvern.page.id) || "";
  }

  function layoutSelection() {
    var stack = (global.wyvern && global.wyvern.stack) || [];
    for (var i = 0; i < stack.length; i++) {
      var entry = stack[i];
      if (entry && entry.page && entry.page.id === "layout-picker") {
        return entry.data || {};
      }
    }
    return {};
  }

  function agentIndexFromId(id) {
    var match = /^agent-(\d+)$/.exec(id || "");
    return match ? Number(match[1]) : 0;
  }

  function nextAgentDescriptor(index) {
    return {
      id: "agent-" + index,
      title: "Agent " + index,
      html: "pages/agent.html",
    };
  }

  function finishDescriptor() {
    return {
      id: "finish",
      title: "Review",
      html: "pages/finish.html",
    };
  }

  function setText(el, text) {
    if (el) {
      el.textContent = text;
    }
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

  async function ensureState() {
    if (typeof global.wyvernWizardState === "function") {
      await global.wyvernWizardState();
    }
    if (!global.wyvern) {
      throw new Error("window.wyvern is not available");
    }
  }

  function renderLayoutPicker() {
    var root = document.querySelector("[data-testid='layout-cards']");
    if (!root) {
      return;
    }
    root.innerHTML = "";
    var layouts =
      (global.wyvern.config && global.wyvern.config.layouts) || [];
    layouts.forEach(function (layout) {
      var card = document.createElement("button");
      card.type = "button";
      card.className = "layout-card";
      card.dataset.testid = "layout-card-" + layout.id;
      card.setAttribute("data-layout-id", layout.id);

      var label = document.createElement("span");
      label.className = "layout-card__label";
      label.dataset.testid = "layout-label";
      label.textContent = layout.label;

      var agents = document.createElement("span");
      agents.className = "layout-card__agents";
      agents.dataset.testid = "layout-agents";
      agents.textContent = layout.agents + " agent" + (layout.agents === 1 ? "" : "s");

      card.appendChild(label);
      card.appendChild(agents);
      card.addEventListener("click", function () {
        selectLayout(layout);
      });
      root.appendChild(card);
    });
  }

  async function selectLayout(layout) {
    try {
      await global.wyvernWizardNext(
        {
          layout_id: layout.id,
          label: layout.label,
          agent_count: layout.agents,
        },
        nextAgentDescriptor(1)
      );
    } catch (err) {
      showError(String(err && err.message ? err.message : err));
    }
  }

  function collectAgentFormData() {
    var nameInput = document.querySelector("[data-testid='agent-name']");
    var descInput = document.querySelector("[data-testid='agent-description']");
    return {
      name: nameInput ? nameInput.value.trim() : "",
      description: descInput ? descInput.value.trim() : "",
    };
  }

  function restoreAgentForm() {
    var data = (global.wyvern && global.wyvern.page_data) || {};
    var nameInput = document.querySelector("[data-testid='agent-name']");
    var descInput = document.querySelector("[data-testid='agent-description']");
    if (nameInput && typeof data.name === "string") {
      nameInput.value = data.name;
    }
    if (descInput && typeof data.description === "string") {
      descInput.value = data.description;
    }
  }

  function wireAgentForm() {
    var form = document.querySelector("[data-testid='agent-form']");
    if (!form) {
      return;
    }
    var current = agentIndexFromId(pageId());
    var selection = layoutSelection();
    var agentCount = Number(selection.agent_count) || 1;
    setText(
      document.querySelector("[data-testid='agent-heading']"),
      "Agent " + current + " of " + agentCount
    );
    restoreAgentForm();
    global.collectCurrentPageData = collectAgentFormData;

    form.addEventListener("submit", function (event) {
      event.preventDefault();
      submitAgent(current, agentCount);
    });

    var back = document.querySelector("[data-testid='agent-back']");
    if (back) {
      back.addEventListener("click", function () {
        global.wyvernWizardBack().catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }
  }

  async function submitAgent(current, agentCount) {
    var nameInput = document.querySelector("[data-testid='agent-name']");
    var descInput = document.querySelector("[data-testid='agent-description']");
    var data = {
      name: nameInput ? nameInput.value.trim() : "",
      description: descInput ? descInput.value.trim() : "",
    };
    var next =
      current < agentCount
        ? nextAgentDescriptor(current + 1)
        : finishDescriptor();
    try {
      await global.wyvernWizardNext(data, next);
    } catch (err) {
      showError(String(err && err.message ? err.message : err));
    }
  }

  function renderFinishSummary() {
    var list = document.querySelector("[data-testid='finish-summary']");
    if (!list) {
      return;
    }
    list.innerHTML = "";
    var stack = (global.wyvern && global.wyvern.stack) || [];
    stack.forEach(function (entry) {
      var li = document.createElement("li");
      var title = document.createElement("div");
      title.className = "summary__title";
      title.textContent = entry.page ? entry.page.title || entry.page.id : "page";
      var body = document.createElement("div");
      body.className = "summary__body";
      body.textContent = JSON.stringify(entry.data || {}, null, 2);
      li.appendChild(title);
      li.appendChild(body);
      list.appendChild(li);
    });
  }

  function wireFinish() {
    renderFinishSummary();
    var finishBtn = document.querySelector("[data-testid='finish-submit']");
    if (finishBtn) {
      finishBtn.addEventListener("click", function () {
        submitFinish();
      });
    }
    var back = document.querySelector("[data-testid='finish-back']");
    if (back) {
      back.addEventListener("click", function () {
        global.wyvernWizardBack().catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }
  }

  async function submitFinish() {
    var currentData = {};
    var stack = ((global.wyvern && global.wyvern.stack) || []).slice();
    stack.push({
      page: global.wyvern.page,
      data: currentData,
    });
    try {
      await global.wyvernWizardFinish({
        button: "finish",
        data: currentData,
        stack: stack,
      });
    } catch (err) {
      showError(String(err && err.message ? err.message : err));
    }
  }

  async function boot() {
    try {
      await ensureState();
      var id = pageId();
      if (id === "layout-picker") {
        renderLayoutPicker();
        if (
          typeof WyvernApi !== "undefined" &&
          typeof WyvernApi.applyWizardLayout === "function"
        ) {
          WyvernApi.applyWizardLayout(
            global.wyvern,
            window.__wyvernViewportBounds || null
          );
        }
      } else if (/^agent-\d+$/.test(id)) {
        wireAgentForm();
      } else if (id === "finish") {
        wireFinish();
      }
    } catch (err) {
      showError(String(err && err.message ? err.message : err));
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})(typeof window !== "undefined" ? window : globalThis);
