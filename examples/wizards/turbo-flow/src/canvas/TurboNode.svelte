<script lang="ts">
  import { Handle, Position, type NodeProps } from "@xyflow/svelte";

  let { id, data }: NodeProps = $props();

  function onNodeActivate(event: MouseEvent) {
    const select = (
      window as Window & { __turboFlowSelectNode?: (nodeId: string) => void }
    ).__turboFlowSelectNode;
    select?.(id);
  }

  function onDoubleClick(event: MouseEvent) {
    event.stopPropagation();
    const configure = (
      window as Window & { __turboFlowConfigureNode?: (nodeId: string) => void }
    ).__turboFlowConfigureNode;
    configure?.(id);
  }
</script>

<div
  class="turbo-node"
  role="button"
  tabindex="0"
  data-testid={"turbo-node-" + id}
  onclick={onNodeActivate}
  ondblclick={onDoubleClick}
>
  <Handle type="target" position={Position.Top} />
  <div class="turbo-node__title">{data?.label || "Node"}</div>
  <div class="turbo-node__subtitle">{data?.subtitle || ""}</div>
  <Handle type="source" position={Position.Bottom} />
</div>

<style>
  .turbo-node {
    min-width: 9rem;
    padding: 0.75rem 0.9rem;
    border-radius: 12px;
    border: 1px solid rgba(42, 138, 246, 0.45);
    background: linear-gradient(160deg, #111827 0%, #1f2937 100%);
    color: #f8fafc;
    box-shadow: 0 10px 30px rgba(15, 23, 42, 0.35);
    font-family: "Segoe UI", system-ui, sans-serif;
  }

  .turbo-node__title {
    font-weight: 650;
    font-size: 0.95rem;
  }

  .turbo-node__subtitle {
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: #94a3b8;
  }
</style>
