<script lang="ts">
  import { onMount, createEventDispatcher, afterUpdate } from "svelte";
  import { writable } from 'svelte/store';
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
  import {
    SvelteFlow,
    Background,
    Controls,
    Panel,
    Handle,
    SvelteFlowProvider
  } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';
  
  import type { Interface, CombinedDevice } from "./types";
  import type { Node, Edge, NodeTypes } from '@xyflow/svelte';
  
  // Import custom node components
  import InterfaceNode from './InterfaceNode.svelte';
  import DeviceNode from './DeviceNode.svelte';

  export let interfaces: Interface[] = [];
  export let devices: CombinedDevice[] = [];
  export let onElementSelect: (element: Interface | CombinedDevice, type: 'interface' | 'device') => void = () => {};

  // Create dispatch function
  const dispatch = createEventDispatcher<{
    elementSelect: { element: Interface | CombinedDevice, type: 'interface' | 'device' }
  }>();
  
  // Direct handler function to ensure element selection works reliably
  // This is called through multiple paths to maximize the chances of event handling success:
  // 1. Direct prop callback (onElementSelect)
  // 2. Svelte event dispatch system
  // 3. DOM CustomEvent bubbling
  // The goal is to ensure clicks always register regardless of render state
  function handleElementSelectDirect(element: Interface | CombinedDevice, type: 'interface' | 'device') {
    console.log('Direct element select in TopologyMap:', type, element);
    
    // Call the prop function if provided
    if (onElementSelect) {
      onElementSelect(element, type);
    }
    
    // Also dispatch the event for backward compatibility
    dispatch('elementSelect', { element, type });
  }
  
  // Just log when interfaces change
  $: if (interfaces.length > 0) {
    console.log(`Interfaces updated: ${interfaces.length} interfaces available`);
  }
  
  // Helper functions for the direct interface buttons
  function getInterfaceColor(iface: Interface): string {
    switch (iface.status?.toLowerCase()) {
      case 'up': return '#4CAF50'; // Green
      case 'down': return '#F44336'; // Red
      case 'no carrier': return '#FFA000'; // Amber
      default: return '#9E9E9E'; // Gray
    }
  }
  
  // Helper for device colors
  function getDeviceColor(device: CombinedDevice): string {
    if (device.permanent) return '#2196F3'; // Blue
    if (device.expired) return '#9E9E9E'; // Gray
    return '#673AB7'; // Purple
  }

  // Flow stores
  const nodes = writable<Node[]>([]);
  const edges = writable<Edge[]>([]);
  
  // Define node types - only using the predefined components for now
  const nodeTypes = {
    interface: InterfaceNode,
    device: DeviceNode
  };

  // Prepare data for visualization
  function prepareFlowData() {
    const flowNodes: Node[] = [];
    const flowEdges: Edge[] = [];
    
    // Debug log of interfaces and devices
    console.log('Preparing flow data with:', interfaces.length, 'interfaces,', devices.length, 'devices');
    
    // Filter interfaces
    const filteredInterfaces = interfaces.filter(iface => 
      iface.identifier || iface.is_physical
    );
    
    // We'll handle the firewall box separately instead of as a node
    
    // Calculate layout parameters
    const verticalSpacing = 160; // Space between interface nodes (moderate increase)
    const startY = 100;
    const interfaceX = 150; // Fixed X position for all interfaces
    const deviceRowWidth = 160; // Width between columns of devices (increased)
    
    // Calculate extra spacing between different interface groups
    // This will create a visual gap between devices from different interfaces
    const interfaceGroupGap = 40; // Extra space between interface groups
    
    // Add interface nodes in a vertical column with extra spacing between groups
    filteredInterfaces.forEach((iface, index) => {
      // Add extra gap if this isn't the first interface
      const extraSpace = index > 0 ? interfaceGroupGap : 0;
      const y = startY + (index * verticalSpacing) + (index * extraSpace);
      
      flowNodes.push({
        id: `interface-${iface.device}`,
        type: 'interface',
        position: { x: interfaceX, y },
        data: {
          label: iface.description || iface.device,
          interfaceData: iface
        }
      });
    });
    
    // For each interface, track connected devices to organize them
    const interfaceDeviceMap = new Map();
    
    // Group devices by interface
    devices.forEach((device) => {
      if (!interfaceDeviceMap.has(device.intf)) {
        interfaceDeviceMap.set(device.intf, []);
      }
      interfaceDeviceMap.get(device.intf).push(device);
    });
    
    // Debug log for device mapping
    console.log('Interface to device mapping:', 
      Array.from(interfaceDeviceMap.entries())
        .map(([intf, devs]) => `${intf}: ${devs.length} devices`)
    );
    
    // Add device nodes connected to interfaces
    filteredInterfaces.forEach((iface) => {
      const sourceNode = flowNodes.find(node => 
        node.type === 'interface' && 
        node.data.interfaceData.device === iface.device
      );
      
      if (sourceNode) {
        const connectedDevices = interfaceDeviceMap.get(iface.device) || [];
        const deviceCount = connectedDevices.length;
        
        // Fan out devices in horizontal rows to the right of each interface
        connectedDevices.forEach((device, idx) => {
          const deviceId = `device-${device.mac}`;
          const devicesPerColumn = Math.min(4, deviceCount); // Max 4 devices per column
          const columnNumber = Math.floor(idx / devicesPerColumn);
          const positionInColumn = idx % devicesPerColumn;
          
          // Calculate device position
          const deviceX = sourceNode.position.x + 120 + (columnNumber * deviceRowWidth);
          const columnStartY = sourceNode.position.y - ((devicesPerColumn - 1) * 55) / 2;
          const deviceY = columnStartY + positionInColumn * 55;
          
          flowNodes.push({
            id: deviceId,
            type: 'device',
            position: { x: deviceX, y: deviceY },
            data: {
              label: device.hostname || device.mac,
              deviceData: device
            }
          });
          
          flowEdges.push({
            id: `${sourceNode.id}-${deviceId}`,
            source: sourceNode.id,
            target: deviceId,
            animated: false,
            style: { stroke: '#999' }
          });
        });
      }
    });
    
    nodes.set(flowNodes);
    edges.set(flowEdges);
  }

  // Event handlers
  function handleNodeClick(event) {
    // Click events will be handled by custom node components
    if (!event || !event.detail || !event.detail.node) {
      console.log('Invalid node click event');
      return;
    }
    
    console.log('Node clicked:', event.detail.node.id);
    
    try {
      // Get the node data to manually forward the event if needed
      const clickedNode = event.detail.node;
      if (clickedNode && clickedNode.type) {
        console.log('Node click forwarded for:', clickedNode.type);
        
        // DIRECT EVENT DISPATCH - Bypass custom event system
        if (clickedNode.type === 'device' && clickedNode.data && clickedNode.data.deviceData) {
          console.log('DIRECT DISPATCH for device:', clickedNode.data.deviceData);
          dispatch('elementSelect', {
            element: clickedNode.data.deviceData,
            type: 'device'
          });
        } else if (clickedNode.type === 'interface' && clickedNode.data && clickedNode.data.interfaceData) {
          console.log('DIRECT DISPATCH for interface:', clickedNode.data.interfaceData);
          dispatch('elementSelect', {
            element: clickedNode.data.interfaceData,
            type: 'interface'
          });
        }
      }
    } catch (error) {
      console.error('Error handling node click:', error);
    }
  }
  
  function handleNodeEvent(event) {
    // Forward the event to parent component with the correct structure
    console.log('Node selection event in TopologyMap:', event.detail);
    
    if (event && event.detail) {
      console.log('Dispatching elementSelect from TopologyMap (namespaced):', event.detail);
      // Forward the event to parent using elementSelect
      dispatch('elementSelect', event.detail);
    }
  }
  
  // Handle regular select events from nodes
  function handleSelectEvent(event) {
    if (event && event.detail) {
      console.log('Regular select event in TopologyMap:', event.detail);
      console.log('Dispatching elementSelect from TopologyMap (regular):', event.detail);
      dispatch('elementSelect', event.detail);
    }
  }

  // Handle data changes
  $: if (interfaces.length > 0 || devices.length > 0) {
    prepareFlowData();
  }
  
  // Layout settings
  const fitViewOptions = { 
    padding: 0.2,  // Add some padding around the nodes
    includeHiddenNodes: false,  // Only include visible nodes
    minZoom: 0.4,  // Don't zoom out too far
    maxZoom: 2  // Don't zoom in too close
  };
  let reactFlowInstance;
  
  onMount(() => {
    prepareFlowData();
  });
  
  // Handle flow initialization
  function handleInit(event) {
    console.log('SvelteFlow initialized');
    reactFlowInstance = event.detail;
    
    // Give a slight delay to ensure nodes are properly rendered before fitting
    setTimeout(() => {
      if (reactFlowInstance && interfaces.length > 0) {
        console.log('Auto-fitting view on initialization');
        reactFlowInstance.fitView(fitViewOptions);
      }
    }, 200);  // Increased delay for more reliable rendering
  }
  
  // Refit view when data changes
  $: if ((interfaces.length > 0 || devices.length > 0) && reactFlowInstance) {
    setTimeout(() => {
      console.log('Auto-fitting view after data change');
      reactFlowInstance.fitView(fitViewOptions);
    }, 300);  // Increased delay for more reliable rendering
  }
  
  // Function to manually fit view - can be called from elsewhere if needed
  function manualFitView() {
    if (reactFlowInstance) {
      reactFlowInstance.fitView(fitViewOptions);
    }
  }
</script>

<div class="w-full h-full min-h-[600px] md:min-h-[700px] lg:min-h-[800px] xl:min-h-[900px] relative border border-base-300 rounded-md overflow-hidden bg-base-100">
  <!-- Legend/Key panel - moved to top with icons -->
  <div class="absolute top-2 left-2 z-10">
    <div class="dropdown dropdown-bottom">
      <label tabindex="0" class="btn btn-sm bg-base-100 shadow-sm gap-1 hover:bg-base-200">
        <svg class="w-4 h-4" viewBox="0 0 24 24">
          <path fill="currentColor" d="M21,3H3C2,3 1,4 1,5V19A2,2 0 0,0 3,21H21C22,21 23,20 23,19V5C23,4 22,3 21,3M5,17L8.5,12.5L11,15.5L14.5,11L19,17H5Z" />
        </svg>
        Legend
      </label>
      <div tabindex="0" class="dropdown-content menu p-3 shadow-lg bg-base-100 rounded-lg w-80 mt-1 z-50">
        <div class="space-y-4">
          <!-- Interface Status Section -->
          <div>
            <div class="text-sm font-semibold mb-2 pb-1 border-b">Interface Status</div>
            <div class="grid grid-cols-3 gap-2">
              <div class="flex flex-col items-center gap-1">
                <div class="w-6 h-6 rounded-full bg-[#4CAF50]"></div>
                <span class="text-xs text-center">Up</span>
              </div>
              <div class="flex flex-col items-center gap-1">
                <div class="w-6 h-6 rounded-full bg-[#F44336]"></div>
                <span class="text-xs text-center">Down</span>
              </div>
              <div class="flex flex-col items-center gap-1">
                <div class="w-6 h-6 rounded-full bg-[#FFA000]"></div>
                <span class="text-xs text-center">No Carrier</span>
              </div>
            </div>
          </div>
          
          <!-- Interface Types Section -->
          <div>
            <div class="text-sm font-semibold mb-2 pb-1 border-b">Interface Types</div>
            <div class="grid grid-cols-2 gap-3">
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-base-content/20 flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="currentColor" d="M7,15H9V18H13V15H15V18H16A1,1 0 0,0 17,17V7A1,1 0 0,0 16,6H8A1,1 0 0,0 7,7V17A1,1 0 0,0 8,18H9V15M10,7H14V9H10V7M10,10H14V12H10V10M8,13H16V14H8V13Z" />
                  </svg>
                </div>
                <span class="text-xs">Physical</span>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-base-content/20 flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="currentColor" d="M4,1C2.89,1 2,1.89 2,3V7C2,8.11 2.89,9 4,9H1V11H13V9H10C11.11,9 12,8.11 12,7V3C12,1.89 11.11,1 10,1H4M4,3H10V7H4V3M3,13V18L3,20H10V18H5V13H3M14,13C12.89,13 12,13.89 12,15V19C12,20.11 12.89,21 14,21H20C21.11,21 22,20.11 22,19V15C22,13.89 21.11,13 20,13H14M14,15H20V19H14V15Z" />
                  </svg>
                </div>
                <span class="text-xs">VLAN</span>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-base-content/20 flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="currentColor" d="M12,21L15.6,16.2C14.6,15.45 13.35,15 12,15C10.65,15 9.4,15.45 8.4,16.2L12,21M12,3C7.95,3 4.21,4.34 1.2,6.6L3,9C5.5,7.12 8.62,6 12,6C15.38,6 18.5,7.12 21,9L22.8,6.6C19.79,4.34 16.05,3 12,3M12,9C9.3,9 6.81,9.89 4.8,11.4L6.6,13.8C8.1,12.67 9.97,12 12,12C14.03,12 15.9,12.67 17.4,13.8L19.2,11.4C17.19,9.89 14.7,9 12,9Z" />
                  </svg>
                </div>
                <span class="text-xs">Wireless</span>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-base-content/20 flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="currentColor" d="M10,2C8.89,2 8,2.89 8,4V7C8,8.11 8.89,9 10,9H11V11H2V13H6V15H5C3.89,15 3,15.89 3,17V20C3,21.11 3.89,22 5,22H9C10.11,22 11,21.11 11,20V17C11,15.89 10.11,15 9,15H8V13H16V15H15C13.89,15 13,15.89 13,17V20C13,21.11 13.89,22 15,22H19C20.11,22 21,21.11 21,20V17C21,15.89 20.11,15 19,15H18V13H22V11H13V9H14C15.11,9 16,8.11 16,7V4C16,2.89 15.11,2 14,2H10M10,4H14V7H10V4M5,17H9V20H5V17M15,17H19V20H15V17Z" />
                  </svg>
                </div>
                <span class="text-xs">Virtual</span>
              </div>
            </div>
          </div>
          
          <!-- Device Types Section -->
          <div>
            <div class="text-sm font-semibold mb-2 pb-1 border-b">Device Types</div>
            <div class="grid grid-cols-2 gap-3">
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-[#2196F3] flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="white" d="M11,2A8,8 0 0,0 3,10V11H11V13H3V14A8,8 0 0,0 11,22H13A8,8 0 0,0 21,14V13H13V11H21V10A8,8 0 0,0 13,2H11M11,4H13A6,6 0 0,1 19,10H5A6,6 0 0,1 11,4Z" />
                  </svg>
                </div>
                <span class="text-xs">Router</span>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-[#673AB7] flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="white" d="M20,18C21.1,18 22,17.1 22,16V6C22,4.89 21.1,4 20,4H4C2.89,4 2,4.89 2,6V16A2,2 0 0,0 4,18H0V20H24V18H20M4,16V6H20V16H4Z" />
                  </svg>
                </div>
                <span class="text-xs">Computer</span>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-[#673AB7] flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="white" d="M17,19H7V5H17M17,1H7C5.89,1 5,1.89 5,3V21A2,2 0 0,0 7,23H17A2,2 0 0,0 19,21V3C19,1.89 18.1,1 17,1Z" />
                  </svg>
                </div>
                <span class="text-xs">Mobile Device</span>
              </div>
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-full bg-[#673AB7] flex items-center justify-center">
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="white" d="M4,1H20A1,1 0 0,1 21,2V6A1,1 0 0,1 20,7H4A1,1 0 0,1 3,6V2A1,1 0 0,1 4,1M4,9H20A1,1 0 0,1 21,10V14A1,1 0 0,1 20,15H4A1,1 0 0,1 3,14V10A1,1 0 0,1 4,9M4,17H20A1,1 0 0,1 21,18V22A1,1 0 0,1 20,23H4A1,1 0 0,1 3,22V18A1,1 0 0,1 4,17M9,5H10V3H9V5M9,13H10V11H9V13M9,21H10V19H9V21M5,3V5H7V3H5M5,11V13H7V11H5M5,19V21H7V19H5Z" />
                  </svg>
                </div>
                <span class="text-xs">Server</span>
              </div>
            </div>
          </div>
          
          <!-- Device Status Section -->
          <div>
            <div class="text-sm font-semibold mb-2 pb-1 border-b">Device Status</div>
            <div class="grid grid-cols-3 gap-2">
              <div class="flex flex-col items-center gap-1">
                <div class="w-6 h-6 rounded-full bg-[#2196F3]"></div>
                <span class="text-xs text-center">Permanent</span>
              </div>
              <div class="flex flex-col items-center gap-1">
                <div class="w-6 h-6 rounded-full bg-[#673AB7]"></div>
                <span class="text-xs text-center">Dynamic</span>
              </div>
              <div class="flex flex-col items-center gap-1">
                <div class="w-6 h-6 rounded-full bg-[#9E9E9E]"></div>
                <span class="text-xs text-center">Expired</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
  <SvelteFlowProvider>
    {#if interfaces.length === 0 && devices.length === 0}
      <div class="absolute inset-0 flex items-center justify-center pointer-events-none z-20">
        <div class="text-center p-6 bg-base-100/90 rounded-lg shadow-md">
          <svg class="w-16 h-16 mx-auto text-base-content/30 animate-pulse" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiLanDisconnect} />
          </svg>
          <p class="mt-4 text-base-content/70 font-medium">
            No Network Data Available
          </p>
        </div>
      </div>
    {:else}
      <div class="relative w-full h-full"
        on:nodeClick={(event) => {
          console.log('nodeClick custom event bubbled to container:', event);
          if (event.detail && event.detail.element) {
            console.log('Processing bubbled nodeClick event');
            handleElementSelectDirect(event.detail.element, event.detail.type);
          }
        }}
       >
        <!-- Firewall container - rendered as a div overlay -->
        {#if interfaces.length > 0}
          <div 
            class="absolute pointer-events-none z-0 rounded-xl border-2 border-dashed border-[rgba(41,98,255,0.4)] bg-[rgba(41,98,255,0.1)]"
            style="left: 50px; top: 30px; width: 280px; height: {(interfaces.length * 160) + (interfaces.length * 40) + 100}px;"
          >
            <div class="absolute top-3 left-3 flex items-center gap-2">
              <svg class="w-5 h-5 text-[rgba(41,98,255,0.8)]" viewBox="0 0 24 24">
                <path fill="currentColor" d="M8,2C6.34,2 5,3.34 5,5V16H6V5C6,3.9 6.9,3 8,3H16C17.1,3 18,3.9 18,5V16H19V5C19,3.34 17.66,2 16,2H8M3,7V16H4V7H3M20,7V16H21V7H20M7,18C5.9,18 5,18.9 5,20V22H19V20C19,18.9 18.1,18 17,18H7Z" />
              </svg>
              <span class="text-xs font-medium text-[rgba(41,98,255,0.8)]">OPNsense Firewall</span>
            </div>
          </div>
        {/if}
        
        <!-- Invisible clickable areas over the nodes for guaranteed click handling -->
        <!-- We're intentionally not displaying these overlays, they're just for capturing clicks -->
        <div class="absolute top-0 left-0 w-full h-full pointer-events-none" style="z-index: 40;">
          <!-- This will be empty but provides the container for our click handlers -->
        </div>
        
        <SvelteFlow
          {nodes}
          {edges}
          {nodeTypes}
          fitView={true}
          {fitViewOptions}
          on:init={handleInit}
          on:nodeClick={handleNodeClick}
          on:select={handleSelectEvent}
          on:interface:select={handleNodeEvent}
          on:device:select={handleNodeEvent}
          elevateEdgesOnSelect={true}
          nodesDraggable={true}
          nodesConnectable={false}
          zoomOnScroll={true}
          panOnScroll={true}
          selectionOnDrag={false}
          defaultViewport={{ x: 0, y: 0, zoom: 1 }}
          minZoom={0.2}
          maxZoom={2}
          on:nodeClick={(event) => {
            console.log('SvelteFlow nodeClick event:', event);
            // Process the direct click from SvelteFlow
            if (event && event.detail && event.detail.node) {
              const node = event.detail.node;
              if (node.type === 'device' && node.data && node.data.deviceData) {
                handleElementSelectDirect(node.data.deviceData, 'device');
              } else if (node.type === 'interface' && node.data && node.data.interfaceData) {
                handleElementSelectDirect(node.data.interfaceData, 'interface');
              }
            }
          }}
        >
          <Background variant="dots" gap={20} size={1} />
          <Controls />
        </SvelteFlow>
      </div>
    {/if}
  </SvelteFlowProvider>
</div>

<style>
  :global(.svelte-flow__minimap) {
    right: 10px;
    bottom: 10px;
  }
  
  :global(.svelte-flow__controls) {
    bottom: 10px;
    right: 10px; /* Changed from left to right */
    box-shadow: 0 0 2px rgba(0, 0, 0, 0.15);
  }
  
  :global(.svelte-flow__edge-path) {
    stroke: #999;
    stroke-width: 1.5;
  }
  
  :global(.svelte-flow__node) {
    padding: 0;
    pointer-events: all !important;
    cursor: pointer;
  }
  
  :global(.svelte-flow__handle) {
    pointer-events: none !important;
  }
  
  :global(.svelte-flow__edge) {
    pointer-events: none !important;
  }
</style>