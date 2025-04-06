<script lang="ts">
  import { onMount, createEventDispatcher, afterUpdate } from "svelte";
  import {
    mdiEthernet,
    mdiLan,
    mdiLanConnect,
    mdiAccessPoint,
    mdiDevices,
    mdiLaptop,
    mdiCellphone,
    mdiRouter,
    mdiServerNetwork,
    mdiDesktopTower,
    mdiLanDisconnect
  } from "@mdi/js";
  import * as d3 from "d3";
  import type { Interface, CombinedDevice, TopologyNode, TopologyLink, TopologyData } from "./types";

  // Global declarations for D3 references
  declare global {
    interface Window {
      topologyContainer: any;
      topologyZoom: any;
      currentTopologyZoom: any;
    }
  }

  export let interfaces: Interface[] = [];
  export let devices: CombinedDevice[] = [];

  let containerElement: HTMLDivElement;
  let svgElement: SVGElement;
  let width = 1000;
  let height = 600;
  let topologyData: TopologyData = { nodes: [], links: [] };
  let simulation: d3.Simulation<TopologyNode, TopologyLink> | null = null;
  let isInitialized = false;

  const dispatch = createEventDispatcher<{
    elementSelect: { element: Interface | CombinedDevice, type: 'interface' | 'device' }
  }>();

  // Prepare data for visualization
  function prepareTopologyData(): TopologyData {
    const nodes: TopologyNode[] = [];
    const links: TopologyLink[] = [];
    
    // Add interface nodes
    interfaces.forEach(iface => {
      if (iface.identifier || iface.is_physical) {
        nodes.push({
          id: `interface-${iface.device}`,
          type: 'interface',
          label: iface.description || iface.device,
          data: iface
        });
      }
    });
    
    // Add device nodes and create links
    devices.forEach(device => {
      const interfaceNode = nodes.find(node => 
        node.type === 'interface' && node.data.device === device.intf
      );
      
      if (interfaceNode) {
        const deviceId = `device-${device.mac}`;
        
        nodes.push({
          id: deviceId,
          type: 'device',
          label: device.hostname || device.mac,
          data: device
        });
        
        links.push({
          source: interfaceNode.id,
          target: deviceId,
          id: `${interfaceNode.id}-${deviceId}`
        });
      }
    });
    
    return { nodes, links };
  }

  // Initialize the D3 visualization
  function initializeVisualization() {
    if (!containerElement || !svgElement) return;

    // Update container dimensions
    const containerRect = containerElement.getBoundingClientRect();
    width = containerRect.width;
    height = containerRect.height;
    console.log(`Container size: ${width}x${height}`);

    // Prepare the data
    topologyData = prepareTopologyData();
    
    // Clear previous SVG content
    const svg = d3.select(svgElement);
    svg.selectAll("*").remove();
    
    // Set SVG attributes
    svg.attr("width", "100%")
       .attr("height", "100%")
       .attr("viewBox", [0, 0, width, height])
       .style("display", "block");
    
    // Create a group for content
    const g = svg.append("g");
    window.topologyContainer = g;
    
    // Set up force simulation
    simulation = d3.forceSimulation(topologyData.nodes)
      .force("link", d3.forceLink(topologyData.links).id((d: any) => d.id))
      .force("charge", d3.forceManyBody().strength(-400))
      .force("center", d3.forceCenter(width / 2, height / 2));
    
    // Define zoom behavior
    const zoom = d3.zoom()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        g.attr("transform", event.transform);
        window.currentTopologyZoom = event.transform;
      });
    
    // Apply zoom behavior
    svg.call(zoom as any);
    window.topologyZoom = zoom;
    
    // Render links
    const link = g.append("g")
      .attr("stroke", "#999")
      .attr("stroke-opacity", 0.6)
      .selectAll("line")
      .data(topologyData.links)
      .join("line")
      .attr("stroke-width", 1.5);
    
    // Render nodes
    const node = g.append("g")
      .attr("class", "nodes")
      .selectAll(".node")
      .data(topologyData.nodes)
      .join("g")
      .attr("class", "node")
      .attr("cursor", "pointer")
      .on("click", (event, d) => {
        event.stopPropagation();
        handleNodeClick(d);
      })
      .call(d3.drag()
        .on("start", dragstarted)
        .on("drag", dragged)
        .on("end", dragended) as any
      );
    
    // Node circles
    node.append("circle")
      .attr("r", d => d.type === 'interface' ? 20 : 15)
      .attr("fill", d => d.type === 'interface' ? getInterfaceColor(d.data as Interface) : getDeviceColor(d.data as CombinedDevice))
      .attr("stroke", "#fff")
      .attr("stroke-width", 1.5);
    
    // Node icons
    node.append("path")
      .attr("d", d => {
        if (d.type === 'interface') {
          const iface = d.data as Interface;
          if (iface.vlan_tag) return mdiLan;
          if (iface.is_physical) return mdiEthernet;
          return mdiLanConnect;
        } else {
          const device = d.data as CombinedDevice;
          const hostname = device.hostname?.toLowerCase() || '';
          
          if (hostname.includes('router')) return mdiRouter;
          if (hostname.includes('ap')) return mdiAccessPoint;
          if (hostname.includes('server')) return mdiServerNetwork;
          if (hostname.includes('phone')) return mdiCellphone;
          if (hostname.includes('laptop')) return mdiLaptop;
          if (hostname.includes('desktop')) return mdiDesktopTower;
          return mdiDevices;
        }
      })
      .attr("fill", "white")
      .attr("transform", d => d.type === 'interface' ? "translate(-10, -10) scale(0.8)" : "translate(-8, -8) scale(0.7)");
    
    // Node labels
    node.append("text")
      .attr("dy", d => d.type === 'interface' ? 32 : 25)
      .attr("text-anchor", "middle")
      .text(d => {
        const label = d.label;
        return label.length > 15 ? label.slice(0, 12) + '...' : label;
      })
      .attr("fill", "#333")
      .attr("font-size", "10px")
      .attr("pointer-events", "none");
    
    // Simulation tick handler
    simulation.on("tick", () => {
      link
        .attr("x1", d => (d.source as any).x)
        .attr("y1", d => (d.source as any).y)
        .attr("x2", d => (d.target as any).x)
        .attr("y2", d => (d.target as any).y);
      
      node.attr("transform", d => `translate(${d.x},${d.y})`);
    });
    
    // Drag functions
    function dragstarted(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }
    
    function dragged(event: any, d: any) {
      d.fx = event.x;
      d.fy = event.y;
    }
    
    function dragended(event: any, d: any) {
      if (!event.active) simulation.alphaTarget(0);
      d.fx = null;
      d.fy = null;
    }
    
    isInitialized = true;
  }

  function handleNodeClick(node: TopologyNode) {
    console.log("Node clicked:", node.id);
    dispatch('elementSelect', {
      element: node.data,
      type: node.type
    });
  }

  function getInterfaceColor(iface: Interface): string {
    switch (iface.status?.toLowerCase()) {
      case 'up': return '#4CAF50'; // Green
      case 'down': return '#F44336'; // Red
      case 'no carrier': return '#FFA000'; // Amber
      default: return '#9E9E9E'; // Gray
    }
  }

  function getDeviceColor(device: CombinedDevice): string {
    if (device.permanent) return '#2196F3'; // Blue
    if (device.expired) return '#9E9E9E'; // Gray
    return '#673AB7'; // Purple
  }

  // Lifecycle handlers
  onMount(() => {
    console.log("TopologyMap component mounted");
    setTimeout(initializeVisualization, 50);
    
    window.addEventListener('resize', handleResize);
    
    return () => {
      window.removeEventListener('resize', handleResize);
      if (simulation) simulation.stop();
    };
  });

  // Debounced resize handler
  let resizeTimeout: number | null = null;
  function handleResize() {
    if (resizeTimeout) clearTimeout(resizeTimeout);
    resizeTimeout = window.setTimeout(() => {
      console.log("Window resized, reinitializing visualization");
      isInitialized = false;
      if (simulation) simulation.stop();
      initializeVisualization();
    }, 300);
  }

  // Update when data changes
  $: if (interfaces && devices && containerElement && !isInitialized) {
    setTimeout(initializeVisualization, 50);
  }
</script>

<div
  class="w-full h-full min-h-[500px] relative border border-base-300 rounded-md overflow-hidden bg-base-100"
  bind:this={containerElement}
>
  <svg
    bind:this={svgElement}
    style="width: 100%; height: 100%; touch-action: none;"
  ></svg>

  {#if !isInitialized && topologyData.nodes.length === 0}
    <div class="absolute inset-0 flex items-center justify-center pointer-events-none z-20">
      <div class="text-center p-6 bg-base-100/90 rounded-lg shadow-md">
        <svg class="w-16 h-16 mx-auto text-base-content/30 animate-pulse" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiLanDisconnect} />
        </svg>
        <p class="mt-4 text-base-content/70 font-medium">
            {#if interfaces.length > 0 || devices.length > 0}
                Loading Topology...
            {:else}
                No Network Data Available
            {/if}
        </p>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Ensure pointer events work correctly */
  :global(.node) {
    pointer-events: all;
  }
  
  :global(.node:hover > circle) {
    stroke: var(--color-primary, #570df8);
    stroke-width: 3px;
  }
</style>