/**
 * Agent DAG demo — turbo-flow canvas (vendored from examples/wizards/turbo-flow).
 * Finish maps Svelte graph state to data.dag. Page JS must not spawn agents
 * or run the DAG (execution deferred).
 */
(function (global) {
  "use strict";

  var merge = global.WyvernStackMerge;

  function pageId() {
    return (global.wyvern && global.wyvern.page && global.wyvern.page.id) || "";
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

  function layoutsFromConfig() {
    var config = global.wyvern && global.wyvern.config;
    var list = config && Array.isArray(config.layouts) ? config.layouts : [];
    return list;
  }

  function layoutIdForNodeCount(count) {
    var list = layoutsFromConfig();
    var i;
    for (i = 0; i < list.length; i += 1) {
      if (list[i] && Number(list[i].agents) === count) {
        return String(list[i].id);
      }
    }
    return "custom";
  }

  function activeNodeId() {
    var data = global.wyvern.page_data || {};
    if (data.editing_node_id) {
      return data.editing_node_id;
    }
    var intent = merge.latestCanvasIntent(global.wyvern.stack || []);
    if (intent) {
      return intent;
    }
    return data.node_id || null;
  }

  function detailDescriptor() {
    return {
      id: "node-detail",
      title: "Configure node",
      html: "pages/detail.html"
    };
  }

  function extrasDescriptor() {
    return {
      id: "node-extras",
      title: "Additional options",
      html: "pages/extras.html"
    };
  }

  function canvasDescriptor() {
    return {
      id: "canvas",
      title: "Agent DAG",
      html: "pages/canvas.html",
      layout: "workspace"
    };
  }

  function reviewDescriptor() {
    return {
      id: "review",
      title: "Review",
      html: "pages/review.html"
    };
  }

  function collectDetailForm() {
    var nameInput = document.querySelector("[data-testid='node-detail-name']");
    var roleInput = document.querySelector("[data-testid='node-detail-role']");
    var descInput = document.querySelector("[data-testid='node-detail-description']");
    return {
      node_id: activeNodeId(),
      name: nameInput ? nameInput.value.trim() : "",
      role: roleInput ? roleInput.value.trim() : "",
      description: descInput ? descInput.value.trim() : ""
    };
  }

  function collectExtrasForm() {
    var promptInput = document.querySelector("[data-testid='node-extras-prompt']");
    var toolInput = document.querySelector("[data-testid='node-extras-tool']");
    return {
      node_id: activeNodeId(),
      prompt: promptInput ? promptInput.value.trim() : "",
      tool: toolInput ? toolInput.value.trim() : ""
    };
  }

  function mergedGraphFromState() {
    return merge.readCachedGraph() || merge.mergeStack(global.wyvern.stack || []);
  }

  function nodeName(node, details) {
    var core = details && details[node.id] && details[node.id].core;
    if (core && typeof core.name === "string" && core.name.trim()) {
      return core.name.trim();
    }
    if (node.data && typeof node.data.label === "string" && node.data.label.trim()) {
      return node.data.label.trim();
    }
    return String(node.id);
  }

  function nodeRole(node, details) {
    var core = details && details[node.id] && details[node.id].core;
    if (core && typeof core.role === "string" && core.role.trim()) {
      return core.role.trim();
    }
    return "agent";
  }

  function assembleDag() {
    var graph = mergedGraphFromState();
    var details = graph.details || {};
    var nodes = (graph.nodes || []).map(function (node) {
      return {
        id: String(node.id),
        name: nodeName(node, details),
        role: nodeRole(node, details)
      };
    });
    var edges = (graph.edges || [])
      .filter(function (edge) {
        return edge && edge.source && edge.target;
      })
      .map(function (edge) {
        return [String(edge.source), String(edge.target)];
      });
    if (edges.length === 0 && nodes.length === 1) {
      edges.push([nodes[0].id, "finish"]);
    } else if (edges.length === 0 && nodes.length > 1) {
      var i;
      for (i = 0; i < nodes.length - 1; i += 1) {
        edges.push([nodes[i].id, nodes[i + 1].id]);
      }
    }
    return {
      layout_id: layoutIdForNodeCount(nodes.length),
      nodes: nodes,
      edges: edges
    };
  }

  function restoreDetailForm() {
    var nodeId = activeNodeId();
    if (!nodeId) {
      return;
    }
    var graph = mergedGraphFromState();
    var core = (graph.details[nodeId] && graph.details[nodeId].core) || {};
    var nameInput = document.querySelector("[data-testid='node-detail-name']");
    var roleInput = document.querySelector("[data-testid='node-detail-role']");
    var descInput = document.querySelector("[data-testid='node-detail-description']");
    if (nameInput) {
      nameInput.value = typeof core.name === "string" ? core.name : "";
    }
    if (roleInput) {
      roleInput.value = typeof core.role === "string" ? core.role : "";
    }
    if (descInput) {
      descInput.value = typeof core.description === "string" ? core.description : "";
    }
    var heading = document.querySelector("[data-testid='node-detail-heading']");
    if (heading) {
      heading.textContent = "Configure " + nodeId;
    }
  }

  function restoreExtrasForm() {
    var nodeId = activeNodeId();
    if (!nodeId) {
      return;
    }
    var graph = mergedGraphFromState();
    var extras = (graph.details[nodeId] && graph.details[nodeId].extras) || {};
    var promptInput = document.querySelector("[data-testid='node-extras-prompt']");
    var toolInput = document.querySelector("[data-testid='node-extras-tool']");
    if (promptInput) {
      promptInput.value = typeof extras.prompt === "string" ? extras.prompt : "";
    }
    if (toolInput) {
      toolInput.value = typeof extras.tool === "string" ? extras.tool : "";
    }
    var heading = document.querySelector("[data-testid='node-extras-heading']");
    if (heading) {
      heading.textContent = "Options for " + nodeId;
    }
  }

  function cacheGraphWithDetailCore() {
    var graph = mergedGraphFromState();
    var nodeId = activeNodeId();
    if (nodeId) {
      graph.details[nodeId] = graph.details[nodeId] || {};
      graph.details[nodeId].core = collectDetailForm();
    }
    merge.cacheGraph(graph);
  }

  function cacheGraphWithExtras() {
    var graph = mergedGraphFromState();
    var nodeId = activeNodeId();
    if (nodeId) {
      graph.details[nodeId] = graph.details[nodeId] || {};
      graph.details[nodeId].extras = collectExtrasForm();
    }
    merge.cacheGraph(graph);
  }

  function applyLayout() {
    if (
      typeof WyvernApi !== "undefined" &&
      typeof WyvernApi.applyWizardLayout === "function"
    ) {
      WyvernApi.applyWizardLayout(global.wyvern, global.__wyvernViewportBounds || null);
    }
  }

  function wireDetail() {
    restoreDetailForm();
    global.collectCurrentPageData = collectDetailForm;

    var form = document.querySelector("[data-testid='node-detail-form']");
    if (form) {
      form.addEventListener("submit", function (event) {
        event.preventDefault();
        cacheGraphWithDetailCore();
        global.wyvernWizardNext(collectDetailForm(), extrasDescriptor()).catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }

    var backGraph = document.querySelector("[data-testid='node-detail-back-graph']");
    if (backGraph) {
      backGraph.addEventListener("click", function () {
        cacheGraphWithDetailCore();
        global.wyvernWizardBack(collectDetailForm()).catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }

    var back = document.querySelector("[data-testid='node-detail-back']");
    if (back) {
      back.addEventListener("click", function () {
        cacheGraphWithDetailCore();
        global.wyvernWizardBack(collectDetailForm()).catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }

    applyLayout();
  }

  function wireExtras() {
    restoreExtrasForm();
    global.collectCurrentPageData = collectExtrasForm;

    var back = document.querySelector("[data-testid='node-extras-back']");
    if (back) {
      back.addEventListener("click", function () {
        cacheGraphWithExtras();
        global.wyvernWizardBack(collectExtrasForm()).catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }

    applyLayout();
  }

  function formatNodeDetail(nodeId, detail) {
    detail = detail || {};
    var core = detail.core || {};
    var extras = detail.extras || {};
    var lines = [];
    if (core.role) {
      lines.push("Role: " + core.role);
    }
    if (core.description) {
      lines.push("Description: " + core.description);
    }
    if (extras.prompt) {
      lines.push("Prompt: " + extras.prompt);
    }
    if (extras.tool) {
      lines.push("Tool: " + extras.tool);
    }
    if (lines.length === 0) {
      lines.push("No configuration saved");
    }
    return lines.join("\n");
  }

  function scheduleReviewLayout() {
    applyLayout();
    if (typeof requestAnimationFrame === "function") {
      requestAnimationFrame(function () {
        requestAnimationFrame(function () {
          applyLayout();
        });
      });
    }
  }

  function renderReview() {
    var dag = assembleDag();
    var idEl = document.querySelector("[data-testid='review-layout-id']");
    if (idEl) {
      idEl.textContent = dag.layout_id || "—";
    }

    var graph = mergedGraphFromState();
    var graphEl = document.querySelector("[data-testid='review-graph-summary']");
    if (graphEl) {
      graphEl.innerHTML = "";
      var graphCard = document.createElement("div");
      graphCard.className = "summary__card";
      var nodeCount = (graph.nodes || []).length;
      var edgeCount = (graph.edges || []).length;
      graphCard.innerHTML =
        "<div class='summary__title'>Graph</div><div class='summary__body'></div>";
      graphCard.querySelector(".summary__body").textContent =
        nodeCount + " node" + (nodeCount === 1 ? "" : "s") + ", " +
        edgeCount + " edge" + (edgeCount === 1 ? "" : "s");
      graphEl.appendChild(graphCard);
    }

    var list = document.querySelector("[data-testid='review-node-summary']");
    if (list) {
      list.innerHTML = "";
      (graph.nodes || []).forEach(function (node) {
        var li = document.createElement("li");
        li.className = "summary__card";
        var title = document.createElement("div");
        title.className = "summary__title";
        title.textContent = merge.nodeLabel(node, graph.details);
        var body = document.createElement("div");
        body.className = "summary__body";
        body.textContent = formatNodeDetail(node.id, graph.details[node.id]);
        li.appendChild(title);
        li.appendChild(body);
        list.appendChild(li);
      });
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

  function finishPayload() {
    return { dag: assembleDag() };
  }

  function wireReview() {
    renderReview();
    global.collectCurrentPageData = finishPayload;
    var finishBtn = document.querySelector("[data-testid='review-finish']");
    if (finishBtn) {
      finishBtn.addEventListener("click", function () {
        submitReviewFinish();
      });
    }
    var back = document.querySelector("[data-testid='review-back']");
    if (back) {
      back.addEventListener("click", function () {
        global.wyvernWizardBack().catch(function (err) {
          showError(String(err && err.message ? err.message : err));
        });
      });
    }
    scheduleReviewLayout();
  }

  async function submitReviewFinish() {
    var stack = ((global.wyvern && global.wyvern.stack) || []).slice();
    var data = finishPayload();
    stack.push({
      page: global.wyvern.page,
      data: data
    });
    try {
      await global.wyvernWizardFinish({
        button: "finish",
        data: data,
        stack: stack
      });
    } catch (err) {
      showError(String(err && err.message ? err.message : err));
    }
  }

  async function boot() {
    try {
      var path = (global.location && global.location.pathname) || "";
      if (path.indexOf("/canvas.html") !== -1) {
        return;
      }
      await ensureState();
      var id = pageId();
      if (id === "node-detail") {
        wireDetail();
      } else if (id === "node-extras") {
        wireExtras();
      } else if (id === "review") {
        wireReview();
      }
    } catch (err) {
      showError(String(err && err.message ? err.message : err));
    }
  }

  global.WyvernTurboFlow = {
    canvasDescriptor: canvasDescriptor,
    detailDescriptor: detailDescriptor,
    extrasDescriptor: extrasDescriptor,
    reviewDescriptor: reviewDescriptor,
    mergeStack: merge.mergeStack,
    assembleDag: assembleDag
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})(typeof window !== "undefined" ? window : globalThis);
