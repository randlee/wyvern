<script lang="ts">
  import { get, writable } from "svelte/store";
  import {
    SvelteFlow,
    Controls,
    SvelteFlowProvider,
    type Node,
    type Edge,
    type Connection,
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";
  import TurboNode from "./TurboNode.svelte";
  import TurboEdge from "./TurboEdge.svelte";
  import { initialNodes, initialEdges, nextNodeId } from "./nodes-edges";
  import "./index.css";

  type GraphDetails = Record<
    string,
    { core?: Record<string, unknown>; extras?: Record<string, unknown> }
  >;

  type WyvernGraph = {
    nodes: Array<Record<string, unknown>>;
    edges: Array<Record<string, unknown>>;
    details: GraphDetails;
  };

  declare global {
    interface Window {
      wyvern?: {
        config?: Record<string, unknown>;
        page?: { id?: string; layout?: string };
        stack?: Array<{ page?: { id?: string }; data?: Record<string, unknown> }>;
      };
      wyvernWizardState?: () => Promise<unknown>;
      wyvernWizardNext?: (data: unknown, next: unknown) => Promise<unknown>;
      WyvernApi?: { applyWizardLayout?: (state: unknown, viewport: unknown) => unknown };
      WyvernStackMerge?: {
        mergeStack: (stack: unknown) => WyvernGraph;
        nodeLabel: (node: Node, details: GraphDetails) => string;
      };
      WyvernTurboFlow?: {
        detailDescriptor: () => Record<string, string>;
        reviewDescriptor: () => Record<string, string>;
      };
      __wyvernViewportBounds?: unknown;
      __turboFlowSelectNode?: (nodeId: string) => void;
      __turboFlowConfigureNode?: (nodeId: string) => void;
    }
  }

  const nodeTypes = { turbo: TurboNode };
  const edgeTypes = { turbo: TurboEdge };
  const defaultEdgeOptions = { type: "turbo", markerEnd: "edge-circle" };

  const nodes = writable<Node[]>(initialNodes.map((n) => ({ ...n })));
  const edges = writable<Edge[]>(initialEdges.map((e) => ({ ...e })));
  let details = $state<GraphDetails>({});
  let selectedId = $state<string | null>(null);
  let colorMode = $state<"dark" | "light">("dark");
  let error = $state("");
  let ready = $state(false);

  function resolveColorMode(): "dark" | "light" {
    const theme = window.wyvern?.config?.theme;
    return theme === "light" ? "light" : "dark";
  }

  function labelForNode(node: Node): string {
    if (window.WyvernStackMerge) {
      return window.WyvernStackMerge.nodeLabel(node, details);
    }
    return (node.data?.label as string) || node.id;
  }

  function refreshNodeLabels() {
    nodes.update((current) =>
      current.map((node) => ({
        ...node,
        data: {
          ...node.data,
          label: labelForNode(node),
          subtitle:
            details[node.id]?.core?.role ||
            details[node.id]?.core?.description ||
            "Click Configure",
        },
      }))
    );
  }

  function serializeGraph(editingNodeId: string | null = null): Record<string, unknown> {
    const nodeList = get(nodes);
    const edgeList = get(edges);
    return {
      nodes: nodeList.map((n) => ({
        id: n.id,
        type: n.type,
        position: n.position,
        data: n.data,
      })),
      edges: edgeList.map((e) => ({
        id: e.id,
        source: e.source,
        target: e.target,
        type: e.type,
      })),
      details,
      editing_node_id: editingNodeId,
    };
  }

  function hydrateFromStack() {
    const stack = window.wyvern?.stack || [];
    const cached = window.WyvernStackMerge?.consumeCachedGraph();
    const merge = cached || window.WyvernStackMerge?.mergeStack(stack);
    if (!merge || merge.nodes.length === 0) {
      nodes.set(initialNodes.map((n) => ({ ...n })));
      edges.set(initialEdges.map((e) => ({ ...e })));
      details = {};
      return;
    }
    details = merge.details || {};
    nodes.set(
      merge.nodes.map((raw) => ({
        id: String(raw.id),
        type: (raw.type as string) || "turbo",
        position: (raw.position as { x: number; y: number }) || { x: 0, y: 0 },
        data: (raw.data as Record<string, unknown>) || {},
      })) as Node[]
    );
    edges.set(
      merge.edges.map((raw) => ({
        id: String(raw.id),
        source: String(raw.source),
        target: String(raw.target),
        type: (raw.type as string) || "turbo",
      })) as Edge[]
    );
    refreshNodeLabels();
  }

  function onConnect(connection: Connection) {
    const id = `edge-${connection.source}-${connection.target}`;
    edges.update((current) => {
      if (current.some((e) => e.id === id)) {
        return current;
      }
      return [
        ...current,
        {
          id,
          source: connection.source || "",
          target: connection.target || "",
          type: "turbo",
        },
      ];
    });
  }

  function addNode() {
    const nodeList = get(nodes);
    const id = nextNodeId(nodeList);
    const offset = nodeList.length * 40;
    nodes.update((current) => [
      ...current,
      {
        id,
        type: "turbo",
        position: { x: 80 + offset, y: 120 + offset },
        data: { label: id, subtitle: "New node" },
      },
    ]);
    selectedId = id;
  }

  async function openDetailFor(nodeId: string) {
    selectedId = nodeId;
    await openDetail();
  }

  function handleNodeClick(nodeId: string) {
    selectedId = nodeId;
  }

  function registerNodeInteractions() {
    window.__turboFlowSelectNode = (nodeId: string) => {
      selectedId = nodeId;
    };
    window.__turboFlowConfigureNode = (nodeId: string) => {
      void openDetailFor(nodeId);
    };
  }

  async function openDetail() {
    if (!selectedId) {
      error = "Select a node to configure.";
      return;
    }
    error = "";
    try {
      if (window.WyvernStackMerge?.cacheGraph) {
        window.WyvernStackMerge.cacheGraph({
          nodes: get(nodes).map((n) => ({
            id: n.id,
            type: n.type,
            position: n.position,
            data: n.data,
          })),
          edges: get(edges).map((e) => ({
            id: e.id,
            source: e.source,
            target: e.target,
            type: e.type,
          })),
          details,
        });
      }
      const wizardNext =
        window.wyvernWizardNext || window.WyvernApi?.wyvernWizardNext;
      await wizardNext?.(
        serializeGraph(selectedId),
        window.WyvernTurboFlow?.detailDescriptor()
      );
    } catch (err) {
      error = String(err instanceof Error ? err.message : err);
    }
  }

  async function openReview() {
    error = "";
    try {
      if (window.WyvernStackMerge?.cacheGraph) {
        window.WyvernStackMerge.cacheGraph({
          nodes: get(nodes).map((n) => ({
            id: n.id,
            type: n.type,
            position: n.position,
            data: n.data,
          })),
          edges: get(edges).map((e) => ({
            id: e.id,
            source: e.source,
            target: e.target,
            type: e.type,
          })),
          details,
        });
      }
      const wizardNext =
        window.wyvernWizardNext || window.WyvernApi?.wyvernWizardNext;
      await wizardNext?.(serializeGraph(null), window.WyvernTurboFlow?.reviewDescriptor());
    } catch (err) {
      error = String(err instanceof Error ? err.message : err);
    }
  }

  async function boot() {
    try {
      await window.wyvernWizardState?.();
      colorMode = resolveColorMode();
      registerNodeInteractions();
      hydrateFromStack();
      window.WyvernApi?.applyWizardLayout?.(window.wyvern, window.__wyvernViewportBounds || null);
      ready = true;
    } catch (err) {
      error = String(err instanceof Error ? err.message : err);
    }
  }

  boot();
</script>

<SvelteFlowProvider>
  <main class="turbo-flow-shell dialog dialog--workspace" id="dialog" data-testid="turbo-flow-workspace">
    <header class="turbo-flow-toolbar">
      <h1>Turbo flow</h1>
      <button type="button" data-testid="turbo-flow-add-node" onclick={addNode}>Add node</button>
      <button
        type="button"
        class="primary"
        data-testid="turbo-flow-configure"
        disabled={!selectedId}
        onclick={openDetail}
      >
        Configure
      </button>
      <button type="button" data-testid="turbo-flow-review" onclick={openReview}>Review</button>
    </header>
    {#if error}
      <p class="turbo-flow-error" data-testid="turbo-flow-error">{error}</p>
    {/if}
    {#if ready}
      <div class="turbo-flow-canvas">
        <SvelteFlow
          {nodes}
          {edges}
          {nodeTypes}
          {edgeTypes}
          {defaultEdgeOptions}
          fitView
          zoomOnDoubleClick={false}
          colorMode={colorMode}
          on:connect={(event) => onConnect(event.detail)}
          on:nodeclick={(event) => {
            handleNodeClick(event.detail.node.id);
          }}
        >
          <Controls showLock={false} />
          <svg>
            <defs>
              <linearGradient id="edge-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stop-color="#ae53ba" />
                <stop offset="100%" stop-color="#2a8af6" />
              </linearGradient>
              <marker
                id="edge-circle"
                viewBox="-5 -5 10 10"
                refX="0"
                refY="0"
                markerUnits="strokeWidth"
                markerWidth="10"
                markerHeight="10"
                orient="auto"
              >
                <circle stroke="#2a8af6" stroke-opacity="0.75" r="2" cx="0" cy="0" fill="none" />
              </marker>
            </defs>
          </svg>
        </SvelteFlow>
      </div>
    {/if}
  </main>
</SvelteFlowProvider>
