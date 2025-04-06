<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { Handle } from '@xyflow/svelte';
    import type { Interface } from './types';
  
    export let id;
    export let data;
    
    const dispatch = createEventDispatcher();
    const { label, interfaceData } = data;
    
    function getInterfaceColor(iface: Interface): string {
      switch (iface.status?.toLowerCase()) {
        case 'up': return '#4CAF50'; // Green
        case 'down': return '#F44336'; // Red
        case 'no carrier': return '#FFA000'; // Amber
        default: return '#9E9E9E'; // Gray
      }
    }
    
    function getInterfaceIcon(iface: Interface): string {
      if (iface.vlan_tag) {
        return 'M4,1C2.89,1 2,1.89 2,3V7C2,8.11 2.89,9 4,9H1V11H13V9H10C11.11,9 12,8.11 12,7V3C12,1.89 11.11,1 10,1H4M4,3H10V7H4V3M3,13V18L3,20H10V18H5V13H3M14,13C12.89,13 12,13.89 12,15V19C12,20.11 12.89,21 14,21H20C21.11,21 22,20.11 22,19V15C22,13.89 21.11,13 20,13H14M14,15H20V19H14V15Z';
      }
      if (iface.is_physical) {
        return 'M7,15H9V18H13V15H15V18H16A1,1 0 0,0 17,17V7A1,1 0 0,0 16,6H8A1,1 0 0,0 7,7V17A1,1 0 0,0 8,18H9V15M10,7H14V9H10V7M10,10H14V12H10V10M8,13H16V14H8V13Z';
      }
      return 'M10,2C8.89,2 8,2.89 8,4V7C8,8.11 8.89,9 10,9H11V11H2V13H6V15H5C3.89,15 3,15.89 3,17V20C3,21.11 3.89,22 5,22H9C10.11,22 11,21.11 11,20V17C11,15.89 10.11,15 9,15H8V13H16V15H15C13.89,15 13,15.89 13,17V20C13,21.11 13.89,22 15,22H19C20.11,22 21,21.11 21,20V17C21,15.89 20.11,15 19,15H18V13H22V11H13V9H14C15.11,9 16,8.11 16,7V4C16,2.89 15.11,2 14,2H10M10,4H14V7H10V4M5,17H9V20H5V17M15,17H19V20H15V17Z';
    }
    
    function handleClick(event) {
      event.stopPropagation();
      console.log('Interface clicked:', interfaceData.device);
      
      // Dispatch events using multiple approaches for robustness
      dispatch('select', {
        element: interfaceData,
        type: 'interface'
      });
      
      dispatch('interface:select', {
        element: interfaceData,
        type: 'interface'
      });
      
      // Also dispatch a custom event that bubbles up through the DOM
      const customEvent = new CustomEvent('nodeClick', {
        detail: {
          element: interfaceData,
          type: 'interface'
        },
        bubbles: true,
        composed: true
      });
      
      event.target.dispatchEvent(customEvent);
    }
  
    const color = getInterfaceColor(interfaceData);
    const iconPath = getInterfaceIcon(interfaceData);
  </script>
  
  <!-- Added pointer-events-auto to ensure click capture -->
  <div class="node interface-node pointer-events-auto" on:click={handleClick} on:mousedown={event => event.stopPropagation()}>
    <Handle type="source" position="bottom" />
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
      width: 45px;
      height: 45px;
      display: flex;
      justify-content: center;
      align-items: center;
      cursor: pointer;
      position: relative;
      z-index: 5;
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
    }
    
    .node-content:hover {
      transform: scale(1.1);
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
      transition: all 0.2s ease;
    }
    
    .node-icon {
      width: 20px;
      height: 20px;
      pointer-events: none; /* Ensure clicks go through to the parent */
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
      max-width: 150px;
      background-color: rgba(255, 255, 255, 0.9);
      padding: 3px 6px;
      border-radius: 4px;
      font-size: 12px;
      z-index: 20;
      font-weight: 500;
      box-shadow: 0 1px 3px rgba(0,0,0,0.15);
      pointer-events: none; /* Ensure clicks go through to the parent */
    }
  </style>