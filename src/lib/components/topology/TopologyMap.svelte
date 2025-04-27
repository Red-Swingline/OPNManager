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
  import LightweightDeviceNode from './LightweightDeviceNode.svelte';
  import LightweightInterfaceNode from './LightweightInterfaceNode.svelte';

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
    
    // Enhanced logging to debug
    if (!element) {
      console.error('handleElementSelectDirect called with null element');
      return;
    }
    
    // First try the direct callback
    try {
      if (onElementSelect) {
        console.log('Calling onElementSelect prop with:', type, element);
        onElementSelect(element, type);
      }
    } catch (error) {
      console.error('Error in onElementSelect callback:', error);
    }
    
    // Then try dispatching event
    try {
      console.log('Dispatching elementSelect event with:', { element, type });
      dispatch('elementSelect', { element, type });
    } catch (error) {
      console.error('Error dispatching elementSelect event:', error);
    }
    
    // Last resort - try updating parent directly
    console.log('Details provided to parent component through multiple channels');
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
  
  // Initialize nodes and edges with empty arrays to prevent undefined errors
  nodes.set([]);
  edges.set([]);
  
  // Detect if we're on a mobile device (iOS/Android) for minor optimizations
  const isMobileDevice = typeof window !== 'undefined' && 
    (/iPhone|iPad|iPod|Android/i.test(navigator.userAgent) || 
    (navigator.maxTouchPoints && navigator.maxTouchPoints > 2));
  
  // Use lightweight components for better performance on all devices
  const nodeTypes = {
    interface: LightweightInterfaceNode,
    device: LightweightDeviceNode
  };

  // Prepare data for visualization with incremental rendering for better performance
  function prepareFlowData() {
    // Start with just the interfaces for immediate feedback
    const flowNodes: Node[] = [];
    const flowEdges: Edge[] = [];
    
    console.log('Preparing flow data with:', interfaces.length, 'interfaces,', devices.length, 'devices');
    
    // Filter interfaces, handling CARP/HA setups better
    const filteredInterfaces = interfaces.filter(iface => {
      // Skip certain system interfaces that aren't useful in the visualization
      if (iface.device === 'lo0' || iface.device === 'pfsync0') {
        return false;
      }
      
      // Special handling for CARP interfaces in HA setups
      const isCarpInterface = 
        iface.device.includes('_vip') || 
        iface.device.startsWith('carp') ||
        (iface.description && iface.description.toLowerCase().includes('carp'));
      
      // Include CARP interfaces only if they're UP (they're the active interfaces in the HA pair)
      if (isCarpInterface) {
        return iface.status?.toLowerCase() === 'up';
      }
      
      // For normal interfaces: include physical, virtual, and interfaces with identifiers
      // Only filter out special system interfaces like lo0 and pfsync0 that we explicitly exclude above
      return true;
    });
    
    // Calculate layout parameters with more spacing to prevent overlap
    const verticalSpacing = 200;  // Base vertical space between interfaces
    const startY = 80;
    const interfaceX = 120; 
    const deviceRowWidth = 130;  // Consistent horizontal space across all platforms
    
    // Calculate extra spacing between different interface groups
    // This will create a visual gap between devices from different interfaces
    const interfaceGroupGap = 150; // Significantly increased spacing between interface clusters
    
    // Additional debug logging for layout adjustments
    console.log('Using enhanced interface spacing of', interfaceGroupGap, 'px between interface clusters');
    
    // Dynamic device limit based on total device count to improve visualization
    const totalDeviceCount = devices.length;
    // Reasonable limits for display based on device type
    // Use consistent device limits across all platforms for uniformity
    let deviceLimit = totalDeviceCount > 200 ? 12 :
                      totalDeviceCount > 100 ? 18 : 
                      totalDeviceCount > 50 ? 25 : 35;
    
    // Add interface nodes in a vertical column with extra spacing between groups
    filteredInterfaces.forEach((iface, index) => {
      // Add extra gap if this isn't the first interface
      const extraSpace = index > 0 ? interfaceGroupGap : 0;
      const y = startY + (index * verticalSpacing) + (index * extraSpace);
      
      // Ensure unique ID for interface
      const interfaceId = `interface-${iface.device}`;
      
      flowNodes.push({
        id: interfaceId,
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
    
    // Group devices by interface, with special handling for CARP interfaces
    devices.forEach((device) => {
      if (device && device.intf) {
        // Create normalized interface name to handle CARP interfaces in HA setups
        // This will map devices on CARP interfaces to their corresponding regular interface
        let normalizedIntf = device.intf;
        
        // Special handling for devices on CARP interfaces
        if (device.intf.includes('_vip') || device.intf.startsWith('carp')) {
          // Try to find the corresponding physical interface
          const baseIntf = device.intf.replace('_vip', '').replace(/^carp\d+_/, '');
          
          // If we can find a matching physical interface, use that instead
          const matchingInterface = filteredInterfaces.find(iface => iface.device === baseIntf);
          if (matchingInterface) {
            // Map this device to the physical interface instead of the CARP interface
            normalizedIntf = matchingInterface.device;
            console.log(`Mapped device on CARP interface ${device.intf} to physical interface ${normalizedIntf}`);
          }
        }
        
        // Now use the normalized interface name
        if (!interfaceDeviceMap.has(normalizedIntf)) {
          interfaceDeviceMap.set(normalizedIntf, []);
        }
        interfaceDeviceMap.get(normalizedIntf).push(device);
      }
    });
    
    console.log('Device map created with', interfaceDeviceMap.size, 'interfaces having devices');
    
    // Process all devices in one go for interfaces 
    filteredInterfaces.forEach((iface) => {
      const sourceNode = flowNodes.find(node => 
        node.type === 'interface' && 
        node.data.interfaceData.device === iface.device
      );
      
      if (sourceNode) {
        const connectedDevices = interfaceDeviceMap.get(iface.device) || [];
        const deviceCount = connectedDevices.length;
        
        console.log(`Processing ${connectedDevices.length} devices for interface ${iface.device}`);
        
        // Apply device limit based on device count
        // Limit if there are many devices to prevent overcrowding
        // Apply consistent limiting rules across all platforms
    const shouldLimit = (totalDeviceCount > 50 && deviceCount > deviceLimit) || deviceCount > 30;
        
        // Sort connected devices to prioritize important ones (permanent devices first, then by hostname)
        const sortedDevices = [...connectedDevices].sort((a, b) => {
          // Permanent devices first
          if (a.permanent && !b.permanent) return -1;
          if (!a.permanent && b.permanent) return 1;
          
          // Then non-expired devices
          if (!a.expired && b.expired) return -1;
          if (a.expired && !b.expired) return 1;
          
          // Then sort by hostname if available
          if (a.hostname && b.hostname) return a.hostname.localeCompare(b.hostname);
          
          return 0;
        });
        
        // Apply the device limit if needed
        const devicesToShow = shouldLimit ? sortedDevices.slice(0, deviceLimit) : sortedDevices;
        
        // Add a "more devices" indicator if we're limiting the view
        let isLimited = deviceCount > devicesToShow.length;
        let limitIndicator = null;
        
        if (isLimited) {
          // Create a visual indicator for additional devices that aren't shown
          const hiddenCount = deviceCount - devicesToShow.length;
          console.log(`Hiding ${hiddenCount} devices for interface ${iface.device} due to limits`);
        }
        
        devicesToShow.forEach((device, idx) => {
          const deviceId = `device-${device.mac}`;
          const devicesPerColumn = Math.min(3, deviceCount); // Consistent 3 devices per column on all platforms
          const columnNumber = Math.floor(idx / devicesPerColumn);
          const positionInColumn = idx % devicesPerColumn;
          
          // Calculate device position with more spacing to prevent overlap
          const deviceX = sourceNode.position.x + 150 + (columnNumber * deviceRowWidth);
          // Adjust the column start Y to position device nodes proportionally 
          const columnStartY = sourceNode.position.y - ((devicesPerColumn - 1) * 100) / 2;
          // Use 100px vertical spacing between devices in the same column
          const deviceY = columnStartY + positionInColumn * 100;
          
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
          
          // Create edge with distinctive styling
          flowEdges.push({
            id: `${sourceNode.id}-${deviceId}`,
            source: sourceNode.id,
            target: deviceId,
            // Remove animation for better performance
          animated: false,
            style: { 
              stroke: isExpired ? 'rgba(158, 158, 158, 0.4)' : 
                     isPermanent ? 'rgba(33, 150, 243, 0.8)' : 
                     'rgba(103, 58, 183, 0.7)',
              strokeWidth: isPermanent ? 2.5 : 1.8,
              // Use wavy lines for active devices, dots for permanent, dashed for expired
              strokeDasharray: isExpired ? '5,5' : 
                              isPermanent ? '1,5' : 
                              undefined
            },
            data: { deviceStatus: isExpired ? 'expired' : isPermanent ? 'permanent' : 'active' },
            type: 'default'
          });
        });
        
        // If devices are being limited, add a visual indicator node
        if (isLimited) {
          const hiddenCount = deviceCount - devicesToShow.length;
          // Calculate position for the indicator - after the last column
          const columnsUsed = Math.ceil(devicesToShow.length / 3); 
          const indicatorX = sourceNode.position.x + 150 + (columnsUsed * deviceRowWidth);
          const indicatorY = sourceNode.position.y;
          
          // Create indicator node showing how many devices are hidden
          const indicatorId = `indicator-${iface.device}`;
          flowNodes.push({
            id: indicatorId,
            type: 'device', // Reuse the device node type for simplicity
            position: { x: indicatorX, y: indicatorY },
            data: {
              label: `+${hiddenCount} more`,
              deviceData: {
                mac: `indicator-${iface.device}`,
                ipv4_addresses: [],
                ipv6_addresses: [],
                intf: iface.device,
                manufacturer: '',
                hostname: `+${hiddenCount} more devices`, 
                intf_description: iface.description || iface.device,
                indicator: true, // Special flag for styling
                isIndicator: true // For special handling in the DeviceNode component
              }
            }
          });
          
          // Add edge from interface to indicator with consistent structure
          flowEdges.push({
            id: `${sourceNode.id}-${indicatorId}`,
            source: sourceNode.id,
            target: indicatorId,
            style: { 
              stroke: 'rgba(158, 158, 158, 0.6)',
              strokeWidth: 1.5,
              strokeDasharray: '2,3'
            },
            data: { deviceStatus: 'indicator' },
            type: 'default'
          });
        }
      }
    });
    
    // Update all nodes and edges at once
    console.log(`Setting ${flowNodes.length} nodes and ${flowEdges.length} edges`);
    nodes.set(flowNodes);
    
    // Ensure edges is always an array before setting it to prevent TypeError
    if (!Array.isArray(flowEdges)) {
      console.error('flowEdges is not an array:', flowEdges);
      edges.set([]);
    } else {
      edges.set(flowEdges);
    }
    
    // Give a short delay before fitting the view
    setTimeout(() => {
      if (reactFlowInstance) {
        console.log('Fitting view after all nodes loaded');
        reactFlowInstance.fitView(fitViewOptions);
      }
    }, 100);
  }

  // Simplified event handlers with less redundancy and better debugging
  function handleNodeClick(event) {
    if (!event?.detail?.node) {
      console.log('No node in click event:', event);
      return;
    }
    
    try {
      const clickedNode = event.detail.node;
      console.log('Node clicked:', clickedNode);
      
      if (clickedNode.type === 'device' && clickedNode.data?.deviceData) {
        console.log('Device node clicked:', clickedNode.data.deviceData);
        handleElementSelectDirect(clickedNode.data.deviceData, 'device');
      } else if (clickedNode.type === 'interface' && clickedNode.data?.interfaceData) {
        console.log('Interface node clicked:', clickedNode.data.interfaceData);
        handleElementSelectDirect(clickedNode.data.interfaceData, 'interface');
      } else {
        console.log('Unknown node type or missing data:', clickedNode);
      }
    } catch (error) {
      console.error('Error handling node click:', error);
    }
  }
  
  // Track node position changes
  function handleNodeDragStop(event) {
    if (!event?.detail?.node) return;
    
    // Update node position in the store to maintain positions
    try {
      const { node } = event.detail;
      const updatedNodes = $nodes.map(n => 
        n.id === node.id 
          ? { ...n, position: node.position } 
          : n
      );
      nodes.set(updatedNodes);
    } catch (error) {
      console.error('Error updating node position:', error);
    }
  }
  
  // Simplified unified handler for all node events
  function handleNodeEvent(event) {
    if (event?.detail?.element) {
      handleElementSelectDirect(event.detail.element, event.detail.type);
    }
  }

  // Handle data changes
  $: if (interfaces.length > 0 || devices.length > 0) {
    prepareFlowData();
  }
  
  // Layout settings - increased padding for better visibility
  const fitViewOptions = { 
    padding: 0.2,  // Increased padding to ensure all nodes are visible
    includeHiddenNodes: false,  // Only include visible nodes
    minZoom: 0.2,  // Allow zooming out further
    maxZoom: 2.5   // Allow zooming in closer
  };
  let reactFlowInstance;
  
  // Force re-render hack to ensure the flow is properly initialized
  let key = 0;
  
  // Reset the component if we detect any issues
  function resetFlow() {
    key += 1;
    setTimeout(() => {
      if (reactFlowInstance) {
        reactFlowInstance.fitView(fitViewOptions);
      }
    }, 500);
  }
  
  onMount(() => {
    prepareFlowData();
  });
  
  // Handle flow initialization
  function handleInit(event) {
    console.log('SvelteFlow initialized');
    reactFlowInstance = event.detail;
    
    // Make sure the flow instance has proper methods
    if (reactFlowInstance) {
      console.log('Flow instance methods:', Object.keys(reactFlowInstance));
      
      // Force set viewport to make sure panning works
      reactFlowInstance.setViewport({ x: 0, y: 0, zoom: 1 });
      
      // Give a slight delay to ensure nodes are properly rendered before fitting
      setTimeout(() => {
        if (reactFlowInstance && interfaces.length > 0) {
          console.log('Auto-fitting view on initialization');
          try {
            reactFlowInstance.fitView(fitViewOptions);
            // Try zooming out a bit to show more context
            const currentViewport = reactFlowInstance.getViewport();
            reactFlowInstance.setViewport({ 
              ...currentViewport, 
              zoom: Math.max(0.8, currentViewport.zoom * 0.9)
            });
          } catch (err) {
            console.error('Error fitting view:', err);
          }
        }
      }, 300);  // Increased delay for more reliable rendering
    }
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
                        <path fill="currentColor" d={mdiEthernet} />
                      </svg>
                    </div>
                    <span class="text-xs">Physical</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-7 h-7 rounded-full bg-base-content/20 flex items-center justify-center">
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiLan} />
                      </svg>
                    </div>
                    <span class="text-xs">VLAN</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-7 h-7 rounded-full bg-base-content/20 flex items-center justify-center">
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiAccessPoint} />
                      </svg>
                    </div>
                    <span class="text-xs">Wireless</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-7 h-7 rounded-full bg-base-content/20 flex items-center justify-center">
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiServerNetwork} />
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
                    <div class="w-7 h-7 rounded-md overflow-hidden device-legend-item">
                      <div class="h-1/3 bg-blue-500"></div>
                      <div class="h-2/3 bg-white dark:bg-gray-700 flex items-center justify-center text-black dark:text-white">
                        <svg class="w-4 h-4" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiRouter} />
                        </svg>
                      </div>
                    </div>
                    <span class="text-xs">Router</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-7 h-7 rounded-md overflow-hidden device-legend-item">
                      <div class="h-1/3 bg-purple-500"></div>
                      <div class="h-2/3 bg-white dark:bg-gray-700 flex items-center justify-center text-black dark:text-white">
                        <svg class="w-4 h-4" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiLaptop} />
                        </svg>
                      </div>
                    </div>
                    <span class="text-xs">Computer</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-7 h-7 rounded-md overflow-hidden device-legend-item">
                      <div class="h-1/3 bg-purple-500"></div>
                      <div class="h-2/3 bg-white dark:bg-gray-700 flex items-center justify-center text-black dark:text-white">
                        <svg class="w-4 h-4" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiCellphone} />
                        </svg>
                      </div>
                    </div>
                    <span class="text-xs">Mobile Device</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-7 h-7 rounded-md overflow-hidden device-legend-item">
                      <div class="h-1/3 bg-orange-500"></div>
                      <div class="h-2/3 bg-white dark:bg-gray-700 flex items-center justify-center text-black dark:text-white">
                        <svg class="w-4 h-4" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiServerNetwork} />
                        </svg>
                      </div>
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
        on:elementSelect={(event) => {
          console.log('elementSelect event bubbled to container:', event);
          if (event.detail && event.detail.element) {
            console.log('Processing bubbled elementSelect event');
            handleElementSelectDirect(event.detail.element, event.detail.type);
          }
        }}
        on:device:select={(event) => {
          console.log('device:select event bubbled to container:', event);
          if (event.detail && event.detail.element) {
            console.log('Processing bubbled device:select event');
            handleElementSelectDirect(event.detail.element, 'device');
          }
        }}
        on:interface:select={(event) => {
          console.log('interface:select event bubbled to container:', event);
          if (event.detail && event.detail.element) {
            console.log('Processing bubbled interface:select event');
            handleElementSelectDirect(event.detail.element, 'interface');
          }
        }}
       >
        <!-- Firewall container - rendered as a div overlay with enhanced styling -->
        {#if interfaces.length > 0}
          <div 
            class="absolute pointer-events-none z-0 rounded-2xl firewall-container"
            style="left: 30px; top: 30px; width: 220px; height: {Math.min(2500, (interfaces.length * 200) + (interfaces.length * 150) + 150)}px;"
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
          {key}
          nodes={$nodes}
          edges={$edges}
          {nodeTypes}
          fitView={true}
          {fitViewOptions}
          on:init={handleInit}
          on:nodeClick={handleNodeClick}
          on:nodeDragStop={handleNodeDragStop} 
          on:elementSelect={handleNodeEvent}
          on:device:select={handleNodeEvent}
          on:interface:select={handleNodeEvent}
          elevateEdgesOnSelect={true}
          nodesDraggable={true}
          nodesConnectable={false}
          zoomOnScroll={true}
          panOnScroll={true}
          panOnDrag={true}
          selectionOnDrag={false}
          defaultViewport={{ x: 0, y: 0, zoom: 1 }}
          minZoom={0.2}
          maxZoom={2.5}
          proOptions={{ hideAttribution: true }}
          maxNodes={10000} 
          connectionMode="loose"
          snapToGrid={false}
          defaultEdgeOptions={{ 
            type: 'default',
            animated: false,
            style: {
              strokeLinecap: 'round',
              strokeLinejoin: 'round'
            }
          }}
          onlyRenderVisibleElements={false}
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
    stroke: rgba(75, 85, 99, 0.7);
    stroke-width: 2;
    transition: stroke 0.3s ease, stroke-width 0.3s ease;
    /* Add rounded line caps for nicer appearance */
    stroke-linecap: round;
    stroke-linejoin: round;
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
  
  /* Special styling for the indicator nodes */
  :global(.svelte-flow__node[data-indicator="true"]) {
    opacity: 0.7;
    transition: opacity 0.3s ease;
  }
  
  :global(.svelte-flow__node[data-indicator="true"]:hover) {
    opacity: 1;
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
  
  /* Mobile device optimizations */
  @media (max-width: 768px) {
    :global(.svelte-flow__edge-path) {
      stroke-width: 1.25 !important;
    }
    
    :global(.svelte-flow__edge.animated .svelte-flow__edge-path) {
      animation: none !important;
    }
    
    :global(.svelte-flow__controls) {
      transform: scale(0.75);
      bottom: 15px;
    }
  }
  
  /* iOS-specific optimizations */
  @supports (-webkit-touch-callout: none) {
    :global(.svelte-flow__edge.animated .svelte-flow__edge-path) {
      animation: none !important;
    }
    
    :global(.svelte-flow__node) {
      transform: translateZ(0);
    }
    
    :global(.svelte-flow__controls button) {
      width: 24px !important;
      height: 24px !important;
    }
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
  
  /* Ensure device legend items have proper contrast in dark mode */
  .device-legend-item {
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  /* Control styling for fit-view button */
  :global(.svelte-flow__controls) {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
</style>