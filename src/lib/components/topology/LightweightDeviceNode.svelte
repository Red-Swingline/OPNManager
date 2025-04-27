<script lang="ts">
  import { Handle } from '@xyflow/svelte';
  import { onMount, createEventDispatcher } from 'svelte';
  import type { CombinedDevice } from './types';
  import { 
    mdiLaptop, 
    mdiCellphone, 
    mdiServerNetwork, 
    mdiTelevision, 
    mdiDotsHorizontal 
  } from '@mdi/js';

  export let data: {
    deviceData: CombinedDevice;
    label: string;
  };
  
  const dispatch = createEventDispatcher();

  // Quick reference to device data
  const device = data.deviceData;
  const isIndicator = device.indicator || device.isIndicator;
  const isExpired = device.expired;
  const isPermanent = device.permanent;
  
  // Handle click to show details in the parent modal
  function handleNodeClick(event) {
    // Don't handle clicks for indicator nodes
    if (isIndicator) return;
    
    // Stop propagation to prevent SvelteFlow from capturing it
    event.stopPropagation();
    
    // Dispatch event to show details in the modal with proper structure
    dispatch('elementSelect', { element: device, type: 'device' });
    
    // Also send a DOM event as backup
    const customEvent = new CustomEvent('device:select', { 
      bubbles: true, 
      composed: true,
      detail: { element: device, type: 'device' }
    });
    event.target.dispatchEvent(customEvent);
    
    console.log('Device node clicked, dispatched elementSelect event with:', device);
  }

  // Determine styling based on device type/status
  let deviceTypeClass = 'device-type-laptop'; // Default
  let deviceColor = 'purple';

  // Set device type based on hostname/MAC pattern
  if (device.hostname) {
    const hostname = device.hostname.toLowerCase();
    if (hostname.includes('phone') || hostname.includes('iphone') || hostname.includes('android')) {
      deviceTypeClass = 'device-type-phone';
    } else if (hostname.includes('tv') || hostname.includes('roku') || hostname.includes('fire')) {
      deviceTypeClass = 'device-type-tv';
      deviceColor = 'green';
    } else if (hostname.includes('server') || hostname.includes('nas') || hostname.includes('pfsense')) {
      deviceTypeClass = 'device-type-server';
      deviceColor = 'orange';
    }
  }

  // Set status colors
  if (isPermanent) {
    deviceColor = 'blue';
  } else if (isExpired) {
    deviceColor = 'gray';
  }

  // For indicator nodes (showing hidden devices)
  if (isIndicator) {
    deviceTypeClass = 'device-type-more';
    deviceColor = 'gray';
  }
</script>

<!-- Handle for connection points - invisible but functional -->
<Handle type="target" position="top" class="w-0 h-0" />

<div class="device-node" data-indicator={isIndicator}>
  <div 
    class="node-card device-color-{deviceColor} {deviceTypeClass}" 
    on:click={handleNodeClick}
  >
    <div class="device-icon-container">
      <svg viewBox="0 0 24 24" class="device-icon-svg">
        <path fill="currentColor" d={
          deviceTypeClass === 'device-type-phone' ? mdiCellphone : 
          deviceTypeClass === 'device-type-server' ? mdiServerNetwork : 
          deviceTypeClass === 'device-type-tv' ? mdiTelevision : 
          deviceTypeClass === 'device-type-more' ? mdiDotsHorizontal : 
          mdiLaptop
        } />
      </svg>
    </div>
    <div class="device-content">
      <div class="device-name" title={data.label}>
        {data.label}
      </div>
      
      {#if !isIndicator && device.ipv4_addresses && device.ipv4_addresses.length > 0}
        <div class="device-ip" title={device.ipv4_addresses[0]}>
          {device.ipv4_addresses[0]}
        </div>
      {/if}
    </div>
  </div>
</div>

<Handle type="source" position="bottom" class="w-0 h-0" />

<style>
  .device-node {
    width: 100px;
    padding: 2px;
    font-family: -apple-system, system-ui, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    -webkit-user-select: none;
    user-select: none;
  }
  
  .node-card {
    display: flex;
    flex-direction: column;
    border-radius: 8px;
    background: #fff;
    border: 1px solid rgba(0,0,0,0.1);
    box-shadow: 0 2px 5px rgba(0,0,0,0.08);
    overflow: hidden;
    cursor: grab;
  }
  
  .device-icon-container {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }
  
  .device-icon-svg {
    width: 18px;
    height: 18px;
  }
  
  .device-content {
    padding: 8px;
    text-align: center;
  }
  
  .device-name {
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .device-ip {
    font-size: 10px;
    opacity: 0.7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 4px;
  }
  
  /* We're now using MDI icons via SVG paths */
  
  /* Device colors */
  .device-color-purple .device-icon-container {
    background-color: #673ab7;
    color: white;
  }
  
  .device-color-blue .device-icon-container {
    background-color: #2196f3;
    color: white;
  }
  
  .device-color-green .device-icon-container {
    background-color: #4caf50;
    color: white;
  }
  
  .device-color-orange .device-icon-container {
    background-color: #ff9800;
    color: white;
  }
  
  .device-color-gray .device-icon-container {
    background-color: #9e9e9e;
    color: white;
  }
  
  /* Hover effects for better interactivity */
  .node-card:hover {
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    transform: translateY(-2px);
    transition: all 0.2s ease;
  }
  
  /* Dark mode support */
  :global([data-theme="dark"]) .node-card {
    background: #2a303c;
    border-color: rgba(255,255,255,0.1);
  }
  
  :global([data-theme="dark"]) .device-name {
    color: rgba(255,255,255,0.9);
  }
  
  :global([data-theme="dark"]) .device-ip {
    color: rgba(255,255,255,0.6);
  }
  
  
  /* Hide handles visually but keep them functional */
  :global(.svelte-flow__handle) {
    opacity: 0;
    pointer-events: all;
  }
</style>