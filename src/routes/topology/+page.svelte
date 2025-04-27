<script lang="ts">
  import { onMount } from "svelte";
  import AppLayout from "../AppLayout.svelte";
  import TopologyMap from "$lib/components/topology/TopologyMap.svelte";
  import TopologyDetails from "$lib/components/topology/TopologyDetails.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import { mdiRefresh, mdiInformation, mdiEye, mdiEyeOff, mdiMenuDown } from "@mdi/js";
  import type { Interface } from "$lib/components/topology/types";
  import type { CombinedDevice } from "$lib/components/topology/types";

  let interfaces: Interface[] = [];
  let devices: CombinedDevice[] = [];
  let isLoading = true;
  let error: string | null = null;
  let selectedElement: Interface | CombinedDevice | null = null;
  let showDetailsPanel = false;
  let selectedInterfaces: string[] = [];
  let isFilterDropdownOpen = false;
  
  // Data loading settings
  let loadTimeout = 30000;  // 30 seconds timeout for data loading
  let hasTimeout = false;   // Flag to track if we hit a timeout

  onMount(async () => {
    await fetchData();
  });

  async function fetchData() {
    const startTime = performance.now();
    isLoading = true;
    error = null;
    hasTimeout = false;
    
    // Keep old data visible while loading new data 
    // for a better user experience
    
    try {
      // Log network characteristics for diagnostics
      const networkIndicators = {
        // Has HA-related interfaces
        hasHaInterfaces: interfaces.some(iface => 
          iface.description && (
            iface.description.toLowerCase().includes('ha') || 
            iface.description.toLowerCase().includes('high availability') ||
            iface.description.toLowerCase().includes('carp')
          )
        ),
        // Has many interfaces (over 8 is already complex)
        hasManyInterfaces: interfaces.length > 8,
        // Previously timed out
        previousTimeout: hasTimeout,
        // Device count information
        deviceCount: devices.length,
        // Has VPN interfaces which often have many remote clients
        hasVpnInterfaces: interfaces.some(iface =>
          iface.description && (
            iface.description.toLowerCase().includes('vpn') ||
            iface.description.toLowerCase().includes('tunnel') ||
            iface.description.toLowerCase().includes('wireguard') ||
            iface.description.toLowerCase().includes('openvpn')
          )
        )
      };
      
      // Log network indicators for diagnostics
      await logNetworkTopologyEvent('networkCheck', networkIndicators);
      
      // Create a timeout promise that rejects after specified time
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => {
          hasTimeout = true;
          reject(new Error(`Request timed out after ${loadTimeout/1000} seconds.`));
        }, loadTimeout);
      });
      
      // Try to get interfaces first, since they're smaller and load faster
      const interfaceFetchStart = performance.now();
      const interfacesPromise = invoke<Interface[]>("get_interfaces");
      
      try {
        // Race the interface load against timeout
        interfaces = await Promise.race([interfacesPromise, timeoutPromise]) as Interface[];
        const interfaceFetchEnd = performance.now();
        
        // Log interface loading performance
        await logNetworkTopologyEvent('interfacesLoaded', {
          count: interfaces.length,
          duration: interfaceFetchEnd - interfaceFetchStart,
          haInterfaces: interfaces.filter(iface => 
            iface.description && (
              iface.description.toLowerCase().includes('ha') || 
              iface.description.toLowerCase().includes('high availability') ||
              iface.description.toLowerCase().includes('carp')
            )
          ).length
        });
        
        // Log updated network information with latest data
        const updatedNetworkInfo = {
          // Interface count and types
          interfaceCount: interfaces.length,
          haInterfaces: interfaces.filter(iface => 
            iface.description && (
              iface.description.toLowerCase().includes('ha') || 
              iface.description.toLowerCase().includes('high availability') ||
              iface.description.toLowerCase().includes('carp')
            )
          ).length,
          vpnInterfaces: interfaces.filter(iface =>
            iface.description && (
              iface.description.toLowerCase().includes('vpn') ||
              iface.description.toLowerCase().includes('tunnel') ||
              iface.description.toLowerCase().includes('wireguard') ||
              iface.description.toLowerCase().includes('openvpn')
            )
          ).length,
          // Status summary
          interfacesUp: interfaces.filter(iface => iface.status?.toLowerCase() === 'up').length,
          interfacesDown: interfaces.filter(iface => iface.status?.toLowerCase() === 'down').length
        };
        
        // Log updated network information
        await logNetworkTopologyEvent('updatedNetworkInfo', updatedNetworkInfo);
        
        // If we got interfaces successfully, update the UI immediately
        // This gives users faster feedback even if devices are still loading
        if (interfaces.length > 0) {
          // Initialize selected interfaces
          if (selectedInterfaces.length === 0) {
            selectedInterfaces = getActiveInterfaces().map(iface => iface.device);
          }
        }
        
        // Now load devices with a separate timeout - if interfaces worked, devices might work too
        const deviceFetchStart = performance.now();
        const devicesPromise = invoke<CombinedDevice[]>("get_combined_devices");
        devices = await Promise.race([devicesPromise, timeoutPromise]) as CombinedDevice[];
        const deviceFetchEnd = performance.now();
        
        // Log device loading performance
        await logNetworkTopologyEvent('devicesLoaded', {
          count: devices.length,
          duration: deviceFetchEnd - deviceFetchStart
        });
        
        // Log if there are many devices 
        if (devices.length > 75) {
          console.log(`Large number of devices detected (${devices.length})`);
          
          // Log large device count in diagnostics
          await logNetworkTopologyEvent('manyDevicesDetected', { 
            deviceCount: devices.length
          });
        }
      
      } catch (err) {
        // If we catch a timeout, retry with longer timeout
        console.error("Request timed out, retrying:", err);
        
        // Log the timeout
        await logNetworkTopologyEvent('requestTimeout', { 
          error: err.toString(),
          phase: interfaces.length ? 'deviceLoading' : 'interfaceLoading'
        });
        
        toasts.warning("Network data request timed out. Retrying...");
        
        // Retry with increased timeout
        try {
          // Log retry attempt
          await logNetworkTopologyEvent('retryingRequest', {
            timeoutMs: loadTimeout * 1.5
          });
          
          // Try interfaces again with longer timeout
          const retryInterfaceStart = performance.now();
          interfaces = await invoke<Interface[]>("get_interfaces");
          const retryInterfaceEnd = performance.now();
          
          // Log retry performance
          await logNetworkTopologyEvent('retryInterfacesLoaded', {
            count: interfaces.length,
            duration: retryInterfaceEnd - retryInterfaceStart
          });
          
          // Initialize selected interfaces if we got any
          if (interfaces.length > 0 && selectedInterfaces.length === 0) {
            selectedInterfaces = getActiveInterfaces().map(iface => iface.device);
          }
          
          // Try devices with longer timeout
          const retryDeviceStart = performance.now();
          devices = await invoke<CombinedDevice[]>("get_combined_devices");
          const retryDeviceEnd = performance.now();
          
          // Log retry device performance
          await logNetworkTopologyEvent('retryDevicesLoaded', {
            count: devices.length,
            duration: retryDeviceEnd - retryDeviceStart
          });
          
        } catch (retryErr) {
          // Log retry failure
          await logNetworkTopologyEvent('retryFailed', {
            error: retryErr.toString()
          });
          
          throw retryErr; // Re-throw to be caught by the outer catch
        }
      }
      
      // Initialize selected interfaces if this is the first load
      if (selectedInterfaces.length === 0) {
        // By default, show all active interfaces
        selectedInterfaces = getActiveInterfaces().map(iface => iface.device);
      }
      
      console.log(`Loaded ${interfaces.length} interfaces and ${devices.length} devices`);
      
      // Log successful load completion
      const endTime = performance.now();
      await logNetworkTopologyEvent('loadCompleted', {
        interfaceCount: interfaces.length,
        deviceCount: devices.length,
        totalDuration: endTime - startTime
      });
      
    } catch (err) {
      console.error("Failed to fetch network data:", err);
      error = `Failed to fetch network data: ${err}`;
      
      // Log load failure
      await logNetworkTopologyEvent('loadFailed', {
        error: err.toString(),
        hasTimeout: hasTimeout
      });
      
      if (hasTimeout) {
        toasts.error(`Network request timed out. Please try again later.`);
      } else {
        toasts.error("Failed to load network topology data");
      }
    } finally {
      isLoading = false;
    }
  }
  
  // Helper function to log network topology events (using standard console logging)
  async function logNetworkTopologyEvent(eventType: string, data: Record<string, any>) {
    try {
      // Log to console instead of diagnostics system
      console.log(`Network Topology Event [${eventType}]:`, data);
    } catch (err) {
      // Don't let logging failures affect the app
      console.error("Failed to log topology event:", err);
    }
  }

  function handleElementSelect(event) {
    console.log("Element selected event received in +page.svelte:", event);
    
    // Check if detail is directly on the event or inside event.detail
    const detail = event?.detail || event;
    
    console.log("Element selected:", detail?.type, detail?.element);
    
    try {
      // Handle direct props
      if (detail?.element) {
        console.log("*** SETTING selectedElement to:", detail.element);
        console.log("*** Element has properties:", Object.keys(detail.element));
        
        // Set both state variables and force re-render
        selectedElement = detail.element;
        showDetailsPanel = true;
        
        console.log("*** AFTER setting: selectedElement=", selectedElement, "showDetailsPanel=", showDetailsPanel);
      } 
      // Handle SvelteFlow native events
      else if (event?.detail?.node?.data) {
        console.log("Got native node event:", event.detail.node);
        
        if (event.detail.node.type === 'device' && event.detail.node.data.deviceData) {
          selectedElement = event.detail.node.data.deviceData;
          showDetailsPanel = true;
          console.log("Set selectedElement from node.data.deviceData:", selectedElement);
        } 
        else if (event.detail.node.type === 'interface' && event.detail.node.data.interfaceData) {
          selectedElement = event.detail.node.data.interfaceData;
          showDetailsPanel = true;
          console.log("Set selectedElement from node.data.interfaceData:", selectedElement);
        }
      } 
      else {
        console.error("Missing element in event detail:", detail);
      }
    } catch (error) {
      console.error("Error handling element select:", error);
    }
  }

  function closeDetailsPanel() {
    console.log('Closing details panel');
    showDetailsPanel = false;
    selectedElement = null; // Remove timeout to ensure immediate cleanup
  }
  
  function toggleInterface(ifaceName: string) {
    if (selectedInterfaces.includes(ifaceName)) {
      selectedInterfaces = selectedInterfaces.filter(name => name !== ifaceName);
    } else {
      selectedInterfaces = [...selectedInterfaces, ifaceName];
    }
  }
  
  function toggleAllInterfaces() {
    if (selectedInterfaces.length === getActiveInterfaces().length) {
      // If all are selected, deselect all
      selectedInterfaces = [];
    } else {
      // Otherwise, select all
      selectedInterfaces = getActiveInterfaces().map(iface => iface.device);
    }
  }
  
  function getActiveInterfaces() {
    // Include all interfaces except system ones
    return interfaces.filter(iface => 
      iface.device !== 'lo0' && iface.device !== 'pfsync0'
    );
  }
  
  function toggleFilterDropdown() {
    isFilterDropdownOpen = !isFilterDropdownOpen;
  }
  
  // Filter interfaces based on selection
  $: filteredInterfaces = interfaces.filter(iface => {
    const isSelected = selectedInterfaces.includes(iface.device);
    return isSelected;
  });
  
  // Count total active interfaces for UI
  $: totalActiveInterfaces = getActiveInterfaces().length;
  
  // Log filter changes and debug selected interfaces
  $: {
    console.log(`Available interfaces: ${interfaces.length}`);
    console.log(`Selected interfaces: ${selectedInterfaces.length}`, selectedInterfaces);
    console.log(`Active interfaces: ${getActiveInterfaces().length}`);
    console.log(`Filtered interfaces: ${filteredInterfaces.length}`, filteredInterfaces);
  }
</script>

<AppLayout>
  <div class="flex flex-col h-full topology-page-container">
    <div class="flex flex-wrap justify-between items-center px-2 py-1 gap-2">
      <h1 class="text-2xl font-bold">Network Topology</h1>
      
      <div class="flex flex-wrap gap-2">
        <!-- Interface selection dropdown -->
        <div class="dropdown dropdown-end">
          <button 
            class="btn btn-sm btn-primary text-white flex items-center gap-1" 
            on:click={toggleFilterDropdown}
          >
            <span>Interfaces</span>
            <svg class="w-5 h-5" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiMenuDown} />
            </svg>
          </button>
          
          {#if isFilterDropdownOpen}
            <div class="dropdown-content z-30 menu p-2 shadow bg-base-100 border border-base-300 rounded-box w-64 mt-1">
              <div class="p-2 border-b border-base-300">
                <label class="cursor-pointer label justify-start gap-2">
                  <input 
                    type="checkbox" 
                    class="checkbox checkbox-sm checkbox-primary" 
                    checked={selectedInterfaces.length === totalActiveInterfaces}
                    on:change={toggleAllInterfaces}
                  />
                  <span class="label-text font-medium">Select All ({totalActiveInterfaces})</span>
                </label>
              </div>
              
              <div class="max-h-[min(60vh,300px)] overflow-y-auto">
                {#each getActiveInterfaces() as iface}
                  <label class="cursor-pointer label justify-start gap-2 hover:bg-base-200 rounded py-2">
                    <input 
                      type="checkbox" 
                      class="checkbox checkbox-sm checkbox-primary" 
                      checked={selectedInterfaces.includes(iface.device)}
                      on:change={() => toggleInterface(iface.device)}
                    />
                    <div class="flex flex-col">
                      <span class="label-text font-medium">{iface.description || iface.device}</span>
                      <div class="flex items-center gap-1">
                        <span class="label-text text-xs opacity-70">{iface.device}</span>
                        <span class="badge badge-xs {iface.status?.toLowerCase() === 'up' ? 'badge-success' : 'badge-error'}">{iface.status}</span>
                        {#if iface.is_physical}
                          <span class="badge badge-xs badge-primary">Physical</span>
                        {:else}
                          <span class="badge badge-xs badge-secondary">Virtual</span>
                        {/if}
                      </div>
                    </div>
                    {#if selectedInterfaces.includes(iface.device)}
                      <svg class="w-4 h-4 ml-auto text-success" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiEye} />
                      </svg>
                    {:else}
                      <svg class="w-4 h-4 ml-auto text-error" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiEyeOff} />
                      </svg>
                    {/if}
                  </label>
                {/each}
              </div>
            </div>
          {/if}
        </div>
        
        
        <button 
          class="btn btn-sm btn-primary text-white" 
          on:click={fetchData}
          disabled={isLoading}
        >
          <svg class="w-5 h-5 {isLoading ? 'animate-spin' : ''}" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiRefresh} />
          </svg>
          Refresh
        </button>
      </div>
    </div>
    
    {#if isLoading}
      <div class="flex justify-center items-center py-12">
        <div class="text-center">
          <span class="loading loading-spinner loading-lg"></span>
          <p class="mt-4 text-base-content">Loading network topology...</p>
        </div>
      </div>
    {:else if error}
      <div class="alert alert-error shadow-lg mb-4">
        <div>
          <svg
            class="w-6 h-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <div>
            <h3 class="font-bold">Error Loading Network Data</h3>
            <div class="text-sm">{error}</div>
            
            {#if error.includes('timeout') || error.includes('timed out')}
              <div class="mt-2 text-xs opacity-80">
                <p>This may be caused by:</p>
                <ul class="list-disc ml-4 mt-1">
                  <li>High load on your firewall</li>
                  <li>Network congestion</li>
                  <li>Complex HA setup with many interfaces</li>
                </ul>
              </div>
            {:else if error.includes('connect')}
              <div class="mt-2 text-xs opacity-80">
                <p>Please check:</p>
                <ul class="list-disc ml-4 mt-1">
                  <li>Your network connectivity</li>
                  <li>Firewall URL and port settings</li>
                  <li>That the OPNsense API is enabled and accessible</li>
                </ul>
              </div>
            {:else if error.includes('parse') || error.includes('missing field') || error.includes('expected')}
              <div class="mt-2 text-xs opacity-80">
                <p>This appears to be a data parsing issue:</p>
                <ul class="list-disc ml-4 mt-1">
                  <li>Your firewall may have custom or unusual interface configurations</li>
                  <li>There may be an API version mismatch between OPNManager and your firewall</li>
                  <li>Try updating both OPNsense and OPNManager to the latest versions</li>
                </ul>
              </div>
            {/if}
          </div>
        </div>
        <div class="flex-none">
          <button class="btn btn-sm" on:click={fetchData}>
            Retry
          </button>
        </div>
      </div>
    {:else}
      <!-- Full width layout with modal details -->
      <div class="relative flex-1 h-[calc(100vh-13rem)]">
        <div class="w-full h-full overflow-hidden">
          <TopologyMap
            interfaces={filteredInterfaces}
            devices={devices}
            onElementSelect={(element, type) => {
              console.log('Direct callback from TopologyMap:', type, element);
              selectedElement = element;
              showDetailsPanel = true;
            }}
            on:elementSelect={handleElementSelect}
          />
        </div>
        
        {#if showDetailsPanel && selectedElement}
          <div class="absolute top-4 right-4 w-80 z-20">
            <div class="bg-base-100 rounded-lg shadow-lg overflow-hidden border border-base-300">
              <TopologyDetails
                element={selectedElement}
                elementType={selectedElement && selectedElement.intf ? 'device' : 'interface'}
                onClose={closeDetailsPanel}
              />
            </div>
          </div>
        {/if}
      </div>
      
    {/if}
  </div>
</AppLayout>

<style>
  /* Fix dropdown positioning */
  .dropdown-content {
    position: absolute;
    right: 0;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1);
  }
  
  @media (max-width: 640px) {
    .dropdown-content {
      width: calc(100vw - 1rem);
      max-width: 100%;
    }
  }
  
  /* Improved dropdown styling for dark mode */
  :global([data-theme="dark"]) .dropdown-content {
    background-color: hsl(var(--b1));
    border-color: rgba(255, 255, 255, 0.1);
  }
  
  /* Only modify padding for topology page without affecting iOS fixes */
  .topology-page-container {
    padding: 0 !important;
  }
  
  /* Fix controls position for iOS */
  @supports (-webkit-touch-callout: none) {
    .topology-page-container .h-\[calc\(100vh-13rem\)\] {
      height: calc(100vh - 13rem - 80px) !important;
    }
  }
</style>