import type { Node, Edge } from "@xyflow/svelte";

export const initialNodes: Node[] = [
  {
    id: "node-1",
    type: "turbo",
    position: { x: 0, y: 0 },
    data: { label: "Agent 1", subtitle: "Click Configure" },
  },
  {
    id: "node-2",
    type: "turbo",
    position: { x: 250, y: 80 },
    data: { label: "Agent 2", subtitle: "Connect & configure" },
  },
];

export const initialEdges: Edge[] = [
  {
    id: "edge-1-2",
    source: "node-1",
    target: "node-2",
    type: "turbo",
  },
];

export function nextNodeId(nodes: Node[]): string {
  var max = 0;
  nodes.forEach(function (node) {
    var match = /^node-(\d+)$/.exec(node.id);
    if (match) {
      max = Math.max(max, Number(match[1]));
    }
  });
  return "node-" + (max + 1);
}
