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
    
    // Calculate layout parameters
    const verticalSpacing = 200;
    const startY = 100;
    const interfaceX = 180; 
    const deviceRowWidth = 180; 
    
    // Calculate extra spacing between different interface groups
    // This will create a visual gap between devices from different interfaces
    const interfaceGroupGap = 60; // Increased extra space between interface groups
    
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
          const deviceX = sourceNode.position.x + 150 + (columnNumber * deviceRowWidth);
          const columnStartY = sourceNode.position.y - ((devicesPerColumn - 1) * 70) / 2;
          const deviceY = columnStartY + positionInColumn * 70;
          
          flowNodes.push({
            id: deviceId,
            type: 'device',
            position: { x: deviceX, y: deviceY },
            data: {
              label: device.hostname || device.mac,
              deviceData: device
            }
          });
          
          // Style edges differently based on device type/status
          const isExpired = device.expired;
          const isPermanent = device.permanent;
          
          flowEdges.push({
            id: `${sourceNode.id}-${deviceId}`,
            source: sourceNode.id,
            target: deviceId,
            animated: !isExpired,
            style: { 
              stroke: isExpired ? 'rgba(158, 158, 158, 0.5)' : 
                     isPermanent ? 'rgba(33, 150, 243, 0.7)' : 
                     'rgba(103, 58, 183, 0.7)',
              strokeWidth: isPermanent ? 2 : 1.5,
              strokeDasharray: isExpired ? '5,5' : undefined
            },
            data: { deviceStatus: isExpired ? 'expired' : isPermanent ? 'permanent' : 'active' }
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
    padding: 0.1,  // Reduce padding to use more space
    includeHiddenNodes: false,  // Only include visible nodes
    minZoom: 0.3,  // Allow zooming out further
    maxZoom: 2.5  // Allow zooming in closer
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

<div class="w-full h-full min-h-[600px] md:min-h-[700px] lg:min-h-[800px] xl:min-h-[900px] relative overflow-hidden bg-base-100">
  <!-- Legend/Key panel - moved to top with icons -->
  <div class="absolute top-2 left-2 z-10">
    <div class="dropdown dropdown-bottom">
      <label tabindex="0" class="btn btn-sm bg-base-100 shadow-sm gap-1 hover:bg-base-200">
        <svg class="w-4 h-4" viewBox="0 0 24 24">
          <path fill="currentColor" d="M21,3H3C2,3 1,4 1,5V19A2,2 0 0,0 3,21H21C22,21 23,20 23,19V5C23,4 22,3 21,3M5,17L8.5,12.5L11,15.5L14.5,11L19,17H5Z" />
        </svg>
        Legend
      </label>
      <div tabindex="0" class="dropdown-content menu p-3 shadow-lg bg-base-100 rounded-lg w-[min(90vw,320px)] md:w-80 mt-1 z-50">
        <div class="space-y-4">
          <!-- Interface Status Section -->
          <div>
            <div class="text-sm font-semibold mb-2 pb-1 border-b">Interface Status</div>
            <div class="grid grid-cols-3 gap-1">
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
          
          <!-- Interface & Device layout with sections side by side on mobile -->
          <div class="md:space-y-4 flex flex-col md:block">
            <!-- On mobile, display as two columns -->
            <div class="grid grid-cols-2 gap-3 md:gap-0 md:block">
              <!-- Interface Types Section -->
              <div class="md:mb-4">
                <div class="text-sm font-semibold mb-2 pb-1 border-b">Interface Types</div>
                <div class="grid grid-cols-1 gap-2">
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
                <div class="grid grid-cols-1 gap-2">
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
        <div class="text-center p-8 bg-gradient-to-b from-base-100/95 to-base-200/95 rounded-2xl shadow-lg backdrop-blur-sm border border-base-300/50 max-w-sm">
          <div class="relative mb-3">
            <svg class="w-20 h-20 mx-auto text-base-content/30 animate-pulse" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiLanDisconnect} />
            </svg>
            <!-- Add a decorative circular background -->
            <div class="absolute inset-0 bg-gradient-to-tr from-base-200 to-base-100 rounded-full opacity-20 animate-ping" style="animation-duration: 3s;"></div>
          </div>
          <p class="text-xl text-base-content/80 font-semibold mb-2">
            No Network Data Available
          </p>
          <p class="text-sm text-base-content/50 max-w-xs mx-auto">
            Use the refresh button above to load network topology information from your OPNsense firewall.
          </p>
          <div class="mt-5 flex justify-center">
            <div class="badge badge-outline badge-lg gap-1">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiRefresh} />
              </svg>
              <span>Refresh Required</span>
            </div>
          </div>
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
        <!-- Firewall container - rendered as a div overlay with enhanced styling -->
        {#if interfaces.length > 0}
          <div 
            class="absolute pointer-events-none z-0 rounded-2xl firewall-container"
            style="left: 50px; top: 30px; width: 330px; height: {(interfaces.length * 200) + (interfaces.length * 60) + 100}px;"
          >
            <!-- Add a subtle grid pattern -->
            <div class="absolute inset-0 opacity-20" 
                style="background-image: 
                  linear-gradient(rgba(33, 150, 243, 0.1) 1px, transparent 1px),
                  linear-gradient(90deg, rgba(33, 150, 243, 0.1) 1px, transparent 1px);
                background-size: 20px 20px;
                border-radius: inherit;"></div>
              
            <!-- Firewall header -->
            <div class="absolute top-0 left-0 right-0 p-3 flex items-center gap-2 border-b border-[rgba(33,150,243,0.15)] bg-gradient-to-r from-[rgba(33,150,243,0.15)] to-transparent rounded-t-2xl">
              <div class="w-8 h-8 flex items-center justify-center rounded-full bg-[rgba(33,150,243,0.15)] shadow-inner">
                <svg class="w-5 h-5 text-[rgba(33,150,243,0.8)]" viewBox="0 0 24 24">
                  <path fill="currentColor" d="M8,2C6.34,2 5,3.34 5,5V16H6V5C6,3.9 6.9,3 8,3H16C17.1,3 18,3.9 18,5V16H19V5C19,3.34 17.66,2 16,2H8M3,7V16H4V7H3M20,7V16H21V7H20M7,18C5.9,18 5,18.9 5,20V22H19V20C19,18.9 18.1,18 17,18H7Z" />
                </svg>
              </div>
              <div>
                <div class="font-medium text-[rgba(33,150,243,0.9)]">OPNsense Firewall</div>
                <div class="text-xs text-[rgba(33,150,243,0.6)]">{interfaces.length} interfaces connected</div>
              </div>
            </div>
            
            <!-- Add subtle decorative elements -->
            <div class="absolute bottom-3 right-3 opacity-20">
              <svg class="w-20 h-20 text-[rgba(33,150,243,0.3)]" viewBox="0 0 24 24">
                <path fill="currentColor" d="M12,1L3,5V11C3,16.55 6.84,21.74 12,23C17.16,21.74 21,16.55 21,11V5L12,1M12,5A3,3 0 0,1 15,8A3,3 0 0,1 12,11A3,3 0 0,1 9,8A3,3 0 0,1 12,5M17.13,17C15.92,18.85 14.11,20.24 12,20.92C9.89,20.24 8.08,18.85 6.87,17C6.53,16.5 6.24,16 6,15.47C6,13.82 8.71,12.47 12,12.47C15.29,12.47 18,13.79 18,15.47C17.76,16 17.47,16.5 17.13,17Z" />
              </svg>
            </div>
          </div>
        {/if}
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
          minZoom={0.3}
          maxZoom={2.5}
          proOptions={{ hideAttribution: true }}
          on:nodeClick={(event) => {
            console.log('SvelteFlow nodeClick event:', event);
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
          <Background 
            variant="dots" 
            gap={24} 
            size={1} 
            color="rgba(0, 0, 0, 0.05)"
          />
          <Controls 
            showFitView={true}
            fitViewOptions={fitViewOptions}
            position="bottom-left"
          />
        </SvelteFlow>
      </div>
    {/if}
  </SvelteFlowProvider>
</div>

<style>
  :global(.svelte-flow__minimap) {
    right: 10px;
    bottom: 10px;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
  
  :global(.svelte-flow__controls) {
    bottom: 50px;
    left: 10px;
    background-color: transparent;
    box-shadow: none;
    border-radius: 0;
    padding: 0;
    transform: scale(0.85);
    transform-origin: bottom left;
  }
  
  :global(.svelte-flow__controls button) {
    background-color: rgba(255, 255, 255, 0.6);
    border-radius: 50%;
    transition: all 0.2s ease;
    border: none;
    width: 28px;
    height: 28px;
    margin: 3px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }
  
  :global(.svelte-flow__controls button:hover) {
    background-color: rgba(255, 255, 255, 0.9);
    transform: translateY(-1px);
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.15);
  }
  
  :global(.svelte-flow__edge-path) {
    stroke: rgba(75, 85, 99, 0.6);
    stroke-width: 1.75;
    transition: stroke 0.3s ease, stroke-width 0.3s ease;
  }
  
  :global(.svelte-flow__edge.selected .svelte-flow__edge-path) {
    stroke: rgba(37, 99, 235, 0.8);
    stroke-width: 2.5;
    filter: drop-shadow(0 0 8px rgba(37, 99, 235, 0.5));
  }
  
  :global(.svelte-flow__edge.animated .svelte-flow__edge-path) {
    stroke-dasharray: 5 5;
    animation: flowAnimation 1s infinite linear;
  }
  
  @keyframes flowAnimation {
    from {
      stroke-dashoffset: 10;
    }
    to {
      stroke-dashoffset: 0;
    }
  }
  
  :global(.svelte-flow__edge-text) {
    font-weight: 600;
    fill: #4B5563;
    text-shadow: 0 1px 2px rgba(255, 255, 255, 0.8);
  }
  
  :global(.svelte-flow__node) {
    padding: 0;
    pointer-events: all !important;
    cursor: pointer;
    transition: transform 0.3s ease;
  }
  
  :global(.svelte-flow__handle) {
    pointer-events: none !important;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 2px solid white;
    background-color: rgba(37, 99, 235, 0.9);
    opacity: 0.8;
  }
  
  :global(.svelte-flow__handle.target) {
    top: -3px;
  }
  
  :global(.svelte-flow__handle.source) {
    bottom: -3px;
  }
  
  :global(.svelte-flow__edge) {
    pointer-events: none !important;
  }
  
  :global(.svelte-flow__background) {
    background-color: rgba(243, 244, 246, 0.8);
  }
  
  /* Hide minimap */
  :global(.svelte-flow__minimap) {
    display: none;
  }
  
  /* Background adjustments for dark theme */
  :global([data-theme="dark"] .svelte-flow__background) {
    background-color: rgba(30, 41, 59, 0.8);
  }
  
  :global([data-theme="dark"] .svelte-flow__background-pattern) {
    color: rgba(255, 255, 255, 0.05) !important;
  }
  
  :global([data-theme="dark"] .svelte-flow__controls button) {
    background-color: rgba(30, 41, 59, 0.8);
    color: rgba(255, 255, 255, 0.9);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  :global([data-theme="dark"] .svelte-flow__controls button:hover) {
    background-color: rgba(51, 65, 85, 0.9);
    color: rgba(255, 255, 255, 1);
  }
  
  :global([data-theme="dark"] .svelte-flow__edge-path) {
    stroke: rgba(255, 255, 255, 0.5);
  }
  
  :global([data-theme="dark"] .svelte-flow__edge.selected .svelte-flow__edge-path) {
    stroke: rgba(96, 165, 250, 0.8);
    filter: drop-shadow(0 0 8px rgba(96, 165, 250, 0.5));
  }
  
  /* Add style for the transparent topology container */
  .topology-container {
    position: relative;
    width: 100%;
    height: 100%;
    background-image: linear-gradient(
      to bottom right,
      rgba(243, 244, 246, 0.5),
      rgba(249, 250, 251, 0.8)
    );
    border-radius: 8px;
    overflow: hidden;
  }
  
  /* Dark theme version of the topology container */
  :global([data-theme="dark"]) .topology-container {
    background-image: linear-gradient(
      to bottom right,
      rgba(15, 23, 42, 0.5),
      rgba(30, 41, 59, 0.8)
    );
  }
  
  /* Firewall container styles */
  .firewall-container {
    background: linear-gradient(135deg, rgba(33, 37, 41, 0.03) 0%, rgba(33, 150, 243, 0.08) 100%);
    box-shadow: 0 0 25px rgba(33, 150, 243, 0.1), inset 0 0 5px rgba(255, 255, 255, 0.5);
    backdrop-filter: blur(3px);
    border: 1px solid rgba(33, 150, 243, 0.2);
  }
  
  /* Dark theme adjustments for firewall container */
  :global([data-theme="dark"]) .firewall-container {
    background: linear-gradient(135deg, rgba(30, 41, 59, 0.2) 0%, rgba(59, 130, 246, 0.1) 100%);
    box-shadow: 0 0 25px rgba(59, 130, 246, 0.1), inset 0 0 5px rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.2);
  }
  
  /* Control styling for fit-view button */
  :global(.svelte-flow__controls) {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
</style>