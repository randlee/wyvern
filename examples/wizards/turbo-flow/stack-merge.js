/**
 * Merge wizard stack entries into a single graph snapshot for the canvas.
 * Canvas blob is authoritative for topology; detail/extras fold into details[nodeId].
 */
(function (global) {
  "use strict";

  function emptyGraph() {
    return { nodes: [], edges: [], details: {} };
  }

  function cloneGraph(graph) {
    return {
      nodes: (graph.nodes || []).map(function (n) {
        return Object.assign({}, n);
      }),
      edges: (graph.edges || []).map(function (e) {
        return Object.assign({}, e);
      }),
      details: Object.assign({}, graph.details || {}),
    };
  }

  function mergeStack(stack) {
    var graph = emptyGraph();
    if (!Array.isArray(stack)) {
      return graph;
    }

    stack.forEach(function (entry) {
      if (!entry || !entry.page || !entry.data) {
        return;
      }
      var id = entry.page.id;
      var data = entry.data;

      if (id === "canvas") {
        if (Array.isArray(data.nodes)) {
          graph.nodes = data.nodes.map(function (n) {
            return Object.assign({}, n);
          });
        }
        if (Array.isArray(data.edges)) {
          graph.edges = data.edges.map(function (e) {
            return Object.assign({}, e);
          });
        }
        if (data.details && typeof data.details === "object") {
          graph.details = Object.assign({}, graph.details, data.details);
        }
        return;
      }

      if (id === "node-detail" && data.node_id) {
        graph.details[data.node_id] = graph.details[data.node_id] || {};
        graph.details[data.node_id].core = Object.assign({}, data);
        return;
      }

      if (id === "node-extras" && data.node_id) {
        graph.details[data.node_id] = graph.details[data.node_id] || {};
        graph.details[data.node_id].extras = Object.assign({}, data);
      }
    });

    return graph;
  }

  function latestCanvasIntent(stack) {
    if (!Array.isArray(stack)) {
      return null;
    }
    for (var i = stack.length - 1; i >= 0; i--) {
      var entry = stack[i];
      if (entry && entry.page && entry.page.id === "canvas" && entry.data) {
        return entry.data.editing_node_id || null;
      }
    }
    return null;
  }

  function nodeLabel(node, details) {
    var core = details && details[node.id] && details[node.id].core;
    if (core && typeof core.name === "string" && core.name.trim()) {
      return core.name.trim();
    }
    if (node.data && typeof node.data.label === "string") {
      return node.data.label;
    }
    return node.id;
  }

  var CACHE_KEY = "wyvern.turbo-flow.graph";

  function readCachedGraph() {
    try {
      var raw = global.sessionStorage && global.sessionStorage.getItem(CACHE_KEY);
      if (!raw) {
        return null;
      }
      var parsed = JSON.parse(raw);
      if (!parsed || !Array.isArray(parsed.nodes)) {
        return null;
      }
      return {
        nodes: parsed.nodes,
        edges: parsed.edges || [],
        details: parsed.details || {},
      };
    } catch (_err) {
      return null;
    }
  }

  function cacheGraph(graph) {
    try {
      if (global.sessionStorage) {
        global.sessionStorage.setItem(CACHE_KEY, JSON.stringify(graph));
      }
    } catch (_err) {
      // ignore quota / private mode
    }
  }

  function consumeCachedGraph() {
    var graph = readCachedGraph();
    try {
      if (global.sessionStorage) {
        global.sessionStorage.removeItem(CACHE_KEY);
      }
    } catch (_err) {
      // ignore
    }
    return graph;
  }

  global.WyvernStackMerge = {
    emptyGraph: emptyGraph,
    cloneGraph: cloneGraph,
    mergeStack: mergeStack,
    latestCanvasIntent: latestCanvasIntent,
    nodeLabel: nodeLabel,
    readCachedGraph: readCachedGraph,
    cacheGraph: cacheGraph,
    consumeCachedGraph: consumeCachedGraph,
  };
})(typeof window !== "undefined" ? window : globalThis);
