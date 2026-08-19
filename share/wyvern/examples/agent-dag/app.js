/**
 * Agent DAG demo — assemble finish data.dag from config.layouts + stack.
 * Page JS must not spawn agents or run the DAG (execution deferred).
 */
(function (global) {
  "use strict";

  function layoutsFromConfig() {
    var config = global.wyvern && global.wyvern.config;
    var list = config && Array.isArray(config.layouts) ? config.layouts : [];
    return list;
  }

  function findLayout(id) {
    var list = layoutsFromConfig();
    for (var i = 0; i < list.length; i += 1) {
      if (list[i] && list[i].id === id) {
        return list[i];
      }
    }
    return null;
  }

  function pageId() {
    return (global.wyvern && global.wyvern.page && global.wyvern.page.id) || "";
  }

  function stackEntries() {
    var stack = (global.wyvern && Array.isArray(global.wyvern.stack)
      ? global.wyvern.stack
      : []) || [];
    return stack;
  }

  function agentIndexFromId(id) {
    var match = /^agent-(\d+)$/.exec(id || "");
    return match ? Number(match[1]) : 0;
  }

  function agentDescriptor(index) {
    return {
      id: "agent-" + index,
      title: "Agent " + index,
      html: "pages/agent.html"
    };
  }

  function reviewDescriptor() {
    return {
      id: "review",
      title: "Review",
      html: "pages/review.html"
    };
  }

  function layoutIdFromState() {
    var stack = stackEntries();
    var i;
    for (i = 0; i < stack.length; i += 1) {
      var entry = stack[i];
      if (entry && entry.page && entry.page.id === "layout" && entry.data && entry.data.layout_id) {
        return String(entry.data.layout_id);
      }
    }
    var pageData = global.wyvern && global.wyvern.page_data;
    if (pageData && pageData.layout_id) {
      return String(pageData.layout_id);
    }
    return "";
  }

  function agentCountFor(layoutId) {
    var layout = findLayout(layoutId);
    var n = layout && layout.agents != null ? Number(layout.agents) : 0;
    return n > 0 ? n : 1;
  }

  function collectAgentFields() {
    var nameEl = document.querySelector("[data-testid='agent-name']");
    var roleEl = document.querySelector("[data-testid='agent-role']");
    var pageData = (global.wyvern && global.wyvern.page_data) || {};
    return {
      name: nameEl ? String(nameEl.value || "") : (pageData.name ? String(pageData.name) : ""),
      role: roleEl ? String(roleEl.value || "") : (pageData.role ? String(pageData.role) : "")
    };
  }

  function agentsFromVisited() {
    var byId = {};
    var stack = stackEntries();
    var i;
    for (i = 0; i < stack.length; i += 1) {
      var entry = stack[i];
      var id = entry && entry.page ? entry.page.id : "";
      if (!agentIndexFromId(id)) {
        continue;
      }
      byId[id] = {
        id: id,
        name: entry.data && entry.data.name != null ? String(entry.data.name) : "",
        role: entry.data && entry.data.role != null ? String(entry.data.role) : ""
      };
    }
    var current = pageId();
    if (agentIndexFromId(current)) {
      var fields = collectAgentFields();
      byId[current] = {
        id: current,
        name: fields.name,
        role: fields.role
      };
    }
    var layoutId = layoutIdFromState();
    var count = agentCountFor(layoutId);
    var nodes = [];
    for (i = 1; i <= count; i += 1) {
      var agentId = "agent-" + i;
      if (byId[agentId]) {
        nodes.push(byId[agentId]);
      }
    }
    return nodes;
  }

  function assembleDag() {
    var layoutId = layoutIdFromState() || "solo";
    var nodes = agentsFromVisited();
    var edges = [];
    if (nodes.length === 0) {
      edges.push(["layout-picker", "finish"]);
    } else {
      edges.push(["layout-picker", nodes[0].id]);
      var i;
      for (i = 0; i < nodes.length - 1; i += 1) {
        edges.push([nodes[i].id, nodes[i + 1].id]);
      }
      edges.push([nodes[nodes.length - 1].id, "finish"]);
    }
    return {
      layout_id: layoutId,
      nodes: nodes,
      edges: edges
    };
  }

  function highlightSelected(id) {
    var cards = document.querySelectorAll("[data-layout-id]");
    for (var i = 0; i < cards.length; i += 1) {
      if (cards[i].getAttribute("data-layout-id") === id) {
        cards[i].classList.add("is-selected");
      } else {
        cards[i].classList.remove("is-selected");
      }
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

  function renderLayoutCards() {
    var root = document.querySelector("[data-testid='layout-cards']");
    if (!root) {
      return;
    }
    root.innerHTML = "";
    var selected = layoutIdFromState();
    layoutsFromConfig().forEach(function (layout) {
      if (!layout || !layout.id) {
        return;
      }
      var card = document.createElement("button");
      card.type = "button";
      card.className = "layout-card";
      card.setAttribute("data-layout-id", layout.id);
      card.dataset.testid = "layout-card-" + layout.id;
      var label = document.createElement("strong");
      label.textContent = layout.label || layout.id;
      var agents = document.createElement("span");
      agents.className = "layout-card__agents";
      var n = Number(layout.agents) || 0;
      agents.textContent = n + " agent" + (n === 1 ? "" : "s");
      card.appendChild(label);
      card.appendChild(agents);
      card.addEventListener("click", function () {
        highlightSelected(layout.id);
        if (typeof global.wyvernWizardNext === "function") {
          global.wyvernWizardNext(
            { layout_id: String(layout.id) },
            agentDescriptor(1)
          ).catch(function (err) {
            showError(String(err && err.message ? err.message : err));
          });
        }
      });
      root.appendChild(card);
    });
    if (selected) {
      highlightSelected(selected);
    }
  }

  function restoreAgentForm() {
    var form = document.querySelector("[data-testid='agent-form']");
    if (!form) {
      return;
    }
    var current = agentIndexFromId(pageId());
    var count = agentCountFor(layoutIdFromState());
    var heading = document.querySelector("[data-testid='agent-heading']");
    if (heading && current) {
      heading.textContent = "Agent " + current + " of " + count;
    }
    var data = (global.wyvern && global.wyvern.page_data) || {};
    var nameEl = document.querySelector("[data-testid='agent-name']");
    var roleEl = document.querySelector("[data-testid='agent-role']");
    if (nameEl && typeof data.name === "string") {
      nameEl.value = data.name;
    }
    if (roleEl && typeof data.role === "string") {
      roleEl.value = data.role;
    }
  }

  function renderReview() {
    var idEl = document.querySelector("[data-testid='review-layout-id']");
    if (!idEl) {
      return;
    }
    var dag = assembleDag();
    idEl.textContent = dag.layout_id || "—";
    var nodesEl = document.querySelector("[data-testid='review-nodes']");
    if (nodesEl) {
      nodesEl.textContent = dag.nodes.length
        ? dag.nodes.map(function (n) {
          return n.id + " (" + (n.name || "unnamed") + " / " + (n.role || "role") + ")";
        }).join(" · ")
        : "No agents";
    }
    var edgesEl = document.querySelector("[data-testid='review-edges']");
    if (edgesEl) {
      edgesEl.innerHTML = "";
      dag.edges.forEach(function (edge) {
        var li = document.createElement("li");
        li.textContent = edge[0] + " → " + edge[1];
        edgesEl.appendChild(li);
      });
    }
  }

  function nextAfterAgent() {
    var current = agentIndexFromId(pageId());
    var count = agentCountFor(layoutIdFromState());
    if (current && current < count) {
      return agentDescriptor(current + 1);
    }
    return reviewDescriptor();
  }

  global.collectCurrentPageData = function () {
    if (document.querySelector("[data-testid='layout-cards']")) {
      var id = layoutIdFromState();
      return id ? { layout_id: id } : {};
    }
    if (document.querySelector("[data-testid='agent-form']")) {
      return collectAgentFields();
    }
    return { dag: assembleDag() };
  };

  if (document.querySelector("[data-testid='layout-cards']")) {
    global.wizardNextDescriptor = function () {
      return agentDescriptor(1);
    };
  } else if (document.querySelector("[data-testid='agent-form']")) {
    global.wizardNextDescriptor = nextAfterAgent;
  }

  function boot() {
    if (typeof global.wyvernWizardState === "function") {
      return global.wyvernWizardState().then(function () {
        renderLayoutCards();
        restoreAgentForm();
        renderReview();
      });
    }
    renderLayoutCards();
    restoreAgentForm();
    renderReview();
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
