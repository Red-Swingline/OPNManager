<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import AppLayout from "../AppLayout.svelte";
  import { mdiPower, mdiPlus, mdiRefresh, mdiDelete, mdiCheck, mdiClose, mdiUpdate, mdiArrowRight } from "@mdi/js";

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
  let hosts: WolHost[] = [];
  let arpDevices: ArpDevice[] = [];
  let interfaces = {};
  let isWaking = false;
  let isAdding = false;
  let isDeleting = false;
  
  // For network device selection
  let selectedDevice = '';
  
  // Modals
  let showAddModal = false;
  let showDeleteConfirm = false;
  let hostToDelete: WolHost | null = null;
  
  // New host form
  let newHost = {
    interface: "",
    mac: "",
    descr: ""
  };
  
  async function checkPluginStatus() {
    try {
      isLoading = true;
      isPluginInstalled = await invoke<boolean>("check_wol_plugin_installed");
      if (isPluginInstalled) {
        await Promise.all([
          loadHosts(),
          loadInterfaces(),
          loadArpDevices()
        ]);
      }
    } catch (error) {
      console.error("Failed to check WoL plugin status:", error);
      isPluginInstalled = false;
    } finally {
      isLoading = false;
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
  
  async function loadHosts() {
    try {
      const response = await invoke<any>("search_wol_hosts");
      if (response && response.rows) {
        hosts = response.rows;
      }
    } catch (error) {
      console.error("Failed to load WoL hosts:", error);
      toasts.error(`Failed to load WoL hosts: ${error}`);
    }
  }
  
  async function loadInterfaces() {
    try {
      interfaces = await invoke<any>("get_wol_interfaces");
      // Set default interface if available
      if (Object.keys(interfaces).length > 0) {
        newHost.interface = Object.keys(interfaces)[0];
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
  
  async function addHost() {
    if (!newHost.mac || !newHost.interface) {
      toasts.error("Interface and MAC address are required");
      return;
    }
    
    try {
      isAdding = true;
      const result = await invoke<any>("add_wol_host", {
        interface: newHost.interface,
        mac: newHost.mac,
        description: newHost.descr
      });
      
      if (result && result.result === "saved") {
        toasts.success("Device added successfully");
        showAddModal = false;
        newHost = {
          interface: Object.keys(interfaces)[0] || "",
          mac: "",
          descr: ""
        };
        await loadHosts();
      } else {
        toasts.error("Failed to add device");
      }
    } catch (error) {
      console.error("Failed to add WoL host:", error);
      toasts.error(`Failed to add WoL host: ${error}`);
    } finally {
      isAdding = false;
    }
  }
  
  function openDeleteConfirm(host: WolHost) {
    hostToDelete = host;
    showDeleteConfirm = true;
  }

  function closeDeleteConfirm() {
    showDeleteConfirm = false;
    hostToDelete = null;
  }
  
  async function deleteHost() {
    if (!hostToDelete) return;
    
    try {
      isDeleting = true;
      await invoke<any>("delete_wol_host", { uuid: hostToDelete.uuid });
      toasts.success(`Device "${hostToDelete.descr || 'Unnamed device'}" deleted`);
      closeDeleteConfirm();
      await loadHosts();
    } catch (error) {
      console.error("Failed to delete WoL host:", error);
      toasts.error(`Failed to delete WoL host: ${error}`);
    } finally {
      isDeleting = false;
    }
  }
  
  function openAddModal() {
    newHost = {
      interface: Object.keys(interfaces)[0] || "",
      mac: "",
      descr: ""
    };
    showAddModal = true;
  }
  
  function closeAddModal() {
    showAddModal = false;
  }
  
  function handleDeviceSelection(device) {
    newHost.mac = device.mac;
    newHost.descr = device.hostname || device.ip;
    
    // Try to match the interface if possible
    if (device.interface) {
      const interfaceKey = Object.keys(interfaces).find(key => 
        interfaces[key]?.value?.toLowerCase() === device.interface.toLowerCase()
      );
      if (interfaceKey) {
        newHost.interface = interfaceKey;
      }
    }
    
    // Close the dropdown by removing focus from the dropdown element
    document.activeElement?.blur();
  }
  
  function formatMacAddress(input: string) {
    // Remove non-hex characters
    const clean = input.replace(/[^0-9a-fA-F]/g, '');
    
    // Format as XX:XX:XX:XX:XX:XX
    const formatted = clean.match(/.{1,2}/g)?.join(':') || '';
    
    return formatted.substring(0, 17); // Limit to a valid MAC length
  }
  
  
  
  onMount(async () => {
    await checkPluginStatus();
  });
</script>

<AppLayout>
  <div class="container mx-auto p-4 max-w-4xl">
    <!-- Header Card -->
    <div class="card bg-base-100 shadow-xl mb-6">
      <div class="card-body p-4">
        <div class="flex justify-between items-center">
          <h1 class="text-xl font-bold">Wake-on-LAN</h1>
          {#if isPluginInstalled}
            <div class="flex gap-2">
              <button class="btn btn-sm btn-ghost" on:click={checkPluginStatus}>
                <svg class="w-5 h-5" viewBox="0 0 24 24">
                  <path fill="currentColor" d={mdiRefresh} />
                </svg>
              </button>
              <button class="btn btn-sm btn-primary" on:click={openAddModal}>
                <svg class="w-5 h-5" viewBox="0 0 24 24">
                  <path fill="currentColor" d={mdiPlus} />
                </svg>
              </button>
            </div>
          {:else}
            <button class="btn btn-sm btn-ghost" on:click={checkPluginStatus}>
              <svg class="w-5 h-5" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiRefresh} />
              </svg>
            </button>
          {/if}
        </div>
      </div>
    </div>
    
    {#if isLoading}
      <div class="flex justify-center items-center h-32">
        <span class="loading loading-spinner loading-lg"></span>
      </div>
    {:else if !isPluginInstalled}
      <div class="card bg-base-100 shadow-xl p-6 text-center">
        <div class="mb-4">
          <h2 class="text-xl font-bold">Wake-on-LAN Plugin Not Installed</h2>
          <p class="mt-2">The WoL plugin is required for this feature.</p>
        </div>
        
        <div class="bg-base-200 p-4 rounded-lg mx-auto max-w-lg">
          <h3 class="font-semibold mb-2">Installation Instructions:</h3>
          <div class="text-sm mb-4">
            Please install the WoL plugin manually through your OPNsense web interface:
          </div>
          <ol class="text-left list-decimal list-inside space-y-1">
            <li>Log in to your OPNsense web interface</li>
            <li>Navigate to System → Firmware → Plugins</li>
            <li>Search for "os-wol" in the search box</li>
            <li>Click the + button next to "os-wol" to install it</li>
            <li>Wait for installation to complete</li>
            <li>Return to this page and click "Check Again"</li>
          </ol>
          
          <div class="mt-4">
            <button 
              class="btn btn-primary"
              on:click={checkPluginStatus}
            >
              Check Again
            </button>
          </div>
        </div>
      </div>
    {:else}
      <!-- WoL Devices Table/Cards -->
      <div class="card bg-base-100 shadow-xl overflow-hidden">
        <div class="card-body p-0">
          {#if hosts.length === 0}
            <div class="p-6 text-center">
              <p class="text-lg">No Wake-on-LAN devices configured</p>
              <p class="text-sm opacity-70 mt-2">
                Click the + button above to add a device
              </p>
            </div>
          {:else}
            <!-- Table for tablets and desktop -->
            <div class="hidden sm:block overflow-x-auto">
              <table class="table w-full">
                <thead>
                  <tr>
                    <th>Description</th>
                    <th>Interface</th>
                    <th>MAC Address</th>
                    <th class="text-right">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {#each hosts as host}
                    <tr class="hover">
                      <td>{host.descr || "Unnamed Device"}</td>
                      <td>{host.interface}</td>
                      <td class="font-mono">{host.mac}</td>
                      <td class="text-right">
                        <div class="flex justify-end gap-2">
                          <button 
                            class="btn btn-sm btn-primary"
                            on:click={() => wakeDevice(host.uuid, host.descr)}
                            disabled={isWaking}
                            title="Wake device"
                          >
                            {#if isWaking}
                              <span class="loading loading-spinner loading-xs"></span>
                            {:else}
                              <svg class="w-4 h-4" viewBox="0 0 24 24">
                                <path fill="currentColor" d={mdiPower} />
                              </svg>
                            {/if}
                          </button>
                          <button 
                            class="btn btn-sm btn-error"
                            on:click={() => openDeleteConfirm(host)}
                            disabled={isDeleting}
                            title="Delete device"
                          >
                            <svg class="w-4 h-4" viewBox="0 0 24 24">
                              <path fill="currentColor" d={mdiDelete} />
                            </svg>
                          </button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
            
            <!-- Card layout for mobile -->
            <div class="sm:hidden p-2 space-y-3">
              {#each hosts as host}
                <div class="card bg-base-200">
                  <div class="card-body p-4">
                    <h3 class="card-title text-base">
                      {host.descr || "Unnamed Device"}
                    </h3>
                    <div class="grid grid-cols-[auto,1fr] gap-x-2 text-sm">
                      <div class="font-medium">Interface:</div>
                      <div>{host.interface}</div>
                      
                      <div class="font-medium">MAC:</div>
                      <div class="font-mono text-xs break-all">{host.mac}</div>
                    </div>
                    <div class="card-actions justify-end mt-2">
                      <button 
                        class="btn btn-sm btn-primary"
                        on:click={() => wakeDevice(host.uuid, host.descr)}
                        disabled={isWaking}
                      >
                        {#if isWaking}
                          <span class="loading loading-spinner loading-xs"></span>
                        {:else}
                          <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                            <path fill="currentColor" d={mdiPower} />
                          </svg>
                          Wake
                        {/if}
                      </button>
                      <button 
                        class="btn btn-sm btn-error"
                        on:click={() => openDeleteConfirm(host)}
                        disabled={isDeleting}
                      >
                        <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiDelete} />
                        </svg>
                        Delete
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Add Device Modal -->
  {#if showAddModal}
    <div class="modal modal-open">
      <div class="modal-box max-w-sm w-full">
        <h3 class="font-bold text-lg mb-4">Add Wake-on-LAN Device</h3>
        <form on:submit|preventDefault={addHost} class="space-y-4">
          <!-- Unified Device Selection/MAC Input -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">Device</span>
            </label>
            <div class="relative">
              <div class="flex items-center">
                <input
                  type="text"
                  class="input input-bordered w-full font-mono pr-24"
                  placeholder="00:11:22:33:44:55"
                  bind:value={newHost.mac}
                  on:input={(e) => newHost.mac = formatMacAddress(e.target.value)}
                  inputmode="text"
                  autocapitalize="none"
                  required
                />
                <div class="dropdown dropdown-end absolute right-0 mr-1">
                  <label tabindex="0" class="btn btn-sm m-1">Select</label>
                  <div tabindex="0" class="dropdown-content z-[1] shadow bg-base-100 rounded-box w-80 max-h-60 overflow-y-auto p-2 mt-1">
                    <ul class="space-y-1 w-full">
                    {#if arpDevices.length === 0}
                      <li class="menu-title">No devices found</li>
                    {:else}
                      <li class="menu-title">Select from network</li>
                      {#each arpDevices as device}
                        <li class="py-1">
                          <button 
                            on:click|preventDefault={() => handleDeviceSelection(device)}
                            class="w-full text-start p-2 hover:bg-base-200 rounded-md"
                          >
                            <div class="font-medium truncate">{device.hostname || device.ip}</div>
                            <div class="text-xs opacity-75 font-mono break-all">{device.mac}</div>
                            {#if device.interface}
                              <div class="text-xs opacity-50 truncate">Interface: {device.interface}</div>
                            {/if}
                          </button>
                        </li>
                      {/each}
                    {/if}
                    </ul>
                  </div>
                </div>
              </div>
              <div class="text-xs opacity-70 mt-1">Format: XX:XX:XX:XX:XX:XX</div>
            </div>
          </div>
          
          <div class="form-control">
            <label class="label" for="interface">
              <span class="label-text">Interface</span>
            </label>
            <select 
              id="interface"
              class="select select-bordered w-full"
              bind:value={newHost.interface}
              required
            >
              <option value="" disabled>Select an interface</option>
              {#each Object.entries(interfaces) as [key, option]}
                <option value={key}>{option.value}</option>
              {/each}
            </select>
          </div>
          
          <div class="form-control">
            <label class="label" for="description">
              <span class="label-text">Description</span>
            </label>
            <input
              type="text"
              id="description"
              class="input input-bordered w-full"
              placeholder="My Device"
              bind:value={newHost.descr}
            />
          </div>
          
          <div class="modal-action flex sm:justify-end justify-between mt-6">
            <button
              type="button"
              class="btn sm:btn-md btn-sm flex-1 sm:flex-none"
              on:click={closeAddModal}
              disabled={isAdding}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="btn btn-primary sm:btn-md btn-sm flex-1 sm:flex-none"
              disabled={isAdding || !newHost.mac || !newHost.interface}
            >
              {#if isAdding}
                <span class="loading loading-spinner loading-xs"></span>
              {/if}
              Add Device
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
  
  <!-- Delete Confirmation Modal -->
  {#if showDeleteConfirm && hostToDelete}
    <div class="modal modal-open">
      <div class="modal-box max-w-sm">
        <h3 class="font-bold text-lg mb-4">Delete Wake-on-LAN Device</h3>
        <p>Are you sure you want to delete this device?</p>
        
        <div class="bg-base-200 p-3 rounded-lg my-4">
          <div class="font-medium">{hostToDelete.descr || "Unnamed Device"}</div>
          <div class="text-sm font-mono">{hostToDelete.mac}</div>
          <div class="text-xs mt-1">Interface: {hostToDelete.interface}</div>
        </div>
        
        <p class="text-sm text-warning">This action cannot be undone.</p>
        
        <div class="modal-action">
          <button
            type="button"
            class="btn btn-outline"
            on:click={closeDeleteConfirm}
            disabled={isDeleting}
          >
            <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiClose} />
            </svg>
            Cancel
          </button>
          <button
            type="button"
            class="btn btn-error"
            on:click={deleteHost}
            disabled={isDeleting}
          >
            {#if isDeleting}
              <span class="loading loading-spinner loading-xs"></span>
            {:else}
              <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiDelete} />
              </svg>
              Delete
            {/if}
          </button>
        </div>
      </div>
    </div>
  {/if}
  
</AppLayout>