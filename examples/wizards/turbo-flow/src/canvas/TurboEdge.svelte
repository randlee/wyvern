<script lang="ts">
  import { getBezierPath, type EdgeProps } from "@xyflow/svelte";

  let { id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition }: EdgeProps =
    $props();

  let path = $derived(
    getBezierPath({
      sourceX,
      sourceY,
      targetX,
      targetY,
      sourcePosition,
      targetPosition,
    })[0]
  );
</script>

<path
  {id}
  class="turbo-edge"
  d={path}
  fill="none"
  stroke="url(#edge-gradient)"
  stroke-width="2"
  marker-end="url(#edge-circle)"
/>

<style>
  .turbo-edge {
    filter: drop-shadow(0 0 6px rgba(42, 138, 246, 0.35));
  }
</style>
