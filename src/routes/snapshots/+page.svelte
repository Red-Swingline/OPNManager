<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import AppLayout from "../AppLayout.svelte";
  import { mdiRefresh, mdiPlus, mdiPencil, mdiContentCopy, mdiDelete, mdiAlertCircleOutline, mdiCheck, mdiShieldCheckOutline } from "@mdi/js";

  // State management
  let isLoading = true;
  let isZfsSupported = false;
  let snapshots = [];
  let currentPage = 1;
  let rowsPerPage = 10;
  let totalSnapshots = 0;

  // Modal state
  let showAddModal = false;
  let showEditModal = false;
  let showDeleteModal = false;
  let showActivateModal = false;
  let isProcessing = false;
  let newSnapshotName = "";
  let currentSnapshot = null;
  let rebootAfterActivate = false;
  
  // Function to convert spaces to underscores in snapshot names
  function handleNameInput(event) {
    // Replace spaces with underscores
    newSnapshotName = event.target.value.replace(/\s+/g, '_');
  }

  async function checkZfsSupport() {
    try {
      isLoading = true;
      isZfsSupported = await invoke("is_snapshots_supported");
      
      if (!isZfsSupported) {
        toasts.warning("ZFS snapshots are not supported on this system.");
      } else {
        loadSnapshots();
      }
    } catch (error) {
      console.error("Failed to check ZFS support:", error);
      toasts.error(`Failed to check ZFS support: ${error}`);
      isZfsSupported = false;
    } finally {
      isLoading = false;
    }
  }

  async function loadSnapshots() {
    try {
      isLoading = true;
      const result = await invoke("get_snapshots", {
        currentPage,
        rowsPerPage
      });
      
      snapshots = result.rows || [];
      totalSnapshots = result.total || 0;
      console.log("Loaded snapshots:", snapshots);
    } catch (error) {
      console.error("Failed to load snapshots:", error);
      toasts.error(`Failed to load snapshots: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  async function openAddModal() {
    try {
      isProcessing = true;
      showAddModal = true;
      const result = await invoke("get_new_snapshot");
      newSnapshotName = result.name;
      currentSnapshot = null;
    } catch (error) {
      console.error("Failed to get new snapshot info:", error);
      toasts.error(`Failed to prepare new snapshot: ${error}`);
      showAddModal = false;
    } finally {
      isProcessing = false;
    }
  }

  async function openEditModal(snapshot) {
    try {
      isProcessing = true;
      const result = await invoke("get_snapshot", {
        uuid: snapshot.uuid
      });
      currentSnapshot = result;
      newSnapshotName = result.name;
      showEditModal = true;
    } catch (error) {
      console.error("Failed to get snapshot info:", error);
      toasts.error(`Failed to load snapshot details: ${error}`);
    } finally {
      isProcessing = false;
    }
  }

  async function openCloneModal(snapshot) {
    try {
      isProcessing = true;
      const result = await invoke("get_snapshot", {
        uuid: snapshot.uuid,
        fetchMode: "copy"
      });
      currentSnapshot = result;
      newSnapshotName = `${result.name}_clone`;
      showAddModal = true;
    } catch (error) {
      console.error("Failed to get snapshot for cloning:", error);
      toasts.error(`Failed to prepare snapshot clone: ${error}`);
    } finally {
      isProcessing = false;
    }
  }

  async function confirmDeleteSnapshot(snapshot) {
    currentSnapshot = snapshot;
    showDeleteModal = true;
  }
  
  async function confirmActivateSnapshot(snapshot) {
    currentSnapshot = snapshot;
    showActivateModal = true;
  }

  async function activateSnapshot() {
    try {
      isProcessing = true;
      if (!currentSnapshot || !currentSnapshot.uuid) {
        toasts.error("No snapshot selected for activation");
        return;
      }
      
      // First activate the snapshot
      const result = await invoke("activate_snapshot", {
        uuid: currentSnapshot.uuid
      });
      
      console.log("Snapshot activation result:", result);
      
      if (result && result.status === "ok") {
        let message = "Snapshot marked for activation on next reboot (R)";
        
        // If user chose to reboot, initiate reboot
        if (rebootAfterActivate) {
          try {
            const rebootResult = await invoke("reboot_firewall");
            if (rebootResult && rebootResult.status === "ok") {
              message = "Snapshot activated and firewall is rebooting";
            } else {
              message = "Snapshot activated, but reboot request failed";
            }
          } catch (rebootError) {
            console.error("Reboot error:", rebootError);
            message = "Snapshot activated, but reboot request failed";
          }
        }
        
        toasts.success(message);
        showActivateModal = false;
        loadSnapshots();
      } else {
        toasts.error(`Failed to activate snapshot: ${result?.result || "Unknown error"}`);
      }
    } catch (error) {
      console.error("Failed to activate snapshot:", error);
      toasts.error(`Failed to activate snapshot: ${error}`);
    } finally {
      isProcessing = false;
      rebootAfterActivate = false; // Reset the checkbox
    }
  }

  async function addSnapshot() {
    try {
      isProcessing = true;
      if (!newSnapshotName || newSnapshotName.trim() === "") {
        toasts.error("Snapshot name cannot be empty");
        return;
      }
      
      // Replace any spaces with underscores (just in case)
      if (newSnapshotName.includes(" ")) {
        newSnapshotName = newSnapshotName.replace(/\s+/g, '_');
      }

      const uuid = currentSnapshot?.uuid || null;
      
      const result = await invoke("add_snapshot", {
        name: newSnapshotName,
        uuid: uuid
      });
      
      console.log("Snapshot creation result:", result);
      
      if (result && result.status === "ok") {
        toasts.success(`Snapshot ${uuid ? "cloned" : "created"} successfully`);
        showAddModal = false;
        loadSnapshots();
      } else {
        toasts.error(`Failed to ${uuid ? "clone" : "create"} snapshot: ${result?.result || "Unknown error"}`);
      }
    } catch (error) {
      console.error("Failed to add snapshot:", error);
      toasts.error(`Failed to ${currentSnapshot?.uuid ? "clone" : "create"} snapshot: ${error}`);
    } finally {
      isProcessing = false;
    }
  }
  
  async function updateSnapshot() {
    try {
      isProcessing = true;
      if (!newSnapshotName || newSnapshotName.trim() === "") {
        toasts.error("Snapshot name cannot be empty");
        return;
      }
      
      // Replace any spaces with underscores (just in case)
      if (newSnapshotName.includes(" ")) {
        newSnapshotName = newSnapshotName.replace(/\s+/g, '_');
      }

      if (!currentSnapshot || !currentSnapshot.uuid) {
        toasts.error("No snapshot selected for updating");
        return;
      }
      
      const result = await invoke("update_snapshot", {
        uuid: currentSnapshot.uuid,
        name: newSnapshotName
      });
      
      console.log("Snapshot update result:", result);
      
      if (result && result.status === "ok") {
        toasts.success("Snapshot renamed successfully");
        showEditModal = false;
        loadSnapshots();
      } else {
        toasts.error(`Failed to rename snapshot: ${result?.result || "Unknown error"}`);
      }
    } catch (error) {
      console.error("Failed to update snapshot:", error);
      toasts.error(`Failed to rename snapshot: ${error}`);
    } finally {
      isProcessing = false;
    }
  }

  async function deleteSnapshot() {
    try {
      isProcessing = true;
      if (!currentSnapshot || !currentSnapshot.uuid) {
        toasts.error("No snapshot selected for deletion");
        return;
      }
      
      const result = await invoke("delete_snapshot", {
        uuid: currentSnapshot.uuid
      });
      
      console.log("Snapshot deletion result:", result);
      
      if (result && result.status === "ok") {
        toasts.success("Snapshot deleted successfully");
        showDeleteModal = false;
        loadSnapshots();
      } else {
        toasts.error(`Failed to delete snapshot: ${result?.result || "Unknown error"}`);
      }
    } catch (error) {
      console.error("Failed to delete snapshot:", error);
      toasts.error(`Failed to delete snapshot: ${error}`);
    } finally {
      isProcessing = false;
    }
  }

  function handlePageChange(newPage) {
    if (newPage >= 1 && newPage <= Math.ceil(totalSnapshots / rowsPerPage)) {
      currentPage = newPage;
      loadSnapshots();
    }
  }

  function formatDate(timestamp) {
    if (!timestamp) return "Unknown";
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  }

  onMount(() => {
    checkZfsSupport();
  });
</script>

<AppLayout>
  <div class="p-4 max-w-7xl mx-auto">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-4 gap-4">
      <h2 class="text-2xl font-bold">ZFS Snapshots</h2>
      <div class="flex w-full sm:w-auto gap-2">
        <button 
          class="btn btn-outline flex-1 sm:flex-none" 
          on:click={loadSnapshots}
          disabled={isLoading || !isZfsSupported}
        >
          {#if isLoading}
            <span class="loading loading-spinner loading-sm"></span>
          {:else}
            <svg class="w-5 h-5 sm:mr-1" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiRefresh} />
            </svg>
            <span class="hidden sm:inline">Refresh</span>
          {/if}
        </button>
        
        <button 
          class="btn btn-primary flex-1 sm:flex-none" 
          on:click={openAddModal}
          disabled={isLoading || !isZfsSupported}
        >
          <svg class="w-5 h-5 sm:mr-1" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiPlus} />
          </svg>
          <span class="hidden sm:inline">New Snapshot</span>
          <span class="sm:hidden">New</span>
        </button>
      </div>
    </div>

    {#if isLoading}
      <div class="flex justify-center items-center h-64">
        <span class="loading loading-spinner loading-lg"></span>
      </div>
    {:else if !isZfsSupported}
      <div class="alert alert-warning">
        <svg class="w-6 h-6 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiAlertCircleOutline} />
        </svg>
        <div>
          <h3 class="font-bold">ZFS Snapshots Not Supported</h3>
          <div class="text-sm">This feature requires OPNsense to be installed on a ZFS filesystem.</div>
        </div>
      </div>
    {:else if snapshots.length === 0}
      <div class="bg-base-200 rounded-lg p-8 text-center">
        <h3 class="text-xl font-semibold mb-2">No Snapshots Found</h3>
        <p class="text-base-content/70 mb-6">Create your first ZFS snapshot to protect your OPNsense configuration.</p>
        <button class="btn btn-primary" on:click={openAddModal}>
          <svg class="w-5 h-5 mr-1" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiPlus} />
          </svg>
          Create Snapshot
        </button>
      </div>
    {:else}
      <!-- Desktop/Tablet View (md and above) -->
      <div class="hidden md:block overflow-x-auto">
        <table class="table w-full">
          <thead>
            <tr>
              <th>Name</th>
              <th>Size</th>
              <th>Created</th>
              <th>Mount Point</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each snapshots as snapshot (snapshot.uuid)}
              <tr class="hover">
                <td class="font-medium">{snapshot.name}</td>
                <td>{snapshot.size}</td>
                <td>{snapshot.created_str || formatDate(snapshot.created)}</td>
                <td>{snapshot.mountpoint || '-'}</td>
                <td>
                  {#if snapshot.active === 'NR'}
                    <div class="badge badge-success tooltip" data-tip="Active now and on reboot">NR</div>
                  {:else if snapshot.active === 'N'}
                    <div class="badge badge-info tooltip" data-tip="Active now">N</div>
                  {:else if snapshot.active === 'R'}
                    <div class="badge badge-warning tooltip" data-tip="Active after reboot">R</div>
                  {:else}
                    <div class="badge badge-ghost tooltip" data-tip="Not active">-</div>
                  {/if}
                </td>
                <td class="flex gap-1">
                  <button 
                    class="btn btn-ghost btn-xs"
                    title="Edit snapshot name"
                    on:click={() => openEditModal(snapshot)}
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiPencil} />
                    </svg>
                  </button>
                  
                  <button 
                    class="btn btn-ghost btn-xs"
                    title="Clone snapshot"
                    on:click={() => openCloneModal(snapshot)}
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiContentCopy} />
                    </svg>
                  </button>
                  
                  {#if !['NR', 'R'].includes(snapshot.active)}
                    <button 
                      class="btn btn-ghost btn-xs text-success tooltip"
                      data-tip="Activate snapshot"
                      on:click={() => confirmActivateSnapshot(snapshot)}
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiCheck} />
                      </svg>
                    </button>
                  {/if}
                  
                  <button 
                    class="btn btn-ghost btn-xs text-error"
                    title="Delete snapshot"
                    on:click={() => confirmDeleteSnapshot(snapshot)}
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiDelete} />
                    </svg>
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      
      <!-- Mobile View (sm and below) -->
      <div class="grid grid-cols-1 gap-4 md:hidden">
        {#each snapshots as snapshot (snapshot.uuid)}
          <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-4">
              <div class="flex justify-between items-start">
                <h3 class="font-bold text-lg break-all">{snapshot.name}</h3>
                {#if snapshot.active === 'NR'}
                  <div class="badge badge-success tooltip" data-tip="Active now and on reboot">NR</div>
                {:else if snapshot.active === 'N'}
                  <div class="badge badge-info tooltip" data-tip="Active now">N</div>
                {:else if snapshot.active === 'R'}
                  <div class="badge badge-warning tooltip" data-tip="Active after reboot">R</div>
                {:else}
                  <div class="badge badge-ghost tooltip" data-tip="Not active">-</div>
                {/if}
              </div>
              
              <div class="grid grid-cols-2 gap-x-4 gap-y-2 my-2 text-sm">
                <div class="text-base-content/70">Size:</div>
                <div>{snapshot.size}</div>
                
                <div class="text-base-content/70">Created:</div>
                <div>{snapshot.created_str || formatDate(snapshot.created)}</div>
                
                <div class="text-base-content/70">Mount Point:</div>
                <div>{snapshot.mountpoint || '-'}</div>
              </div>
              
              <div class="card-actions justify-end flex-wrap mt-2">
                <button 
                  class="btn btn-sm btn-outline"
                  on:click={() => openEditModal(snapshot)}
                >
                  <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                    <path fill="currentColor" d={mdiPencil} />
                  </svg>
                  Edit
                </button>
                
                <button 
                  class="btn btn-sm btn-outline"
                  on:click={() => openCloneModal(snapshot)}
                >
                  <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                    <path fill="currentColor" d={mdiContentCopy} />
                  </svg>
                  Clone
                </button>
                
                {#if !['NR', 'R'].includes(snapshot.active)}
                  <button 
                    class="btn btn-sm btn-outline btn-success"
                    on:click={() => confirmActivateSnapshot(snapshot)}
                  >
                    <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiShieldCheckOutline} />
                    </svg>
                    Activate
                  </button>
                {/if}
                
                <button 
                  class="btn btn-sm btn-outline btn-error"
                  on:click={() => confirmDeleteSnapshot(snapshot)}
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

      {#if totalSnapshots > rowsPerPage}
        <div class="flex justify-center mt-4">
          <div class="join">
            <button 
              class="join-item btn btn-sm" 
              disabled={currentPage === 1}
              on:click={() => handlePageChange(currentPage - 1)}
            >
              «
            </button>
            
            <!-- Desktop pagination (show all page numbers) -->
            <div class="hidden sm:flex">
              {#each Array(Math.ceil(totalSnapshots / rowsPerPage)) as _, i}
                <button 
                  class="join-item btn btn-sm {currentPage === i + 1 ? 'btn-active' : ''}" 
                  on:click={() => handlePageChange(i + 1)}
                >
                  {i + 1}
                </button>
              {/each}
            </div>
            
            <!-- Mobile pagination (show current of total) -->
            <div class="sm:hidden join-item btn btn-sm">
              {currentPage} / {Math.ceil(totalSnapshots / rowsPerPage)}
            </div>
            
            <button 
              class="join-item btn btn-sm" 
              disabled={currentPage === Math.ceil(totalSnapshots / rowsPerPage)}
              on:click={() => handlePageChange(currentPage + 1)}
            >
              »
            </button>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</AppLayout>

<!-- Add Snapshot Modal -->
{#if showAddModal}
  <div class="modal modal-open">
    <div class="modal-box max-w-sm">
      <h3 class="font-bold text-lg mb-4">
        {currentSnapshot?.uuid ? 'Clone Snapshot' : 'Create New Snapshot'}
      </h3>
      <form on:submit|preventDefault={addSnapshot} class="space-y-4">
        <div class="form-control">
          <label class="label" for="snapshot-name">
            <span class="label-text">Snapshot Name</span>
          </label>
          <input 
            type="text" 
            id="snapshot-name"
            class="input input-bordered w-full" 
            bind:value={newSnapshotName}
            on:input={handleNameInput}
            disabled={isProcessing}
            placeholder="Enter snapshot name"
            required
          />
          <label class="label">
            <span class="label-text-alt text-info">Spaces → underscores (_)</span>
          </label>
        </div>
        
        <div class="modal-action">
          <button 
            type="button"
            class="btn" 
            on:click={() => showAddModal = false}
            disabled={isProcessing}
          >
            Cancel
          </button>
          <button 
            type="submit"
            class="btn btn-primary" 
            disabled={isProcessing}
          >
            {#if isProcessing}
              <span class="loading loading-spinner loading-sm"></span>
              {currentSnapshot?.uuid ? 'Cloning...' : 'Creating...'}
            {:else if currentSnapshot?.uuid}
              Clone Snapshot
            {:else}
              Create Snapshot
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Edit Snapshot Modal -->
{#if showEditModal}
  <div class="modal modal-open">
    <div class="modal-box max-w-sm">
      <h3 class="font-bold text-lg mb-4">Edit Snapshot</h3>
      <form on:submit|preventDefault={updateSnapshot} class="space-y-4">
        <div class="form-control">
          <label class="label" for="edit-snapshot-name">
            <span class="label-text">Snapshot Name</span>
          </label>
          <input 
            type="text" 
            id="edit-snapshot-name"
            class="input input-bordered w-full" 
            bind:value={newSnapshotName}
            on:input={handleNameInput}
            disabled={isProcessing}
            placeholder="Enter snapshot name"
            required
          />
          <label class="label">
            <span class="label-text-alt text-info">Spaces → underscores (_)</span>
          </label>
        </div>
        
        <div class="modal-action">
          <button 
            type="button"
            class="btn" 
            on:click={() => showEditModal = false}
            disabled={isProcessing}
          >
            Cancel
          </button>
          <button 
            type="submit"
            class="btn btn-primary" 
            disabled={isProcessing}
          >
            {#if isProcessing}
              <span class="loading loading-spinner loading-sm"></span>
              Saving...
            {:else}
              Save Changes
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Confirm Delete</h3>
      <p class="py-4">
        Are you sure you want to delete the snapshot <span class="font-semibold break-all">{currentSnapshot?.name || ''}</span>?
        <br>
        This action cannot be undone.
      </p>
      <div class="modal-action">
        <button 
          class="btn" 
          on:click={() => showDeleteModal = false}
          disabled={isProcessing}
        >
          Cancel
        </button>
        <button 
          class="btn btn-error" 
          on:click={deleteSnapshot}
          disabled={isProcessing}
        >
          {#if isProcessing}
            <span class="loading loading-spinner loading-sm"></span>
            Deleting...
          {:else}
            Delete
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Activate Confirmation Modal -->
{#if showActivateModal}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Activate Snapshot</h3>
      <div class="py-4 space-y-4">
        <p>
          Are you sure you want to activate the snapshot <span class="font-semibold break-all">{currentSnapshot?.name || ''}</span>?
        </p>
        
        <div class="bg-base-200 p-3 rounded-lg text-sm">
          <p>This will mark the snapshot to be used on next reboot (indicated by 'R').</p>
          
          <div class="mt-2 text-xs">
            <p class="font-medium mb-1">Snapshot states:</p>
            <ul class="space-y-1 pl-4">
              <li><span class="badge badge-info badge-sm mr-1">N</span> Active now</li>
              <li><span class="badge badge-warning badge-sm mr-1">R</span> Active after reboot</li>
              <li><span class="badge badge-success badge-sm mr-1">NR</span> Active now and on reboots</li>
              <li><span class="badge badge-ghost badge-sm mr-1">-</span> Not active</li>
            </ul>
          </div>
        </div>
        
        <div class="form-control">
          <label class="label cursor-pointer justify-start gap-2">
            <input type="checkbox" class="checkbox checkbox-primary" bind:checked={rebootAfterActivate} />
            <span class="label-text">
              <span class="block sm:hidden">Reboot after activation</span>
              <span class="hidden sm:block">Reboot firewall to apply changes immediately</span>
            </span>
          </label>
        </div>
      </div>
      
      <div class="modal-action">
        <button 
          class="btn" 
          on:click={() => showActivateModal = false}
          disabled={isProcessing}
        >
          Cancel
        </button>
        <button 
          class="btn btn-success" 
          on:click={activateSnapshot}
          disabled={isProcessing}
        >
          {#if isProcessing}
            <span class="loading loading-spinner loading-sm"></span>
            {rebootAfterActivate ? 'Activating & Rebooting...' : 'Activating...'}
          {:else if rebootAfterActivate}
            Activate & Reboot
          {:else}
            Activate
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

