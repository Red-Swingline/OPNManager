<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import { goto } from "$app/navigation";
  import { mdiPower, mdiRefresh, mdiCog, mdiChevronDown, mdiChevronUp } from "@mdi/js";

  interface WolHost {
    uuid: string;
    interface: string;
    mac: string;
    descr: string;
  }

  interface ArpDevice {
    mac: string;
    ip: string;
    hostname: string;
    interface: string;
    manufacturer?: string;
  }
  
  let isLoading = true;
  let isPluginInstalled = false;
  let savedHosts: WolHost[] = [];
  let arpDevices: ArpDevice[] = [];
  let interfaces = {};
  let isWaking = false;
  let isExpanded = false;
  // Selected device from ARP list
  let selectedInterface = "";
  let selectedMac = "";
  let selectedDescription = "";
  
  function toggleExpand() {
    isExpanded = !isExpanded;
  }
  
  async function checkPluginStatus() {
    try {
      isPluginInstalled = await invoke<boolean>("check_wol_plugin_installed");
      if (isPluginInstalled) {
        await Promise.all([
          loadSavedHosts(),
          loadArpDevices(),
          loadInterfaces()
        ]);
      }
    } catch (error) {
      console.error("Failed to check WoL plugin status:", error);
      isPluginInstalled = false;
    } finally {
      isLoading = false;
    }
  }
  
  async function loadSavedHosts() {
    try {
      const response = await invoke<any>("search_wol_hosts");
      if (response && response.rows) {
        savedHosts = response.rows;
      }
    } catch (error) {
      console.error("Failed to load WoL hosts:", error);
      toasts.error(`Failed to load WoL hosts: ${error}`);
    }
  }
  
  async function loadArpDevices() {
    try {
      const response = await invoke<any>("get_arp_devices");
      if (response) {
        arpDevices = Object.values(response);
      }
    } catch (error) {
      console.error("Failed to load ARP devices:", error);
      toasts.error(`Failed to load ARP devices: ${error}`);
    }
  }
  
  async function loadInterfaces() {
    try {
      interfaces = await invoke<any>("get_wol_interfaces");
      // Set default interface if available
      if (Object.keys(interfaces).length > 0) {
        selectedInterface = Object.keys(interfaces)[0];
      }
    } catch (error) {
      console.error("Failed to load interfaces:", error);
      toasts.error(`Failed to load interfaces: ${error}`);
    }
  }

  async function wakeDevice(uuid: string, description: string) {
    try {
      isWaking = true;
      const result = await invoke<any>("wake_device", { uuid });
      
      // Check possible success response formats from OPNsense WoL API
      if (result === "[]" || // Empty array string response
          Array.isArray(result) && result.length === 0 || // Empty array
          !result || // Empty/null response can also indicate success
          (result && 
           (result.status === "OK" || 
            result.result === "OK" || 
            result.result === "saved" || 
            (result.wake && result.validations === null)))) {
        toasts.success(`Wake-on-LAN packet sent to ${description || "device"}`);
      } else {
        // Try to extract error message from various possible formats
        const errorMsg = result?.error_msg || 
                         result?.validations?.wake?.uuid?.[0] || 
                         "Unknown error";
        console.error("WoL response:", JSON.stringify(result));
        toasts.error(`Failed to wake device: ${errorMsg}`);
      }
    } catch (error) {
      console.error(`Failed to wake device ${description}:`, error);
      toasts.error(`Failed to wake device: ${error}`);
    } finally {
      isWaking = false;
    }
  }
  
  async function wakeMacAddress() {
    if (!selectedMac || !selectedInterface) {
      toasts.error("Please select an interface and MAC address");
      return;
    }
    
    try {
      isWaking = true;
      const result = await invoke<any>("wake_mac_address", { 
        interface: selectedInterface,
        mac: selectedMac,
        description: selectedDescription
      });
      
      // Check possible success response formats from OPNsense WoL API
      if (result === "[]" || // Empty array string response
          Array.isArray(result) && result.length === 0 || // Empty array
          !result || // Empty/null response can also indicate success
          (result && 
           (result.status === "OK" || 
            result.result === "OK" || 
            result.result === "saved" || 
            (result.wake && result.validations === null)))) {
        toasts.success(`Wake-on-LAN packet sent to ${selectedMac}`);
        // Reset selections
        selectedMac = "";
        selectedDescription = "";
      } else {
        // Try to extract error message from various possible formats
        const errorMsg = result?.error_msg || 
                         result?.validations?.wake?.mac?.[0] || 
                         result?.validations?.wake?.interface?.[0] ||
                         "Unknown error";
        console.error("WoL response:", JSON.stringify(result));
        toasts.error(`Failed to wake device: ${errorMsg}`);
      }
    } catch (error) {
      console.error(`Failed to wake device:`, error);
      toasts.error(`Failed to wake device: ${error}`);
    } finally {
      isWaking = false;
    }
  }
  
  function handleDeviceSelection(device: ArpDevice) {
    selectedMac = device.mac;
    selectedDescription = device.hostname || device.ip;
  }
  
  function navigateToWolPage() {
    goto("/wol");
  }
  
  
  onMount(async () => {
    await checkPluginStatus();
  });
</script>

<div class="card bg-base-100 shadow-xl">
  <div class="card-body">
    <div class="flex justify-between items-center">
      <h3 class="card-title text-lg flex items-center gap-2">
        <svg class="w-5 h-5 text-primary" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiPower} />
        </svg>
        Wake-on-LAN
        {#if isPluginInstalled && !isLoading && savedHosts.length > 0}
          <span class="badge badge-sm">{savedHosts.length}</span>
        {/if}
      </h3>
      <div class="flex gap-2">
        {#if isPluginInstalled && !isLoading}
          <button
            class="btn btn-ghost btn-sm"
            on:click={toggleExpand}
            title={isExpanded ? "Collapse" : "Expand"}
          >
            <svg class="w-5 h-5" viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d={isExpanded ? mdiChevronUp : mdiChevronDown}
              />
            </svg>
          </button>
          <button 
            class="btn btn-sm btn-ghost" 
            on:click={checkPluginStatus}
            disabled={isLoading || isWaking}
            title="Refresh"
          >
            <svg class="w-5 h-5" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiRefresh} />
            </svg>
          </button>
          <button 
            class="btn btn-sm btn-ghost" 
            on:click={navigateToWolPage}
            title="WoL Settings"
          >
            <svg class="w-5 h-5" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiCog} />
            </svg>
          </button>
        {:else}
          <button 
            class="btn btn-sm btn-ghost" 
            on:click={checkPluginStatus}
            disabled={isLoading}
            title="Refresh"
          >
            <svg class="w-5 h-5" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiRefresh} />
            </svg>
          </button>
        {/if}
      </div>
    </div>

    <div class="divider my-2"></div>

    {#if isLoading}
      <div class="flex justify-center items-center h-24">
        <span class="loading loading-spinner loading-md"></span>
      </div>
    {:else if !isPluginInstalled}
      <div class="text-center py-4">
        <p>Wake-on-LAN plugin is not installed.</p>
        
        <div class="mt-4 flex flex-col items-center gap-3">
          <div class="text-sm max-w-xs">
            To use this feature, please install the WoL plugin manually in OPNsense:
          </div>
          
          <ol class="text-xs opacity-80 text-left list-decimal list-inside mt-2">
            <li>Log in to your OPNsense web interface</li>
            <li>Navigate to System → Firmware → Plugins</li>
            <li>Search for "os-wol" in the search box</li>
            <li>Click the + button next to "os-wol" to install it</li>
            <li>Refresh this page when done</li>
          </ol>
          
          <button 
            class="btn btn-outline btn-sm mt-2"
            on:click={checkPluginStatus}
          >
            Check Again
          </button>
        </div>
      </div>
    {:else}
      <!-- Quick access section (always visible) -->
      <div class="mb-4">
        {#if savedHosts.length > 0}
          <div class="flex flex-wrap gap-2">
            {#each savedHosts.slice(0, isExpanded ? savedHosts.length : Math.min(3, savedHosts.length)) as host}
              <button 
                class="btn btn-sm btn-outline w-28" 
                on:click={() => wakeDevice(host.uuid, host.descr)}
                disabled={isWaking}
                title={`Wake ${host.descr || 'device'} (${host.mac})`}
              >
                <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24">
                  <path fill="currentColor" d={mdiPower} />
                </svg>
                <span class="truncate ml-1">{host.descr || "Device"}</span>
              </button>
            {/each}
            {#if !isExpanded && savedHosts.length > 3}
              <span class="text-sm opacity-70 self-center">+{savedHosts.length - 3} more</span>
            {/if}
          </div>
        {:else if !isExpanded}
          <div class="text-center py-2">
            <p class="text-sm">No saved Wake-on-LAN devices</p>
            <button 
              class="btn btn-sm btn-outline mt-2"
              on:click={navigateToWolPage}
            >
              Configure WoL Devices
            </button>
          </div>
        {/if}
      </div>
      
      <!-- Expanded content -->
      {#if isExpanded}
        <!-- Wake device by MAC section -->
        <div class="mb-4 p-3 border rounded-lg bg-base-200">
          <h4 class="font-medium mb-2">Wake Device by MAC Address</h4>
          <div class="space-y-2">
            <div class="form-control">
              <label class="label py-1">
                <span class="label-text">Interface</span>
              </label>
              <select 
                class="select select-bordered select-sm w-full" 
                bind:value={selectedInterface}
              >
                <option value="" disabled>Select an interface</option>
                {#each Object.entries(interfaces) as [key, option]}
                  <option value={key}>{option.value}</option>
                {/each}
              </select>
            </div>
            
            <div class="form-control">
              <label class="label py-1">
                <span class="label-text">Device</span>
              </label>
              <select 
                class="select select-bordered select-sm w-full" 
                bind:value={selectedMac}
                on:change={() => {
                  const device = arpDevices.find(d => d.mac === selectedMac);
                  if (device) {
                    selectedDescription = device.hostname || device.ip;
                  }
                }}
              >
                <option value="" disabled>Select a device</option>
                {#each arpDevices as device}
                  <option value={device.mac}>
                    {device.hostname || device.ip} ({device.mac})
                  </option>
                {/each}
              </select>
            </div>
            
            <div class="flex justify-end">
              <button 
                class="btn btn-sm btn-primary" 
                on:click={wakeMacAddress}
                disabled={!selectedMac || !selectedInterface || isWaking}
              >
                {#if isWaking}
                  <span class="loading loading-spinner loading-xs"></span>
                {/if}
                Wake Device
              </button>
            </div>
          </div>
        </div>
        
      {/if}
    {/if}
  </div>
</div>