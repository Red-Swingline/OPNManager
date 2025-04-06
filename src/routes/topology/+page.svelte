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

  onMount(async () => {
    await fetchData();
  });

  async function fetchData() {
    isLoading = true;
    error = null;
    try {
      // Fetch data in parallel
      const [interfacesData, devicesData] = await Promise.all([
        invoke<Interface[]>("get_interfaces"),
        invoke<CombinedDevice[]>("get_combined_devices")
      ]);
      
      interfaces = interfacesData;
      devices = devicesData;
      
      // Initialize selected interfaces if this is the first load
      if (selectedInterfaces.length === 0) {
        // By default, show all active interfaces
        selectedInterfaces = interfaces
          .filter(iface => iface.status?.toLowerCase() === 'up' && (iface.identifier || iface.is_physical))
          .map(iface => iface.device);
      }
      
      console.log(`Fetched ${interfaces.length} interfaces and ${devices.length} devices`);
    } catch (err) {
      console.error("Failed to fetch network data:", err);
      error = `Failed to fetch network data: ${err}`;
      toasts.error("Failed to load network topology data");
    } finally {
      isLoading = false;
    }
  }

  function handleElementSelect(detail: { element: Interface | CombinedDevice, type: 'interface' | 'device' }) {
    console.log("Element selected:", detail.type, detail.element);
    selectedElement = detail.element;
    showDetailsPanel = true;
  }

  function closeDetailsPanel() {
    showDetailsPanel = false;
    setTimeout(() => {
      selectedElement = null;
    }, 300);
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
    return interfaces.filter(iface => 
      iface.status?.toLowerCase() === 'up' && (iface.identifier || iface.is_physical)
    );
  }
  
  function toggleFilterDropdown() {
    isFilterDropdownOpen = !isFilterDropdownOpen;
  }
  
  // Filter interfaces based on selection
  $: filteredInterfaces = interfaces.filter(iface => selectedInterfaces.includes(iface.device));
  
  // Count total active interfaces for UI
  $: totalActiveInterfaces = getActiveInterfaces().length;
</script>

<AppLayout>
  <div class="p-4">
    <div class="flex flex-wrap justify-between items-center mb-4 gap-2">
      <h1 class="text-2xl font-bold">Network Topology</h1>
      
      <div class="flex flex-wrap gap-2">
        <!-- Interface selection dropdown -->
        <div class="dropdown dropdown-end">
          <button 
            class="btn btn-sm btn-outline flex items-center gap-1" 
            on:click={toggleFilterDropdown}
          >
            <span>Interfaces: {selectedInterfaces.length}/{totalActiveInterfaces}</span>
            <svg class="w-5 h-5" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiMenuDown} />
            </svg>
          </button>
          
          {#if isFilterDropdownOpen}
            <div class="dropdown-content z-30 menu p-2 shadow bg-base-100 rounded-box w-64 mt-1">
              <div class="p-2 border-b">
                <label class="cursor-pointer label justify-start gap-2">
                  <input 
                    type="checkbox" 
                    class="checkbox checkbox-sm" 
                    checked={selectedInterfaces.length === totalActiveInterfaces}
                    on:change={toggleAllInterfaces}
                  />
                  <span class="label-text font-medium">Select All Interfaces</span>
                </label>
              </div>
              
              <div class="max-h-60 overflow-y-auto">
                {#each getActiveInterfaces() as iface}
                  <label class="cursor-pointer label justify-start gap-2 hover:bg-base-200 rounded">
                    <input 
                      type="checkbox" 
                      class="checkbox checkbox-sm" 
                      checked={selectedInterfaces.includes(iface.device)}
                      on:change={() => toggleInterface(iface.device)}
                    />
                    <div class="flex flex-col">
                      <span class="label-text font-medium">{iface.description || iface.device}</span>
                      <span class="label-text text-xs opacity-70">{iface.device}</span>
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
          class="btn btn-sm btn-outline" 
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
            <h3 class="font-bold">Error</h3>
            <div class="text-xs">{error}</div>
          </div>
        </div>
        <div class="flex-none">
          <button class="btn btn-sm" on:click={fetchData}>Retry</button>
        </div>
      </div>
    {:else}
      <!-- Desktop view: Side-by-side layout -->
      <div class="hidden lg:grid lg:grid-cols-3 lg:gap-4">
        <div class="lg:col-span-{showDetailsPanel ? '2' : '3'} bg-base-100 rounded-lg shadow-md overflow-hidden h-[calc(100vh-12rem)]">
          <TopologyMap
            interfaces={filteredInterfaces}
            devices={devices}
            on:elementSelect={e => handleElementSelect(e.detail)}
          />
        </div>
        
        {#if showDetailsPanel && selectedElement}
          <div class="bg-base-100 rounded-lg shadow-md h-[calc(100vh-12rem)] overflow-hidden">
            <TopologyDetails
              element={selectedElement}
              elementType={selectedElement.hasOwnProperty('intf') ? 'device' : 'interface'}
              onClose={closeDetailsPanel}
            />
          </div>
        {/if}
      </div>
      
      <!-- Mobile view: Map with overlay details panel -->
      <div class="lg:hidden relative">
        <div class="bg-base-100 rounded-lg shadow-md overflow-hidden h-[calc(100vh-12rem)]">
          <TopologyMap
            interfaces={filteredInterfaces}
            devices={devices}
            on:elementSelect={e => handleElementSelect(e.detail)}
          />
        </div>
        
        {#if showDetailsPanel && selectedElement}
          <div class="absolute top-0 left-0 right-0 bottom-0 z-10">
            <div class="bg-base-100 rounded-lg shadow-lg overflow-hidden h-[calc(100vh-12rem)]" style="opacity: 0.98;">
              <TopologyDetails
                element={selectedElement}
                elementType={selectedElement.hasOwnProperty('intf') ? 'device' : 'interface'}
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
  }
  
  @media (max-width: 640px) {
    .dropdown-content {
      width: calc(100vw - 2rem);
      max-width: 100%;
    }
  }
</style>