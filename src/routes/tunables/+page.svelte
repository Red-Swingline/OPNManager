<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import { fabStore } from "$lib/stores/fabStore";
  import AppLayout from "../AppLayout.svelte";
  
  // Page state
  let loading = true;
  let error: string | null = null;
  let tunables = [];
  let total = 0;
  let currentPage = 1;
  let rowsPerPage = 25;
  let searchPhrase = "";
  let editModal = { isOpen: false, tunable: null };
  let addModal = { isOpen: false };
  let deleteModal = { isOpen: false, uuid: null, name: '' };
  let newTunable = { tunable: '', value: '', descr: '' };
  let expandedRow: string | null = null; // Track which row is expanded on mobile
  let selectedTunables = {}; // Track selected tunables by uuid
  let selectAll = false; // Track if all are selected
  
  // Initialize the FAB
  fabStore.set({ 
    show: false, 
    actions: [] 
  });
  
  // Close any modals on page unload
  onDestroy(() => {
    editModal.isOpen = false;
  });
  
  // Load tunables
  async function loadTunables() {
    loading = true;
    error = null;
    try {
      const result = await invoke("search_tunables", {
        currentPage,
        rowCount: rowsPerPage,
        searchPhrase
      });
      
      tunables = result.rows || [];
      total = result.total || 0;
      
      // Reset selection state
      selectAll = false;
      selectedTunables = {}; 
      // Initialize selection state for all tunables
      tunables.forEach(tunable => {
        selectedTunables[tunable.uuid] = false;
      });
      
      // Set FAB actions
      fabStore.set({
        show: false,
        actions: []
      });
    } catch (err) {
      console.error("Failed to load tunables:", err);
      error = err.toString();
      tunables = [];
      total = 0;
      
      // Display toast only if it's not a 404 (version compatibility) or 403 (permissions)
      // since we'll show these directly in the UI instead
      if (!error.includes('404') && !error.includes('403')) {
        toasts.error(`Failed to load tunables: ${err}`);
      }
    } finally {
      loading = false;
    }
  }
  
  // Store the currently edited UUID separately to avoid the UUID getting lost
  let currentEditUuid = "";
  
  // Reactive statement to handle selection changes
  $: selectedCount = getSelectedCount();
  
  // Open edit modal for a tunable
  async function openEditModal(uuid: string) {
    console.log("Opening edit modal for UUID:", uuid);
    
    // Store the UUID in a variable outside the modal context
    currentEditUuid = uuid;
    
    try {
      const tunable = await invoke("get_tunable", { uuid });
      console.log("Received tunable from API:", tunable);
      
      // Store the tunable with its UUID explicitly included
      const tunableWithUuid = {
        ...tunable.sysctl,
        uuid: uuid  // Store the UUID with the tunable object
      };
      
      console.log("Storing tunable with UUID:", tunableWithUuid);
      
      editModal = { 
        isOpen: true, 
        tunable: tunableWithUuid
      };
    } catch (error) {
      console.error("Failed to get tunable details:", error);
      toasts.error(`Failed to get tunable details: ${error}`);
    }
  }
  
  // Close edit modal
  function closeEditModal() {
    editModal.isOpen = false;
    editModal.tunable = null;
    currentEditUuid = ""; // Reset the current edit UUID
  }
  
  // Open add modal
  function openAddModal() {
    addModal.isOpen = true;
    // Reset the new tunable form
    newTunable = { tunable: '', value: '', descr: '' };
  }
  
  // Close add modal
  function closeAddModal() {
    addModal.isOpen = false;
  }
  
  // Open delete confirmation modal
  function openDeleteModal(uuid: string, tunableName: string) {
    deleteModal = {
      isOpen: true,
      uuid,
      name: tunableName
    };
  }
  
  // Close delete confirmation modal
  function closeDeleteModal() {
    deleteModal = {
      isOpen: false,
      uuid: null,
      name: ''
    };
  }
  
  // Handle deleting a tunable
  async function handleDeleteTunable() {
    if (!deleteModal.uuid) {
      toasts.error("No tunable selected for deletion");
      return;
    }
    
    try {
      console.log("Deleting tunable with UUID:", deleteModal.uuid);
      const result = await invoke("delete_tunable", {
        uuid: deleteModal.uuid
      });
      
      console.log("Delete tunable result:", result);
      
      if (result.result === "deleted") {
        // Apply the changes immediately
        const applyResult = await invoke("apply_tunables");
        console.log("Apply result after delete:", applyResult);
        
        if (applyResult.status === "ok") {
          toasts.success("Tunable deleted and changes applied successfully");
        } else {
          toasts.success("Tunable deleted but could not apply changes");
        }
        
        closeDeleteModal();
        
        // Reload the tunables list
        loadTunables();
      } else {
        toasts.error("Failed to delete tunable");
      }
    } catch (error) {
      console.error("Failed to delete tunable:", error);
      toasts.error(`Failed to delete tunable: ${error}`);
    }
  }
  
  // Handle saving a tunable
  async function handleSaveTunable(event) {
    const { uuid, tunable, value, description } = event.detail;
    
    console.log("handleSaveTunable received:", { uuid, tunable, value, description });
    
    // Validate we have a UUID before proceeding
    if (!uuid) {
      console.error("Missing UUID in save tunable request");
      toasts.error("Cannot save tunable: missing UUID");
      return;
    }
    
    try {
      // Create payload object and log it before sending
      const payload = {
        uuid,
        tunable,
        value,
        description
      };
      
      console.log("Invoking save_and_apply_tunable with payload:", payload);
      
      const result = await invoke("save_and_apply_tunable", payload);
      
      console.log("Save result:", result);
      
      // Check for successful save - needs to handle both result formats
      const saveSuccess = result.set?.result === "saved";
      
      // Check for successful apply - needs to handle both result formats
      const applySuccess = result.apply?.result === "applied" || result.apply?.status === "ok";
      
      if (saveSuccess && applySuccess) {
        toasts.success("Tunable saved and applied successfully");
        closeEditModal();
        loadTunables();
      } else if (saveSuccess) {
        toasts.success("Tunable saved successfully, but could not be applied");
        closeEditModal();
        loadTunables();
      } else {
        toasts.error("Failed to save tunable");
      }
    } catch (error) {
      console.error("Failed to save tunable:", error);
      toasts.error(`Failed to save tunable: ${error}`);
    }
  }
  
  // Apply all tunable changes
  async function handleApplyChanges() {
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
  
  // Get count of selected tunables
  function getSelectedCount() {
    return Object.values(selectedTunables).filter(Boolean).length;
  }
  
  // Handle batch delete for selected tunables
  async function handleBatchDelete() {
    const selectedCount = getSelectedCount();
    if (selectedCount === 0) {
      toasts.error("No tunables selected for deletion");
      return;
    }
    
    if (!confirm(`Are you sure you want to delete ${selectedCount} selected tunable${selectedCount > 1 ? 's' : ''}?`)) {
      return;
    }
    
    try {
      const selectedUuids = Object.entries(selectedTunables)
        .filter(([_, isSelected]) => isSelected)
        .map(([uuid]) => uuid);
      
      // Delete one by one since there's no batch delete endpoint
      let successCount = 0;
      for (const uuid of selectedUuids) {
        try {
          const result = await invoke("delete_tunable", { uuid });
          if (result.result === "deleted") {
            successCount++;
          }
        } catch (error) {
          console.error("Failed to delete tunable:", error);
        }
      }
      
      // Apply changes after all deletions
      await invoke("apply_tunables");
      
      if (successCount === selectedUuids.length) {
        toasts.success(`Successfully deleted ${successCount} tunables`);
      } else {
        toasts.warning(`Deleted ${successCount} of ${selectedUuids.length} tunables`);
      }
      
      // Reload the list
      await loadTunables();
    } catch (error) {
      console.error("Batch delete operation failed:", error);
      toasts.error(`Batch delete operation failed: ${error}`);
    }
  }
  
  // Handle adding a new tunable
  async function handleAddTunable() {
    if (!newTunable.tunable || !newTunable.value) {
      toasts.error("Tunable name and value are required");
      return;
    }
    
    try {
      console.log("Adding new tunable:", newTunable);
      const result = await invoke("add_tunable", {
        tunable: newTunable.tunable,
        value: newTunable.value,
        description: newTunable.descr || ""
      });
      
      console.log("Add tunable result:", result);
      
      if (result.result === "saved") {
        // Store the UUID from the response for future reference if needed
        const uuid = result.uuid;
        console.log("Added tunable with UUID:", uuid);
        
        // Apply the changes immediately
        const applyResult = await invoke("apply_tunables");
        console.log("Apply result:", applyResult);
        
        if (applyResult.status === "ok") {
          toasts.success("Tunable added and applied successfully");
        } else {
          toasts.success("Tunable added but could not be applied");
        }
        
        closeAddModal();
        
        // Reload the tunables list
        loadTunables();
      } else {
        toasts.error("Failed to add tunable");
      }
    } catch (error) {
      console.error("Failed to add tunable:", error);
      toasts.error(`Failed to add tunable: ${error}`);
    }
  }
  
  // Handle pagination
  function handlePageChange(newPage: number) {
    currentPage = newPage;
    loadTunables();
  }
  
  // Handle search
  function handleSearch() {
    currentPage = 1;
    loadTunables();
  }
  
  // Initial load
  onMount(() => {
    loadTunables();
  });
</script>

<AppLayout>
  <div class="p-4">
    <div class="mb-6">
      <h1 class="text-2xl font-bold mb-2">System Tunables</h1>
      <p class="text-sm opacity-75">
        System tunables allow you to modify kernel and system parameters.
        Use with caution as incorrect values may affect system stability.
      </p>
    </div>
    
    <!-- Actions and Search Bar -->
    <div class="mb-6 space-y-3">
      <!-- First row: Search with icon inside input (OPNsense style) -->
      <div class="relative">
        <input
          type="text"
          placeholder="Search tunables..."
          class="input input-bordered w-full pl-10"
          bind:value={searchPhrase}
          on:keyup={(e) => e.key === 'Enter' && handleSearch()}
        />
        <button 
          class="absolute left-0 top-0 bottom-0 px-3 flex items-center justify-center"
          on:click={handleSearch}
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </button>
      </div>
      
      <!-- Second row: Per page dropdown and Add button -->
      <div class="flex justify-between items-center">
        <div class="form-control">
          <select class="select select-bordered" bind:value={rowsPerPage} on:change={() => {currentPage = 1; loadTunables()}}>
            <option value={10}>10 per page</option>
            <option value={25}>25 per page</option>
            <option value={50}>50 per page</option>
            <option value={100}>100 per page</option>
          </select>
        </div>
        
        <button class="btn btn-primary" on:click={openAddModal}>
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 5v14M5 12h14"></path>
          </svg>
          Add
        </button>
      </div>
    </div>
    
    <!-- Loading state -->
    {#if loading}
      <div class="flex justify-center items-center py-12">
        <div class="loading loading-spinner loading-lg"></div>
      </div>
    
    <!-- Error states -->
    {:else if error}
      <div class="alert {error.includes('404') ? 'alert-warning' : error.includes('403') ? 'alert-error' : 'alert-error'} shadow-lg mb-4">
        <div>
          <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
          {#if error.includes('404')}
            <div>
              <h3 class="font-bold">Feature Not Available</h3>
              <div class="text-sm">Tunables management requires OPNsense 25.x or newer. Your firewall appears to be running an older version.</div>
            </div>
          {:else if error.includes('403')}
            <div>
              <h3 class="font-bold">Permission Denied</h3>
              <div class="text-sm">Your API credentials don't have sufficient permissions to manage tunables. Please check your API key permissions in OPNsense.</div>
            </div>
          {:else}
            <div>
              <h3 class="font-bold">Error Loading Tunables</h3>
              <div class="text-sm">{error}</div>
            </div>
          {/if}
        </div>
      </div>
    
    <!-- Tunables table for desktop view -->
    {:else if tunables.length > 0}
      <!-- Batch actions toolbar (shown when items are selected) -->
      {#if getSelectedCount() > 0}
        <div class="bg-base-200 p-2 rounded-md mb-3 flex items-center justify-between">
          <span class="font-semibold">{getSelectedCount()} item{getSelectedCount() > 1 ? 's' : ''} selected</span>
          <div class="flex gap-2">
            <button 
              class="btn btn-sm btn-error" 
              on:click={handleBatchDelete}
            >
              Delete Selected
            </button>
          </div>
        </div>
      {/if}
      
      <!-- Desktop table view (hidden on mobile) -->
      <div class="hidden md:block overflow-x-auto bg-base-100 rounded-lg shadow">
        <table class="table table-zebra w-full">
          <thead>
            <tr>
              <th class="w-8">
                <input 
                  type="checkbox" 
                  class="checkbox checkbox-sm" 
                  bind:checked={selectAll}
                  on:change={() => {
                    // When selectAll changes, update all individual checkboxes
                    tunables.forEach(tunable => {
                      selectedTunables[tunable.uuid] = selectAll;
                    });
                  }}
                />
              </th>
              <th>Tunable Name</th>
              <th>Type</th>
              <th>Value</th>
              <th>Default</th>
              <th class="hidden lg:table-cell">Description</th>
              <th class="w-24">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each tunables as tunable (tunable.uuid)}
              <tr>
                <td>
                  <input 
                    type="checkbox" 
                    class="checkbox checkbox-sm" 
                    bind:checked={selectedTunables[tunable.uuid]}
                    on:change={() => {
                      // Check if all are selected
                      selectAll = tunables.every(t => selectedTunables[t.uuid]);
                    }} 
                  />
                </td>
                <td class="font-mono text-xs md:text-sm max-w-[200px] truncate">
                  <span title={tunable.tunable}>{tunable.tunable}</span>
                </td>
                <td>runtime</td>
                <td class="font-mono text-xs md:text-sm max-w-[120px] truncate">
                  <span title={tunable.value}>{tunable.value}</span>
                </td>
                <td class="font-mono text-xs md:text-sm max-w-[120px] truncate">
                  <span title={tunable.default_value || tunable.value}>{tunable.default_value || tunable.value}</span>
                </td>
                <td class="hidden lg:table-cell text-xs md:text-sm max-w-[200px] truncate">
                  <span title={tunable.descr || '-'}>{tunable.descr || '-'}</span>
                </td>
                <td>
                  <div class="flex gap-1">
                    <button 
                      class="btn btn-sm btn-ghost btn-square"
                      on:click={() => openEditModal(tunable.uuid)}
                      title="Edit"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                      </svg>
                    </button>
                    <button
                      class="btn btn-sm btn-ghost btn-square"
                      title="Delete"
                      on:click={() => openDeleteModal(tunable.uuid, tunable.tunable)}
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M3 6h18"></path>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      
      <!-- Mobile card view (shown on small screens) -->
      <div class="md:hidden space-y-4">
        {#each tunables as tunable (tunable.uuid)}
          <div class="card bg-base-100 shadow-md hover:shadow-lg transition-shadow">
            <div class="card-body p-4">
              <div class="flex flex-col">
                <div class="font-mono text-xs font-medium truncate mb-2">{tunable.tunable}</div>
                
                <div class="divider my-1"></div>
                
                <div class="flex flex-col gap-2 text-sm">
                  <div>
                    <div class="opacity-70 text-xs">Value</div>
                    <div class="font-mono break-all">{tunable.value}</div>
                  </div>
                  <div>
                    <div class="opacity-70 text-xs">Default</div>
                    <div class="font-mono break-all">{tunable.default_value || tunable.value}</div>
                  </div>
                  <div>
                    <div class="opacity-70 text-xs">Description</div>
                    <div class="break-all text-sm">{tunable.descr || '-'}</div>
                  </div>
                </div>
                
                <div class="card-actions justify-end mt-3">
                  <button 
                    class="btn btn-sm btn-primary" 
                    on:click={() => openEditModal(tunable.uuid)}
                  >
                    Edit
                  </button>
                  <button 
                    class="btn btn-sm btn-error" 
                    on:click={() => openDeleteModal(tunable.uuid, tunable.tunable)}
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
      
      <!-- Pagination and Record Count -->
      {#if total > 0}
        <div class="flex flex-col sm:flex-row justify-between items-center mt-6 gap-3">
          <div class="text-sm text-base-content/70">
            Showing {(currentPage - 1) * rowsPerPage + 1} to {Math.min(currentPage * rowsPerPage, total)} of {total} entries
          </div>
          
          {#if total > rowsPerPage}
            <div class="btn-group">
              <button 
                class="btn btn-sm" 
                disabled={currentPage === 1}
                on:click={() => handlePageChange(1)}
              >
                «
              </button>
              <button 
                class="btn btn-sm" 
                disabled={currentPage === 1}
                on:click={() => handlePageChange(currentPage - 1)}
              >
                ‹
              </button>
              
              {#each Array(Math.ceil(total / rowsPerPage)) as _, i}
                {#if i + 1 === currentPage || i + 1 === 1 || i + 1 === Math.ceil(total / rowsPerPage) || (i + 1 >= currentPage - 1 && i + 1 <= currentPage + 1)}
                  <button 
                    class="btn btn-sm" 
                    class:btn-active={i + 1 === currentPage}
                    on:click={() => handlePageChange(i + 1)}
                  >
                    {i + 1}
                  </button>
                {:else if i + 1 === currentPage - 2 || i + 1 === currentPage + 2}
                  <button class="btn btn-sm btn-disabled">...</button>
                {/if}
              {/each}
              
              <button 
                class="btn btn-sm" 
                disabled={currentPage === Math.ceil(total / rowsPerPage)}
                on:click={() => handlePageChange(currentPage + 1)}
              >
                ›
              </button>
              <button 
                class="btn btn-sm" 
                disabled={currentPage === Math.ceil(total / rowsPerPage)}
                on:click={() => handlePageChange(Math.ceil(total / rowsPerPage))}
              >
                »
              </button>
            </div>
          {/if}
        </div>
      {/if}
    
    <!-- Empty state -->
    {:else}
      <div class="text-center py-12">
        <p class="text-lg mb-2">No tunables found</p>
        {#if searchPhrase}
          <p class="text-sm opacity-75">Try adjusting your search criteria</p>
        {/if}
      </div>
    {/if}
  </div>
</AppLayout>

<!-- Edit Tunable Modal -->
{#if editModal.isOpen && editModal.tunable}
  <div class="modal modal-open">
    <div class="modal-box max-w-2xl">
      <h3 class="font-bold text-lg mb-4">Edit Tunable {editModal.tunable.uuid ? `(${editModal.tunable.uuid})` : ''}</h3>
      
      <form on:submit|preventDefault={() => {
        const tunable = editModal.tunable;
        
        // Use currentEditUuid as a fallback if the uuid is missing from the tunable object
        const uuid = tunable.uuid || currentEditUuid;
        
        console.log("Submitting tunable with UUID:", uuid);
        console.log("Complete tunable object:", tunable);
        
        const detailObj = {
            uuid: uuid,
            tunable: tunable.tunable,
            value: tunable.value, 
            description: tunable.descr
        };
        
        console.log("Sending detail object:", detailObj);
        
        handleSaveTunable({
          detail: detailObj
        });
      }}>
        <div class="form-control mb-4">
          <label class="label">
            <span class="label-text">Tunable Name</span>
          </label>
          <input 
            type="text" 
            class="input input-bordered w-full" 
            bind:value={editModal.tunable.tunable}
            disabled
          />
        </div>
        
        <div class="form-control mb-4">
          <label class="label">
            <span class="label-text">Value</span>
          </label>
          <input 
            type="text" 
            class="input input-bordered w-full" 
            bind:value={editModal.tunable.value}
            required
          />
        </div>
        
        <div class="form-control mb-6">
          <label class="label">
            <span class="label-text">Description</span>
          </label>
          <textarea 
            class="textarea textarea-bordered w-full"
            rows="3"
            bind:value={editModal.tunable.descr}
          ></textarea>
        </div>
        
        <div class="modal-action">
          <button type="submit" class="btn btn-primary">Save & Apply</button>
          <button type="button" class="btn" on:click={closeEditModal}>Cancel</button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" on:click={closeEditModal}></div>
  </div>
{/if}

<!-- Add Tunable Modal -->
{#if addModal.isOpen}
  <div class="modal modal-open">
    <div class="modal-box max-w-2xl">
      <h3 class="font-bold text-lg mb-4">Add New Tunable</h3>
      
      <form on:submit|preventDefault={handleAddTunable}>
        <div class="form-control mb-4">
          <label class="label">
            <span class="label-text">Tunable Name</span>
          </label>
          <input 
            type="text" 
            class="input input-bordered w-full" 
            bind:value={newTunable.tunable}
            placeholder="e.g. net.inet.tcp.keepidle"
            required
          />
        </div>
        
        <div class="form-control mb-4">
          <label class="label">
            <span class="label-text">Value</span>
          </label>
          <input 
            type="text" 
            class="input input-bordered w-full" 
            bind:value={newTunable.value}
            placeholder="e.g. 7200"
            required
          />
        </div>
        
        <div class="form-control mb-6">
          <label class="label">
            <span class="label-text">Description</span>
          </label>
          <textarea 
            class="textarea textarea-bordered w-full"
            rows="3"
            bind:value={newTunable.descr}
            placeholder="(Optional) Description of this tunable"
          ></textarea>
        </div>
        
        <div class="modal-action">
          <button type="submit" class="btn btn-primary">Add & Apply</button>
          <button type="button" class="btn" on:click={closeAddModal}>Cancel</button>
        </div>
      </form>
    </div>
    <div class="modal-backdrop" on:click={closeAddModal}></div>
  </div>
{/if}

<!-- Delete Confirmation Modal -->
{#if deleteModal.isOpen}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg mb-4">Confirm Deletion</h3>
      
      <p class="mb-6">
        Are you sure you want to delete the tunable <span class="font-mono font-semibold">{deleteModal.name}</span>?
        This action cannot be undone.
      </p>
      
      <div class="modal-action">
        <button 
          class="btn btn-error" 
          on:click={handleDeleteTunable}
        >
          Delete
        </button>
        <button type="button" class="btn" on:click={closeDeleteModal}>Cancel</button>
      </div>
    </div>
    <div class="modal-backdrop" on:click={closeDeleteModal}></div>
  </div>
{/if}