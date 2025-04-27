<script lang="ts">
    import { onMount } from 'svelte';
    import { dashboardStore, type DashboardWidgetPref } from '$lib/stores/dashboardStore';
    import { toasts } from '$lib/stores/toastStore';
    import { 
      mdiDragVertical, 
      mdiEyeOutline, 
      mdiEyeOffOutline, 
      mdiCheck, 
      mdiClose, 
      mdiRefresh, 
      mdiCog
    } from '@mdi/js';
  
    // Widget display names for UI
    const widgetNames = {
      uptime: "System Status",
      memory: "Memory Usage",
      disk: "Disk Usage",
      services: "Services",
      traffic_graph: "Traffic Graph",
      gateways: "Gateways",
      interfaces: "Interface Traffic",
      wol: "Wake-on-LAN",
      tunables: "System Tunables"
    };
  
    // Enable drag and drop reordering
    let draggedItem: string | null = null;
    let widgets: DashboardWidgetPref[] = [];
    let originalWidgets: DashboardWidgetPref[] = [];
    
    $: widgets = [...$dashboardStore.widgets].sort((a, b) => a.position - b.position);
    
    onMount(() => {
      // Store original state for potential cancel action
      originalWidgets = [...$dashboardStore.widgets];
    });
  
    // Drag and drop handlers
    function handleDragStart(event: DragEvent, key: string) {
      if (!event.dataTransfer) return;
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', key);
      draggedItem = key;
    }
  
    function handleDragOver(event: DragEvent) {
      event.preventDefault();
      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'move';
      }
    }
  
    function handleDrop(event: DragEvent, targetKey: string) {
      event.preventDefault();
      if (!draggedItem || draggedItem === targetKey) return;
      
      // Reorder widgets
      const reorderedKeys = widgets
        .map(w => w.widget_key)
        .filter(key => key !== draggedItem);
      
      const targetIndex = reorderedKeys.indexOf(targetKey);
      reorderedKeys.splice(targetIndex, 0, draggedItem);
      
      dashboardStore.updatePositions(reorderedKeys);
      draggedItem = null;
    }
  
    function toggleWidget(key: string) {
      dashboardStore.toggleWidget(key);
    }
  
    async function saveChanges() {
      try {
        const success = await dashboardStore.savePreferences();
        if (success) {
          toasts.success('Dashboard preferences saved successfully');
          dashboardStore.exitEditMode();
        } else {
          toasts.error('Failed to save dashboard preferences');
        }
      } catch (error) {
        console.error('Failed to save dashboard preferences:', error);
        toasts.error('Failed to save dashboard preferences');
      }
    }
  
    function cancelChanges() {
      // Reset to original state
      dashboardStore.exitEditMode();
      
      // Use a setTimeout to ensure we're outside of the current update cycle
      setTimeout(() => {
        const orderedKeys = originalWidgets
          .sort((a, b) => a.position - b.position)
          .map(w => w.widget_key);
        
        dashboardStore.updatePositions(orderedKeys);
        
        // Reset visibility to original state
        originalWidgets.forEach(originalWidget => {
          const currentWidget = widgets.find(w => w.widget_key === originalWidget.widget_key);
          if (currentWidget && currentWidget.visible !== originalWidget.visible) {
            dashboardStore.toggleWidget(originalWidget.widget_key);
          }
        });
      }, 0);
    }
  
    function resetToDefaults() {
      dashboardStore.resetToDefaults();
      toasts.info('Dashboard reset to defaults');
    }
  </script>
  
  <div class="bg-base-100 p-4 rounded-lg shadow-lg mb-6">
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-lg font-bold flex items-center gap-2">
        <svg class="w-5 h-5" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiCog} />
        </svg>
        Dashboard Configuration
      </h3>
      
      <div class="flex gap-2">
        <button 
          class="btn btn-sm btn-outline" 
          on:click={resetToDefaults}
          title="Reset to defaults"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiRefresh} />
          </svg>
          Reset
        </button>
      </div>
    </div>
    
    <p class="text-sm opacity-70 mb-4">
      Drag and drop to reorder widgets or toggle their visibility. Changes won't take effect until you save.
    </p>
    
    <div class="overflow-x-auto">
      <table class="table w-full">
        <thead>
          <tr>
            <th class="w-10"></th>
            <th>Widget</th>
            <th class="w-20 text-center">Visible</th>
          </tr>
        </thead>
        <tbody>
          {#each widgets as widget (widget.widget_key)}
            <tr 
              draggable={true}
              on:dragstart={(e) => handleDragStart(e, widget.widget_key)}
              on:dragover={handleDragOver}
              on:drop={(e) => handleDrop(e, widget.widget_key)}
              class="hover:bg-base-200 cursor-move"
            >
              <td class="text-center">
                <svg class="w-5 h-5 mx-auto opacity-50" viewBox="0 0 24 24">
                  <path fill="currentColor" d={mdiDragVertical} />
                </svg>
              </td>
              <td class="py-3">
                {widgetNames[widget.widget_key] || widget.widget_key}
              </td>
              <td class="text-center">
                <button 
                  class="btn btn-sm btn-circle btn-ghost"
                  on:click={() => toggleWidget(widget.widget_key)}
                  title={widget.visible ? "Hide widget" : "Show widget"}
                >
                  <svg class="w-5 h-5" viewBox="0 0 24 24">
                    <path 
                      fill="currentColor" 
                      d={widget.visible ? mdiEyeOutline : mdiEyeOffOutline} 
                    />
                  </svg>
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    
    <div class="flex justify-end gap-2 mt-6">
      <button 
        class="btn btn-sm btn-outline"
        on:click={cancelChanges}
      >
        <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiClose} />
        </svg>
        Cancel
      </button>
      
      <button 
        class="btn btn-sm btn-primary"
        on:click={saveChanges}
      >
        <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiCheck} />
        </svg>
        Save Changes
      </button>
    </div>
  </div>