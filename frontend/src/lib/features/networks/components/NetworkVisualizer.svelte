<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as d3 from "d3";
  import type { NetworkController, TopologyNode, TopologyEdge } from "../network-controller.svelte";
  import { Search, ZoomIn, ZoomOut, RefreshCw } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";

  let { ctrl }: { ctrl: NetworkController } = $props();

  let svgElement: SVGSVGElement;
  let container: HTMLDivElement;
  let width = $state(0);
  let height = $state(0);

  let simulation: d3.Simulation<any, undefined>;
  let svg: d3.Selection<SVGSVGElement, unknown, null, undefined>;
  let g: d3.Selection<SVGGElement, unknown, null, undefined>;

  let nodes = $derived(ctrl.topology.nodes);
  let edges = $derived(ctrl.topology.edges);

  function initGraph() {
    if (!svgElement || nodes.length === 0) return;

    if (simulation) simulation.stop();

    svg = d3.select(svgElement);
    svg.selectAll("*").remove();

    g = svg.append("g");

    // Zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        g.attr("transform", event.transform);
      });

    svg.call(zoom);

    // Use snapshots to prevent Svelte 5 Proxy mutation issues with D3
    const nodesData = $state.snapshot(nodes).map((n: any) => ({ ...n }));
    const linksMapped = $state.snapshot(edges).map((d: any) => ({
        ...d,
        source: d.from,
        target: d.to
    }));

    simulation = d3.forceSimulation(nodesData as any)
      .force("link", d3.forceLink(linksMapped).id((d: any) => d.id).distance(150))
      .force("charge", d3.forceManyBody().strength(-500))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide().radius(60));

    // Draw lines
    const link = g.append("g")
      .attr("stroke", "#94a3b8")
      .attr("stroke-opacity", 0.6)
      .selectAll("line")
      .data(linksMapped)
      .join("line")
      .attr("stroke-width", 2);

    // Draw nodes
    const node = g.append("g")
      .selectAll("g")
      .data(nodesData)
      .join("g")
      .call(d3.drag<any, any>()
        .on("start", dragstarted)
        .on("drag", dragged)
        .on("end", dragended) as any);

    // Node circles
    node.append("circle")
      .attr("r", (d: any) => d.type === 'network' ? 30 : 20)
      .attr("fill", (d: any) => {
        if (d.type === 'network') return "#8b5cf6";
        if (d.type === 'container') {
            return d.metadata.state === 'running' ? "#22c55e" : "#ef4444";
        }
        return "#f59e0b";
      })
      .attr("stroke", "#fff")
      .attr("stroke-width", 2)
      .attr("class", "cursor-pointer transition-all hover:scale-110");

    // Labels
    node.append("text")
      .text((d: any) => d.label)
      .attr("x", 0)
      .attr("y", (d: any) => d.type === 'network' ? 45 : 35)
      .attr("text-anchor", "middle")
      .attr("fill", "currentColor")
      .attr("class", "text-[10px] font-medium pointer-events-none select-none");

    // Icons/Type labels
    node.append("text")
      .text((d: any) => d.type === 'network' ? '🌐' : '📦')
      .attr("x", 0)
      .attr("y", 5)
      .attr("text-anchor", "middle")
      .attr("class", "text-[12px] pointer-events-none select-none");

    simulation.on("tick", () => {
      link
        .attr("x1", (d: any) => d.source.x)
        .attr("y1", (d: any) => d.source.y)
        .attr("x2", (d: any) => d.target.x)
        .attr("y2", (d: any) => d.target.y);

      node.attr("transform", (d: any) => `translate(${d.x},${d.y})`);
    });

    function dragstarted(event: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: any) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragended(event: any) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }
  }

  $effect(() => {
    if (nodes.length > 0 && width > 0) {
      initGraph();
    }
  });

  onMount(() => {
    const updateSize = () => {
      if (container) {
        width = container.clientWidth;
        height = container.clientHeight || 600;
      }
    };
    window.addEventListener("resize", updateSize);
    updateSize();
  });

  onDestroy(() => {
    if (simulation) simulation.stop();
  });

  function handleZoom(type: 'in' | 'out' | 'reset') {
    if (!svg) return;
    const transition = svg.transition().duration(300);
    if (type === 'reset') {
       svg.call(d3.zoom<SVGSVGElement, unknown>().transform as any, d3.zoomIdentity);
    } else {
        const factor = type === 'in' ? 1.5 : 0.6;
        svg.call(d3.zoom<SVGSVGElement, unknown>().scaleBy as any, factor);
    }
  }
</script>

<div class="relative flex flex-col w-full h-[600px] border rounded-xl bg-background overflow-hidden" bind:this={container}>
  <div class="absolute top-4 left-4 z-10 flex flex-col gap-2">
    <div class="bg-card/80 backdrop-blur-sm border p-4 rounded-lg shadow-sm">
        <h3 class="text-sm font-semibold mb-2">Legend</h3>
        <div class="space-y-2">
            <div class="flex items-center gap-2">
                <div class="w-3 h-3 rounded-full bg-[#8b5cf6]"></div>
                <span class="text-xs">Docker Network</span>
            </div>
            <div class="flex items-center gap-2">
                <div class="w-3 h-3 rounded-full bg-[#22c55e]"></div>
                <span class="text-xs">Running Container</span>
            </div>
            <div class="flex items-center gap-2">
                <div class="w-3 h-3 rounded-full bg-[#ef4444]"></div>
                <span class="text-xs">Stopped Container</span>
            </div>
        </div>
    </div>
  </div>

  <div class="absolute top-4 right-4 z-10 flex gap-2">
    <Button variant="outline" size="icon" onclick={() => ctrl.loadTopology()} disabled={ctrl.loading}>
        <RefreshCw class="h-4 w-4 {ctrl.loading ? 'animate-spin' : ''}" />
    </Button>
    <Button variant="outline" size="icon" onclick={() => handleZoom('in')}>
        <ZoomIn class="h-4 w-4" />
    </Button>
    <Button variant="outline" size="icon" onclick={() => handleZoom('out')}>
        <ZoomOut class="h-4 w-4" />
    </Button>
    <Button variant="outline" onclick={() => handleZoom('reset')}>
        Reset
    </Button>
  </div>

  {#if ctrl.loading && nodes.length === 0}
    <div class="absolute inset-0 flex items-center justify-center bg-background/50 backdrop-blur-[2px] z-20">
      <div class="flex flex-col items-center gap-2">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        <p class="text-sm text-muted-foreground">Discovering network topology...</p>
      </div>
    </div>
  {/if}

  <svg
    bind:this={svgElement}
    class="w-full h-full cursor-grab active:cursor-grabbing"
    viewBox="0 0 {width} {height}"
  ></svg>
</div>

<style>
  svg :global(text) {
    pointer-events: none;
  }
</style>
