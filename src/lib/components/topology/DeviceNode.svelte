<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { Handle } from '@xyflow/svelte';
    import type { CombinedDevice } from './types';
  
    export let id;
    export let data;
    
    const dispatch = createEventDispatcher();
    const { label, deviceData } = data;
    
    function getDeviceColor(device: CombinedDevice): string {
      if (device.permanent) return '#2196F3'; // Blue
      if (device.expired) return '#9E9E9E'; // Gray
      return '#673AB7'; // Purple
    }
    
    function getDeviceIcon(device: CombinedDevice): string {
      const hostname = device.hostname?.toLowerCase() || '';
      
      if (hostname.includes('router')) {
        return 'M11,2A8,8 0 0,0 3,10V11H11V13H3V14A8,8 0 0,0 11,22H13A8,8 0 0,0 21,14V13H13V11H21V10A8,8 0 0,0 13,2H11M11,4H13A6,6 0 0,1 19,10H5A6,6 0 0,1 11,4Z';
      }
      if (hostname.includes('ap')) {
        return 'M12,21L15.6,16.2C14.6,15.45 13.35,15 12,15C10.65,15 9.4,15.45 8.4,16.2L12,21M12,3C7.95,3 4.21,4.34 1.2,6.6L3,9C5.5,7.12 8.62,6 12,6C15.38,6 18.5,7.12 21,9L22.8,6.6C19.79,4.34 16.05,3 12,3M12,9C9.3,9 6.81,9.89 4.8,11.4L6.6,13.8C8.1,12.67 9.97,12 12,12C14.03,12 15.9,12.67 17.4,13.8L19.2,11.4C17.19,9.89 14.7,9 12,9Z';
      }
      if (hostname.includes('server')) {
        return 'M4,1H20A1,1 0 0,1 21,2V6A1,1 0 0,1 20,7H4A1,1 0 0,1 3,6V2A1,1 0 0,1 4,1M4,9H20A1,1 0 0,1 21,10V14A1,1 0 0,1 20,15H4A1,1 0 0,1 3,14V10A1,1 0 0,1 4,9M4,17H20A1,1 0 0,1 21,18V22A1,1 0 0,1 20,23H4A1,1 0 0,1 3,22V18A1,1 0 0,1 4,17M9,5H10V3H9V5M9,13H10V11H9V13M9,21H10V19H9V21M5,3V5H7V3H5M5,11V13H7V11H5M5,19V21H7V19H5Z';
      }
      if (hostname.includes('phone')) {
        return 'M17,19H7V5H17M17,1H7C5.89,1 5,1.89 5,3V21A2,2 0 0,0 7,23H17A2,2 0 0,0 19,21V3C19,1.89 18.1,1 17,1Z';
      }
      if (hostname.includes('laptop')) {
        return 'M20,18C21.1,18 22,17.1 22,16V6C22,4.89 21.1,4 20,4H4C2.89,4 2,4.89 2,6V16A2,2 0 0,0 4,18H0V20H24V18H20M4,16V6H20V16H4Z';
      }
      if (hostname.includes('desktop')) {
        return 'M8,2H16A2,2 0 0,1 18,4V20A2,2 0 0,1 16,22H8A2,2 0 0,1 6,20V4A2,2 0 0,1 8,2M8,4V6H16V4H8M16,8H8V10H16V8M16,18H14V20H16V18Z';
      }
      return 'M3,6H21V8H3V6M3,11H21V13H3V11M3,16H21V18H3V16Z';
    }
    
    function handleClick(event) {
      event.stopPropagation();
      console.log('Device clicked:', deviceData.mac);
      
      // Dispatch both events - regular and namespaced
      dispatch('select', {
        element: deviceData,
        type: 'device'
      });
      
      dispatch('device:select', {
        element: deviceData,
        type: 'device'
      });
      
      // Also dispatch a custom element directly to the parent
      const customEvent = new CustomEvent('nodeClick', {
        detail: {
          element: deviceData,
          type: 'device'
        },
        bubbles: true,
        composed: true
      });
      
      event.target.dispatchEvent(customEvent);
    }
    
    const color = getDeviceColor(deviceData);
    const iconPath = getDeviceIcon(deviceData);
  </script>
  
  <!-- Added 'pointer-events-auto' to ensure all elements in the node capture clicks -->
  <div class="node device-node pointer-events-auto" on:click={handleClick} on:mousedown={event => event.stopPropagation()}>
    <Handle type="target" position="top" />
    <div class="node-content" style="background-color: {color};">
      <svg class="node-icon" viewBox="0 0 24 24">
        <path fill="white" d={iconPath} />
      </svg>
    </div>
    <div class="node-label">{label}</div>
  </div>
  
  <style>
    .node {
      border-radius: 50%;
      width: 35px;
      height: 35px;
      display: flex;
      justify-content: center;
      align-items: center;
      cursor: pointer;
      position: relative;
      z-index: 5;
      filter: drop-shadow(0 2px 3px rgba(0, 0, 0, 0.15));
      transition: filter 0.3s ease, transform 0.2s ease;
    }
    
    .node:hover {
      filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.25));
      transform: translateY(-2px);
    }
    
    .node-content {
      border-radius: 50%;
      width: 100%;
      height: 100%;
      display: flex;
      justify-content: center;
      align-items: center;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
      border: 2px solid white;
      position: relative;
      z-index: 5;
      transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
    }
    
    .node-content:hover {
      transform: scale(1.15);
      box-shadow: 0 6px 12px rgba(0, 0, 0, 0.25);
    }
    
    .node-icon {
      width: 18px;
      height: 18px;
      pointer-events: none; /* Ensure clicks go through to the parent */
      filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.2));
    }
    
    .node-label {
      position: absolute;
      top: 100%;
      left: 50%;
      transform: translateX(-50%);
      margin-top: 5px;
      white-space: nowrap;
      text-overflow: ellipsis;
      overflow: hidden;
      max-width: 130px;
      background-color: rgba(255, 255, 255, 0.95);
      padding: 3px 8px;
      border-radius: 6px;
      font-size: 11px;
      font-weight: 500;
      z-index: 10;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.15);
      pointer-events: none; /* Ensure clicks go through to the parent */
      opacity: 1; /* Always visible */
    }
    
    /* Dark theme adjustments for labels */
    :global([data-theme="dark"]) .node-label {
      background-color: rgba(30, 41, 59, 0.95);
      color: rgba(255, 255, 255, 0.9);
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }
  </style>