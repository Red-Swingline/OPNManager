<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import { mdiTuneVertical, mdiChevronDown, mdiChevronUp } from "@mdi/js";
  import { goto } from "$app/navigation";
  
  export let widget_id: string;
  export let widget_name: string = "System Tunables";
  export let widget_height: string = "auto";

  // Widget state
  let loading = true;
  let tunables = [];
  let isCollapsed = false;
  let maxVisibleItems = 5;

  // Load tunables
  async function loadTunables() {
    loading = true;
    try {
      const result = await invoke("search_tunables", {
        currentPage: 1,
        rowCount: maxVisibleItems,
        searchPhrase: ""
      });
      
      tunables = result.rows || [];
    } catch (error) {
      console.error("Failed to load tunables:", error);
      toasts.error(`Failed to load tunables: ${error}`);
    } finally {
      loading = false;
    }
  }

  // Apply tunable changes
  async function applyChanges() {
    try {
      const result = await invoke("apply_tunables");
      
      if (result.result === "applied") {
        toasts.success("All tunable changes applied successfully");
      } else {
        toasts.error("Failed to apply tunable changes");
      }
    } catch (error) {
      console.error("Failed to apply tunable changes:", error);
      toasts.error(`Failed to apply tunable changes: ${error}`);
    }
  }

  // Go to full tunables page
  function goToTunablesPage() {
    goto("/tunables");
  }

  // Toggle collapsed state
  function toggleCollapsed() {
    isCollapsed = !isCollapsed;
  }

  // Initial load
  onMount(() => {
    loadTunables();
  });
</script>

<div class="card bg-base-100 shadow-xl h-full" style={`height: ${widget_height};`}>
  <div class="card-body p-4">
    <div class="flex justify-between items-center mb-4">
      <h2 class="card-title text-lg flex items-center">
        <svg class="w-5 h-5 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiTuneVertical} />
        </svg>
        {widget_name}
      </h2>
      <div class="flex space-x-1">
        <button class="btn btn-sm btn-ghost" on:click={toggleCollapsed}>
          <svg class="w-5 h-5" viewBox="0 0 24 24">
            <path fill="currentColor" d={isCollapsed ? mdiChevronDown : mdiChevronUp} />
          </svg>
        </button>
      </div>
    </div>
    
    {#if !isCollapsed}
      {#if loading}
        <div class="flex justify-center items-center py-4">
          <div class="loading loading-spinner loading-md"></div>
        </div>
      {:else if tunables.length > 0}
        <div class="overflow-x-auto overflow-y-auto max-h-64">
          <table class="table table-xs table-zebra w-full">
            <thead>
              <tr>
                <th>Tunable</th>
                <th>Value</th>
              </tr>
            </thead>
            <tbody>
              {#each tunables as tunable (tunable.uuid)}
                <tr>
                  <td class="truncate max-w-[160px]" title={tunable.tunable}>
                    <span class="font-mono text-xs">{tunable.tunable}</span>
                  </td>
                  <td class="truncate max-w-[120px]" title={tunable.value}>
                    <span class="font-mono text-xs">{tunable.value}</span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
        
        <div class="card-actions justify-end mt-4">
          <button class="btn btn-sm btn-outline" on:click={goToTunablesPage}>
            View All
          </button>
        </div>
      {:else}
        <div class="text-center py-4">
          <p class="text-sm mb-2">No tunables found</p>
          <button class="btn btn-sm btn-outline mt-2" on:click={goToTunablesPage}>
            Manage Tunables
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>