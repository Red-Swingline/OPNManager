<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AppLayout from "../AppLayout.svelte";
  import { 
    mdiEthernet, 
    mdiCheckCircle, 
    mdiCloseCircle, 
    mdiArrowUp, 
    mdiArrowDown, 
    mdiLanDisconnect, 
    mdiLanConnect, 
    mdiInformation, 
    mdiAlertCircle
  } from "@mdi/js";
  import { toasts } from "$lib/stores/toastStore";

  interface Interface {
    device: string;
    description: string;
    identifier: string;
    status: string;
    is_physical: boolean;
    macaddr: string;
    ipv4: IpAddress[];
    ipv6: IpAddress[];
    enabled: boolean;
    vlan_tag?: string;
    flags: string[];
    mtu: string;
    media?: string;
    media_raw?: string;
    link_type?: string;
    addr4?: string;
    addr6?: string;
    gateways: string[];
  }

  interface IpAddress {
    ipaddr: string;
  }

  let interfaces: Interface[] = [];
  let isLoading = true;
  let error: string | null = null;
  let selectedInterface: Interface | null = null;
  let showModal = false;

  onMount(async () => {
    await fetchInterfaces();
  });

  async function fetchInterfaces() {
    isLoading = true;
    error = null;
    try {
      interfaces = await invoke<Interface[]>("get_interfaces");
    } catch (err) {
      console.error("Failed to fetch interfaces:", err);
      error = `Failed to fetch interfaces: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  function showInterfaceDetails(iface: Interface) {
    selectedInterface = iface;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    setTimeout(() => {
      selectedInterface = null;
    }, 300);
  }

  function getStatusIcon(status: string) {
    switch (status.toLowerCase()) {
      case "up":
        return {
          icon: mdiCheckCircle,
          color: "text-success",
          text: "Up"
        };
      case "down":
        return {
          icon: mdiCloseCircle,
          color: "text-error",
          text: "Down"
        };
      case "no carrier":
        return {
          icon: mdiLanDisconnect,
          color: "text-warning",
          text: "No Carrier"
        };
      default:
        return {
          icon: mdiAlertCircle,
          color: "text-base-content/50",
          text: status
        };
    }
  }

  function getInterfaceTypeIcon(iface: Interface) {
    // VLAN interface
    if (iface.vlan_tag) {
      return {
        icon: mdiLanConnect,
        text: `VLAN ${iface.vlan_tag}`
      };
    }
    
    // Physical interface
    if (iface.is_physical) {
      return {
        icon: mdiEthernet,
        text: "Physical"
      };
    }
    
    // Virtual/Other interface
    return {
      icon: mdiLanConnect,
      text: "Virtual"
    };
  }

  async function copyToClipboard(text: string, description: string) {
    try {
      await navigator.clipboard.writeText(text);
      toasts.success(`${description} copied to clipboard`);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
      toasts.error("Failed to copy to clipboard");
    }
  }

  // Helper to format IP addresses nicely
  function formatIpAddress(ipAddr: string): string {
    return ipAddr.split('/')[0];
  }
</script>

<AppLayout>
  <div class="p-4">
    <h1 class="text-2xl font-bold mb-4">Network Interfaces</h1>

    {#if isLoading}
      <div class="flex justify-center items-center py-12">
        <div class="text-center">
          <span class="loading loading-spinner loading-lg"></span>
          <p class="mt-4 text-base-content">Loading interfaces...</p>
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
          <button class="btn btn-sm" on:click={fetchInterfaces}>Retry</button>
        </div>
      </div>
    {:else if interfaces.length === 0}
      <div class="flex justify-center items-center py-12">
        <div class="text-center p-6 bg-base-200 rounded-lg max-w-md">
          <svg class="w-12 h-12 mx-auto text-base-content/50" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiEthernet} />
          </svg>
          <h3 class="mt-4 text-lg font-medium">No interfaces found</h3>
          <p class="mt-2 text-base-content/70">
            No network interfaces were detected on your firewall.
          </p>
          <button class="btn btn-primary mt-4" on:click={fetchInterfaces}>
            Refresh Interfaces
          </button>
        </div>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each interfaces as iface}
          <div 
            class="card bg-base-100 shadow-md hover:shadow-lg transition-shadow duration-200"
          >
            <div class="card-body p-4">
              <div class="flex justify-between items-start">
                <h2 class="card-title">
                  {iface.description || iface.device}
                  {#if iface.vlan_tag}
                    <span class="badge badge-outline">VLAN {iface.vlan_tag}</span>
                  {/if}
                </h2>
                
                <!-- Status indicator -->
                <div class="flex items-center">
                  {#if true}
                    {@const status = getStatusIcon(iface.status)}
                    <div class="tooltip" data-tip={status.text}>
                      <svg class="w-5 h-5 {status.color}" viewBox="0 0 24 24">
                        <path fill="currentColor" d={status.icon} />
                      </svg>
                    </div>
                  {/if}
                </div>
              </div>
              
              <div class="text-sm text-base-content/70 mb-2">
                <div class="flex items-center mb-1">
                  {#if true}
                    {@const type = getInterfaceTypeIcon(iface)}
                    <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                      <path fill="currentColor" d={type.icon} />
                    </svg>
                    <span class="mr-2">{type.text}</span>
                  {/if}
                  <span class="font-mono">{iface.device}</span>
                </div>
              </div>

              <!-- IP Addresses -->
              <div class="space-y-1 mb-2">
                {#if iface.ipv4 && iface.ipv4.length > 0}
                  <div class="text-sm flex items-start">
                    <div class="w-8 flex-shrink-0">
                      <span>IPv4:</span>
                    </div>
                    <div class="font-mono text-xs break-all">
                      {#each iface.ipv4 as ip, i}
                        <div>
                          {formatIpAddress(ip.ipaddr)}
                          {#if i < iface.ipv4.length - 1}
                            ,
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
                
                {#if iface.ipv6 && iface.ipv6.length > 0}
                  <div class="text-sm flex items-start">
                    <div class="w-8 flex-shrink-0">
                      <span>IPv6:</span>
                    </div>
                    <div class="font-mono text-xs break-all">
                      {#if iface.ipv6.filter(ip => !ip.ipaddr.startsWith('fe80::')).length > 0}
                        {#each iface.ipv6.filter(ip => !ip.ipaddr.startsWith('fe80::')).slice(0, 1) as ip}
                          <div>
                            {formatIpAddress(ip.ipaddr)}
                            {#if iface.ipv6.length > 1}
                              <span class="text-xs text-base-content/50">+{iface.ipv6.length - 1} more</span>
                            {/if}
                          </div>
                        {/each}
                      {:else if iface.ipv6.length > 0}
                        <div>
                          {formatIpAddress(iface.ipv6[0].ipaddr)}
                          {#if iface.ipv6.length > 1}
                            <span class="text-xs text-base-content/50">+{iface.ipv6.length - 1} more</span>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}
              </div>
              
              <div class="card-actions justify-end mt-2">
                <button 
                  class="btn btn-sm btn-primary"
                  on:click={() => showInterfaceDetails(iface)}
                >
                  <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                    <path fill="currentColor" d={mdiInformation} />
                  </svg>
                  Details
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</AppLayout>

<!-- Interface Detail Modal -->
{#if showModal && selectedInterface}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
    on:click|self={closeModal}
    on:keydown={(e) => e.key === 'Escape' && closeModal()}
  >
    <div 
      class="bg-base-100 rounded-lg shadow-xl max-w-3xl w-full max-h-[90vh] overflow-y-auto"
      on:click|stopPropagation
    >
      <div class="sticky top-0 bg-base-100 px-4 py-3 border-b border-base-300 flex justify-between items-center z-10">
        <h2 class="text-xl font-bold">
          {selectedInterface.description || selectedInterface.device}
        </h2>
        <button 
          class="btn btn-sm btn-circle" 
          on:click={closeModal}
        >✕</button>
      </div>
      
      <div class="p-4">
        <!-- Basic Information Section -->
        <div class="mb-4">
          <h3 class="text-lg font-semibold mb-2 border-b border-base-300 pb-1">
            Basic Information
          </h3>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-2">
            <div class="flex justify-between">
              <span class="font-medium">Device:</span>
              <div class="flex items-center">
                <span class="font-mono">{selectedInterface.device}</span>
                <button 
                  class="btn btn-ghost btn-xs"
                  on:click={() => copyToClipboard(selectedInterface.device, "Device name")}
                  title="Copy device name"
                >
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path
                      fill="currentColor"
                      d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"
                    />
                  </svg>
                </button>
              </div>
            </div>
            
            <div class="flex justify-between">
              <span class="font-medium">Status:</span>
              <div class="flex items-center">
                {#if true}
                  {@const status = getStatusIcon(selectedInterface.status)}
                  <svg class="w-4 h-4 mr-1 {status.color}" viewBox="0 0 24 24">
                    <path fill="currentColor" d={status.icon} />
                  </svg>
                  <span>{status.text}</span>
                {/if}
              </div>
            </div>
            
            <div class="flex justify-between">
              <span class="font-medium">Type:</span>
              <span>
                {#if selectedInterface.vlan_tag}
                  VLAN Interface (tag: {selectedInterface.vlan_tag})
                {:else if selectedInterface.is_physical}
                  Physical Interface
                {:else}
                  Virtual Interface
                {/if}
              </span>
            </div>
            
            <div class="flex justify-between">
              <span class="font-medium">Link Type:</span>
              <span>{selectedInterface.link_type || "Not set"}</span>
            </div>
            
            <div class="flex justify-between">
              <span class="font-medium">MTU:</span>
              <span>{selectedInterface.mtu}</span>
            </div>
            
            <div class="flex justify-between">
              <span class="font-medium">Media:</span>
              <span>{@html selectedInterface.media || "Not available"}</span>
            </div>
          </div>
        </div>
        
        <!-- Network Information Section -->
        <div class="mb-4">
          <h3 class="text-lg font-semibold mb-2 border-b border-base-300 pb-1">
            Network Information
          </h3>
          
          <!-- IPv4 Addresses -->
          <div class="mb-3">
            <h4 class="text-base font-medium mb-1">IPv4 Addresses</h4>
            {#if selectedInterface.ipv4 && selectedInterface.ipv4.length > 0}
              <div class="space-y-1">
                {#each selectedInterface.ipv4 as ip}
                  <div class="flex items-center">
                    <span class="font-mono text-sm">{ip.ipaddr}</span>
                    <button 
                      class="btn btn-ghost btn-xs"
                      on:click={() => copyToClipboard(formatIpAddress(ip.ipaddr), "IPv4 address")}
                      title="Copy IPv4 address"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path
                          fill="currentColor"
                          d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"
                        />
                      </svg>
                    </button>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-sm italic text-base-content/70">No IPv4 addresses</div>
            {/if}
          </div>
          
          <!-- IPv6 Addresses -->
          <div class="mb-3">
            <h4 class="text-base font-medium mb-1">IPv6 Addresses</h4>
            {#if selectedInterface.ipv6 && selectedInterface.ipv6.length > 0}
              <div class="space-y-1">
                {#each selectedInterface.ipv6 as ip}
                  <div class="flex items-center">
                    <span class="font-mono text-sm break-all">{ip.ipaddr}</span>
                    <button 
                      class="btn btn-ghost btn-xs flex-shrink-0"
                      on:click={() => copyToClipboard(formatIpAddress(ip.ipaddr), "IPv6 address")}
                      title="Copy IPv6 address"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path
                          fill="currentColor"
                          d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"
                        />
                      </svg>
                    </button>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-sm italic text-base-content/70">No IPv6 addresses</div>
            {/if}
          </div>
          
          <!-- Gateways -->
          <div>
            <h4 class="text-base font-medium mb-1">Gateways</h4>
            {#if selectedInterface.gateways && selectedInterface.gateways.length > 0}
              <div class="space-y-1">
                {#each selectedInterface.gateways as gateway}
                  <div class="flex items-center">
                    <span class="font-mono text-sm">{gateway}</span>
                    <button 
                      class="btn btn-ghost btn-xs"
                      on:click={() => copyToClipboard(gateway, "Gateway address")}
                      title="Copy gateway address"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path
                          fill="currentColor"
                          d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"
                        />
                      </svg>
                    </button>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-sm italic text-base-content/70">No gateways</div>
            {/if}
          </div>
        </div>
        
        <!-- Hardware Information Section -->
        <div>
          <h3 class="text-lg font-semibold mb-2 border-b border-base-300 pb-1">
            Hardware Information
          </h3>
          
          <div class="grid grid-cols-1 gap-y-2">
            <div class="flex justify-between">
              <span class="font-medium">MAC Address:</span>
              <div class="flex items-center">
                <span class="font-mono">{selectedInterface.macaddr}</span>
                <button 
                  class="btn btn-ghost btn-xs"
                  on:click={() => copyToClipboard(selectedInterface.macaddr, "MAC address")}
                  title="Copy MAC address"
                >
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path
                      fill="currentColor"
                      d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"
                    />
                  </svg>
                </button>
              </div>
            </div>
            
            {#if selectedInterface.flags && selectedInterface.flags.length > 0}
              <div>
                <span class="font-medium">Interface Flags:</span>
                <div class="flex flex-wrap gap-1 mt-1">
                  {#each selectedInterface.flags as flag}
                    <span class="badge badge-sm">{flag}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}