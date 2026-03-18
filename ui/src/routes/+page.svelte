<script lang="ts">
  import { onMount } from "svelte";
  import { Share2 } from "@lucide/svelte";
  import {
    SvelteFlow,
    type Node,
    type Edge,
    Background,
    Controls,
    useSvelteFlow
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";
 import Dagre from '@dagrejs/dagre';

  let nodes = $state<Node[]>([]);
  let edges = $state<Edge[]>([]);
  let socket: WebSocket;
    const { fitView } = useSvelteFlow();
  const nodeWidth = 150;
const nodeHeight = 50;



   function getLayoutedElements(nodes: Node[], edges: Edge[]) {
    const g = new Dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));
    const options = {direction : 'TB'}
    g.setGraph({ rankdir: options.direction });
    g.setDefaultEdgeLabel(() => ({}));

    nodes.forEach((node: Node) => {
    g.setNode(node.id, { width: nodeWidth, height: nodeHeight });
  });

 edges.forEach((edge: Edge) => {
    g.setEdge(edge.source, edge.target);
  });
    Dagre.layout(g);
 
    return {
      nodes: nodes.map((node) => {
        const nodeWithPosition = g.node(node.id);
 
        return {
          ...node,
           position: {
        x: nodeWithPosition.x - nodeWidth / 2,
        y: nodeWithPosition.y - nodeHeight / 2,
      },
          // sourcePosition: options.direction === 'LR' ? 'right' : 'bottom',
          // targetPosition: options.direction === 'LR' ? 'left' : 'top',
        };
      }),
      edges,
    };
  }

  onMount(() => {
    socket = new WebSocket("ws://localhost:8000/ws");

    socket.onmessage = (event) => {
      const update = JSON.parse(event.data);
      if (update.kind === "graph") {
        const payload = update.payload;

        const _nodes = payload.nodes.map((n: any, i: number) => ({
          id: n.id,
          data: { label: n.label },
          position: { x: i * 150, y: i * 50 },
          class: 'bg-base-100! text-base-content-200! border border-base-content-100 rounded-md p-2 rounded'
        }));

        const _edges = payload.edges.map((e: any, i: number) => ({
          id: `e-${i}`,
          source: e.source,
          target: e.target,
          class: 'stroke-gray/60 stroke-2',
          // markerEnd: { type: "arrowclosed", color: theme.edge },
        }));

                const layouted = getLayoutedElements(_nodes, _edges);
        nodes = [...layouted.nodes]
        edges = [...layouted.edges]
         fitView();

      }
    };

    return () => {
      socket.close();
    };
  });

</script>

<main class="w-screen h-screen relative overflow-hidden">
  <!-- UI Overlay -->
  <div class="absolute top-4 left-4 z-10 flex flex-col gap-2">
    <div
      class="flex items-center gap-2 p-3 rounded-lg border shadow-xl backdrop-blur-md bg-base-300 border-base-100"
    >
      <Share2 class="text-base-content-100" size={20} />
      <h1 class="font-bold tracking-tight">
        Graph <span class='text-base-content-200/50'>Visualizer</span
        >
      </h1>
    </div>
    <div
      class="p-2 rounded-md border text-xs bg-base-300/80"
    >
      Monitoring {nodes.length} modules and {edges.length} dependencies
    </div>
  </div>

  <div class="w-full h-full">
    <SvelteFlow {nodes} {edges} fitView>
      <Background class="bg-base-300!" gap={24} />
      <Controls
        class="bg-base-100 border"
      />
    </SvelteFlow>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    overflow: hidden;
  }
</style>
