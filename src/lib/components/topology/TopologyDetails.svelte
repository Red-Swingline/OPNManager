<script lang="ts">
  import type { Interface, CombinedDevice } from "./types";
  import { 
    mdiClose, 
    mdiEthernet, 
    mdiAccessPoint, 
    mdiLan, 
    mdiLanConnect,
    mdiCheckCircle,
    mdiCloseCircle,
    mdiAlertCircle,
    mdiLanDisconnect
  } from "@mdi/js";
  
  export let element: Interface | CombinedDevice;
  export let elementType: 'interface' | 'device';
  export let onClose: () => void;
  
  function formatIpAddress(ipAddr: string): string {
    return ipAddr.split('/')[0];
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
        icon: mdiLan,
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

  // Helper function to copy text to clipboard
  async function copyToClipboard(text: string, description: string) {
    try {
      await navigator.clipboard.writeText(text);
      // Could use a toast here, but avoiding dependencies for simplicity
      console.log(`${description} copied to clipboard`);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  }
</script>

<div class="p-4 h-full flex flex-col">
  <div class="flex justify-between items-center mb-4">
    <h2 class="text-xl font-bold">
      {elementType === 'interface' 
        ? (element as Interface).description || (element as Interface).device 
        : (element as CombinedDevice).hostname || "Device Details"}
    </h2>
    <button class="btn btn-sm btn-circle" on:click={onClose}>
      <svg class="w-4 h-4" viewBox="0 0 24 24">
        <path fill="currentColor" d={mdiClose} />
      </svg>
    </button>
  </div>
  
  <div class="overflow-y-auto flex-grow">
    {#if elementType === 'interface'}
      {@const iface = element as Interface}
      <div class="space-y-4">
        <!-- Basic Info -->
        <div class="bg-base-200 p-3 rounded-lg">
          <div class="flex items-center mb-2">
            {#if true}
              {@const type = getInterfaceTypeIcon(iface)}
              <svg class="w-5 h-5 mr-2" viewBox="0 0 24 24">
                <path fill="currentColor" d={type.icon} />
              </svg>
              <span class="font-medium">{type.text} Interface</span>
            {/if}
            
            <div class="ml-auto">
              {#if true}
                {@const status = getStatusIcon(iface.status)}
                <div class="flex items-center">
                  <svg class="w-5 h-5 mr-1 {status.color}" viewBox="0 0 24 24">
                    <path fill="currentColor" d={status.icon} />
                  </svg>
                  <span>{status.text}</span>
                </div>
              {/if}
            </div>
          </div>
          
          <div class="grid grid-cols-2 gap-2 text-sm">
            <div>
              <span class="text-base-content/60">Name:</span>
              <span class="font-medium ml-1">{iface.device}</span>
            </div>
            {#if iface.identifier}
              <div>
                <span class="text-base-content/60">Identifier:</span>
                <span class="font-medium ml-1">{iface.identifier}</span>
              </div>
            {/if}
            <div class="col-span-2">
              <span class="text-base-content/60">MAC Address:</span>
              <span class="font-mono ml-1">{iface.macaddr}</span>
            </div>
            {#if iface.mtu}
              <div>
                <span class="text-base-content/60">MTU:</span>
                <span class="font-medium ml-1">{iface.mtu}</span>
              </div>
            {/if}
            {#if iface.media}
              <div class="col-span-2">
                <span class="text-base-content/60">Media:</span>
                <span class="ml-1">{@html iface.media}</span>
              </div>
            {/if}
          </div>
        </div>
        
        <!-- IP Addresses -->
        <div>
          <h3 class="text-lg font-medium mb-2">IP Configuration</h3>
          
          {#if iface.ipv4 && iface.ipv4.length > 0}
            <div class="mb-3">
              <h4 class="text-base font-medium mb-1">IPv4 Addresses</h4>
              <div class="bg-base-200 p-2 rounded-lg">
                {#each iface.ipv4 as ip}
                  <div class="flex items-center justify-between py-1">
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
            </div>
          {/if}
          
          {#if iface.ipv6 && iface.ipv6.length > 0}
            <div class="mb-3">
              <h4 class="text-base font-medium mb-1">IPv6 Addresses</h4>
              <div class="bg-base-200 p-2 rounded-lg">
                {#each iface.ipv6 as ip}
                  <div class="flex items-center justify-between py-1">
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
            </div>
          {/if}
          
          {#if iface.gateways && iface.gateways.length > 0}
            <div>
              <h4 class="text-base font-medium mb-1">Gateways</h4>
              <div class="bg-base-200 p-2 rounded-lg">
                {#each iface.gateways as gateway}
                  <div class="flex items-center justify-between py-1">
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
            </div>
          {/if}
        </div>
        
        <!-- Flags -->
        {#if iface.flags && iface.flags.length > 0}
          <div>
            <h3 class="text-lg font-medium mb-2">Interface Flags</h3>
            <div class="flex flex-wrap gap-1">
              {#each iface.flags as flag}
                <span class="badge badge-sm">{flag}</span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {:else}
      {@const device = element as CombinedDevice}
      <div class="space-y-4">
        <!-- Basic Info -->
        <div class="bg-base-200 p-3 rounded-lg">
          <div class="grid grid-cols-2 gap-2 text-sm">
            <div class="col-span-2">
              <span class="text-base-content/60">MAC Address:</span>
              <span class="font-mono ml-1">{device.mac}</span>
              <button 
                class="btn btn-ghost btn-xs"
                on:click={() => copyToClipboard(device.mac, "MAC address")}
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
            <div class="col-span-2">
              <span class="text-base-content/60">Interface:</span>
              <span class="font-medium ml-1">{device.intf}</span>
              {#if device.intf_description}
                <span class="text-xs text-base-content/60 ml-1">({device.intf_description})</span>
              {/if}
            </div>
            {#if device.hostname}
              <div class="col-span-2">
                <span class="text-base-content/60">Hostname:</span>
                <span class="font-medium ml-1">{device.hostname}</span>
              </div>
            {/if}
            {#if device.manufacturer}
              <div class="col-span-2">
                <span class="text-base-content/60">Manufacturer:</span>
                <span class="ml-1">{device.manufacturer}</span>
              </div>
            {/if}
            {#if device.device_type}
              <div>
                <span class="text-base-content/60">Type:</span>
                <span class="ml-1">{device.device_type}</span>
              </div>
            {/if}
            {#if device.permanent !== undefined}
              <div>
                <span class="text-base-content/60">Permanent:</span>
                <span class="ml-1">{device.permanent ? "Yes" : "No"}</span>
              </div>
            {/if}
            {#if device.expired !== undefined}
              <div>
                <span class="text-base-content/60">Status:</span>
                <span class="ml-1 {device.expired ? 'text-error' : 'text-success'}">
                  {device.expired ? "Expired" : "Active"}
                </span>
              </div>
            {/if}
          </div>
        </div>
        
        <!-- IP Addresses -->
        <div>
          <h3 class="text-lg font-medium mb-2">IP Addresses</h3>
          
          {#if device.ipv4_addresses && device.ipv4_addresses.length > 0}
            <div class="mb-3">
              <h4 class="text-base font-medium mb-1">IPv4 Addresses</h4>
              <div class="bg-base-200 p-2 rounded-lg">
                {#each device.ipv4_addresses as ip}
                  <div class="flex items-center justify-between py-1">
                    <span class="font-mono text-sm">{ip}</span>
                    <button 
                      class="btn btn-ghost btn-xs"
                      on:click={() => copyToClipboard(ip, "IPv4 address")}
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
            </div>
          {/if}
          
          {#if device.ipv6_addresses && device.ipv6_addresses.length > 0}
            <div>
              <h4 class="text-base font-medium mb-1">IPv6 Addresses</h4>
              <div class="bg-base-200 p-2 rounded-lg">
                {#each device.ipv6_addresses as ip}
                  <div class="flex items-center justify-between py-1">
                    <span class="font-mono text-sm break-all">{ip}</span>
                    <button 
                      class="btn btn-ghost btn-xs flex-shrink-0"
                      on:click={() => copyToClipboard(ip, "IPv6 address")}
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
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>