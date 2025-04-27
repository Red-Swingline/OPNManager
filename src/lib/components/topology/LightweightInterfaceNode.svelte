<script lang="ts">
  import { Handle } from '@xyflow/svelte';
  import { createEventDispatcher } from 'svelte';
  import type { Interface } from './types';
  import { 
    mdiAccessPoint, 
    mdiEthernet, 
    mdiLan, 
    mdiServerNetwork 
  } from '@mdi/js';

  export let data: {
    interfaceData: Interface;
    label: string;
  };
  
  const dispatch = createEventDispatcher();
  
  function handleNodeClick(event) {
    // Stop propagation to prevent SvelteFlow from capturing it
    event.stopPropagation();
    
    // Dispatch event to show details in the modal with proper structure
    dispatch('elementSelect', { element: iface, type: 'interface' });
    
    // Also send a DOM event as backup
    const customEvent = new CustomEvent('interface:select', { 
      bubbles: true, 
      composed: true,
      detail: { element: iface, type: 'interface' }
    });
    event.target.dispatchEvent(customEvent);
    
    console.log('Interface node clicked, dispatched elementSelect event with:', iface);
  }

  // Quick reference to interface data
  const iface = data.interfaceData;
  const status = iface.status?.toLowerCase() || 'unknown';
  
  // Set colors based on interface status
  let statusColor = 'gray';
  let statusLabel = 'Unknown';
  
  switch (status) {
    case 'up':
      statusColor = 'green';
      statusLabel = 'Up';
      break;
    case 'down':
      statusColor = 'red';
      statusLabel = 'Down';
      break;
    case 'no carrier':
      statusColor = 'amber';
      statusLabel = 'No Carrier';
      break;
  }
  
  // Determine interface type
  let interfaceType = 'ethernet';
  if (iface.device?.includes('wlan') || iface.device?.includes('wifi') || 
      (iface.description && iface.description.toLowerCase().includes('wireless'))) {
    interfaceType = 'wireless';
  } else if (iface.device?.includes('vlan') || iface.device?.includes('_vlan')) {
    interfaceType = 'vlan';
  } else if (iface.device?.includes('bridge') || iface.device?.includes('ovpn') || 
      iface.device?.includes('tun') || iface.device?.includes('wg')) {
    interfaceType = 'virtual';
  }
</script>

<!-- Connection handle at top -->
<Handle type="target" position="top" class="w-0 h-0" />

<div class="interface-node">
  <div 
    class="interface-card interface-status-{statusColor} interface-type-{interfaceType}" 
    on:click={handleNodeClick}
  >
    <div class="interface-header">
      <span class="interface-type-icon">
        <svg viewBox="0 0 24 24" class="interface-type-svg">
          <path fill="currentColor" d={
            interfaceType === 'wireless' ? mdiAccessPoint : 
            interfaceType === 'vlan' ? mdiLan : 
            interfaceType === 'virtual' ? mdiServerNetwork : 
            mdiEthernet
          } />
        </svg>
      </span>
      <span class="interface-status">{statusLabel}</span>
      <span class="interface-id">{iface.device}</span>
    </div>
    
    <div class="interface-content">
      <div class="interface-name" title={data.label}>
        {data.label}
      </div>
      
      {#if iface.ipv4 && iface.ipv4.length > 0}
        <div class="interface-ip" title={iface.ipv4[0].ipaddr}>
          {iface.ipv4[0].ipaddr}
        </div>
      {/if}
      
      <div class="interface-details">
        <span class="interface-indicator {iface.is_physical ? 'physical' : 'virtual'}"></span>
        <span class="interface-type-label">{iface.is_physical ? 'Physical' : 'Virtual'}</span>
      </div>
    </div>
  </div>
</div>

<!-- Connection handle at bottom -->
<Handle type="source" position="bottom" class="w-0 h-0" />

<style>
  .interface-node {
    width: 120px;
    padding: 2px;
    font-family: -apple-system, system-ui, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    -webkit-user-select: none;
    user-select: none;
  }
  
  .interface-card {
    display: flex;
    flex-direction: column;
    border-radius: 8px;
    background: white;
    border: 2px solid transparent;
    box-shadow: 0 2px 6px rgba(0,0,0,0.1);
    overflow: hidden;
    cursor: grab;
  }
  
  .interface-header {
    display: flex;
    align-items: center;
    padding: 8px;
    color: white;
    position: relative;
  }
  
  .interface-type-icon {
    margin-right: 6px;
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background-color: rgba(255,255,255,0.15);
  }
  
  .interface-type-svg {
    width: 16px;
    height: 16px;
  }
  
  .interface-status {
    font-size: 12px;
    font-weight: 500;
  }
  
  .interface-id {
    font-size: 9px;
    padding: 2px 5px;
    background: rgba(0,0,0,0.15);
    border-radius: 4px;
    margin-left: auto;
  }
  
  .interface-content {
    padding: 10px;
  }
  
  .interface-name {
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .interface-ip {
    font-size: 10px;
    opacity: 0.7;
    margin-bottom: 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .interface-details {
    display: flex;
    align-items: center;
    font-size: 10px;
    opacity: 0.7;
  }
  
  .interface-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-right: 5px;
  }
  
  .interface-indicator.physical {
    background-color: #2196f3;
  }
  
  .interface-indicator.virtual {
    background-color: #9c27b0;
  }
  
  /* Interface types now use direct text content */
  
  /* Status colors */
  .interface-status-green .interface-header {
    background: linear-gradient(to right, #4caf50, #8bc34a);
  }
  
  .interface-status-red .interface-header {
    background: linear-gradient(to right, #f44336, #e91e63);
  }
  
  .interface-status-amber .interface-header {
    background: linear-gradient(to right, #ff9800, #ff5722);
  }
  
  .interface-status-gray .interface-header {
    background: linear-gradient(to right, #9e9e9e, #607d8b);
  }
  
  /* Border highlights based on status */
  .interface-status-green {
    border-color: rgba(76, 175, 80, 0.3);
  }
  
  .interface-status-red {
    border-color: rgba(244, 67, 54, 0.3);
  }
  
  .interface-status-amber {
    border-color: rgba(255, 152, 0, 0.3);
  }
  
  /* Interactive hover effect */
  .interface-card:hover {
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    transform: translateY(-2px);
    transition: all 0.2s ease;
  }
  
  /* Dark mode */
  :global([data-theme="dark"]) .interface-card {
    background: #2a303c;
    border-color: rgba(255,255,255,0.1);
  }
  
  :global([data-theme="dark"]) .interface-name {
    color: rgba(255,255,255,0.9);
  }
  
  :global([data-theme="dark"]) .interface-ip,
  :global([data-theme="dark"]) .interface-details {
    color: rgba(255,255,255,0.6);
  }
  
  /* Hide handles visually but keep them functional */
  :global(.svelte-flow__handle) {
    opacity: 0;
    pointer-events: all;
  }
</style>