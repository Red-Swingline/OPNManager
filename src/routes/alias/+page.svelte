<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { debounce } from "lodash-es";
  import AppLayout from "../AppLayout.svelte";
  import AddAliasModal from "$lib/components/alias/AddAliasModal.svelte";
  import { toasts } from "$lib/stores/toastStore";
  import { authStore } from "$lib/stores/authStore";
  import {
    mdiRefresh,
    mdiMagnify,
    mdiPlus,
    mdiClose,
    mdiToggleSwitch,
    mdiToggleSwitchOffOutline,
    mdiDelete,
    mdiCheck,
  } from "@mdi/js";

  interface Alias {
    name: string;
    description: string;
  }

  interface AliasDetails {
    uuid: string;
    enabled: string;
    name: string;
    description: string;
    type: string;
    content: string;
    current_items: string;
    last_updated: string;
    categories_uuid: string[];
  }

  interface AliasItemsResponse {
    rows: AliasDetails[];
  }

  let aliases: Record<string, Alias> = {};
  let aliasDetails: Record<string, AliasDetails> = {};
  let filteredAliases: Record<string, Alias> = {};
  let isLoading = true;
  let isAddingIp = false;
  let isProcessing = false;
  let error: string | null = null;
  let selectedAlias: AliasDetails | null = null;
  let isModalOpen = false;
  let showAddAliasModal = false;
  let newIpAddress = "";
  let filter = "";
  let ipToRemove: string | null = null;
  let selectedIndex = -1;

  // State variables for toggle and delete actions
  let showToggleConfirmation = false;
  let showDeleteConfirmation = false;
  let aliasToToggle: AliasDetails | null = null;
  let aliasToDelete: AliasDetails | null = null;

  const debouncedApplyFilter = debounce(applyFilter, 300);

  onMount(() => {
    if ($authStore.isLoggedIn) {
      fetchAliasesAndDetails();
    }
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });

  async function fetchAliasesAndDetails(): Promise<void> {
    isLoading = true;
    error = null;
    try {
      const detailsResult =
        await invoke<AliasItemsResponse>("search_alias_items");

      if (detailsResult && detailsResult.rows) {
        const excludedPrefixes = ["bogons", "__", "virusprot", "sshlockout"];

        aliases = {};
        aliasDetails = {};

        detailsResult.rows.forEach((item) => {
          if (
            !excludedPrefixes.some((prefix) => item.name.startsWith(prefix))
          ) {
            aliases[item.name] = {
              name: item.name,
              description: item.description,
            };
            aliasDetails[item.name] = item;
          }
        });
      } else {
        throw new Error("Invalid response from search_alias_items");
      }

      applyFilter();
    } catch (err) {
      console.error("Failed to fetch aliases:", err);
      error =
        err instanceof Error ? err.message : "An unexpected error occurred";
      toasts.error(`Failed to fetch aliases: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  function applyFilter(): void {
    filteredAliases = Object.fromEntries(
      Object.entries(aliases).filter(
        ([_, alias]) =>
          alias.name.toLowerCase().includes(filter.toLowerCase()) ||
          alias.description.toLowerCase().includes(filter.toLowerCase()),
      ),
    );
    selectedIndex = -1;
  }

  function openAliasDetails(alias: Alias): void {
    selectedAlias = aliasDetails[alias.name];
    isModalOpen = true;
  }

  async function addIpToAlias(): Promise<void> {
    if (!newIpAddress || !selectedAlias) return;

    isAddingIp = true;
    try {
      const currentContent = selectedAlias.content || "";
      const updatedContent = currentContent
        ? `${currentContent}\n${newIpAddress}`
        : newIpAddress;

      await invoke("add_ip_to_alias", {
        uuid: selectedAlias.uuid,
        currentContent: updatedContent,
        newIp: newIpAddress,
      });

      await refreshAliasDetails(selectedAlias.name);
      newIpAddress = "";
      toasts.success("IP address added successfully");
    } catch (err) {
      console.error("Failed to add IP address:", err);
      toasts.error(
        `Failed to add IP address: ${err instanceof Error ? err.message : String(err)}`,
      );
    } finally {
      isAddingIp = false;
    }
  }

  function showRemoveConfirmation(ip: string): void {
    ipToRemove = ip;
  }

  async function confirmRemoveIp(): Promise<void> {
    if (ipToRemove) {
      await removeIpFromAlias(ipToRemove);
      ipToRemove = null;
    }
  }

  function cancelRemoveIp(): void {
    ipToRemove = null;
  }

  async function removeIpFromAlias(ip: string): Promise<void> {
    if (!selectedAlias) return;

    try {
      const currentContent = selectedAlias.content || "";
      const contentArray = currentContent
        .split("\n")
        .map((item) => item.trim());
      const updatedContentArray = contentArray.filter((item) => item !== ip);
      const updatedContent = updatedContentArray.join("\n");

      await invoke("remove_ip_from_alias", {
        uuid: selectedAlias.uuid,
        currentContent: updatedContent,
      });

      await refreshAliasDetails(selectedAlias.name);

      toasts.success("IP address removed successfully");
    } catch (err) {
      console.error("Failed to remove IP address:", err);
      toasts.error(
        `Failed to remove IP address: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  }

  // Function to toggle alias enabled/disabled state
  function openToggleConfirmation(alias: AliasDetails): void {
    aliasToToggle = alias;
    showToggleConfirmation = true;
  }

  async function confirmToggleAlias(): Promise<void> {
    if (!aliasToToggle) return;

    isProcessing = true;
    try {
      const result = await invoke<{ result: string; changed: boolean }>(
        "toggle_alias",
        {
          uuid: aliasToToggle.uuid,
        },
      );

      if (result.changed) {
        const action = result.result === "Enabled" ? "enabled" : "disabled";
        toasts.success(`Alias ${aliasToToggle.name} ${action} successfully`);
        await fetchAliasesAndDetails();

        // If the currently selected alias was toggled, refresh the details
        if (selectedAlias && selectedAlias.uuid === aliasToToggle.uuid) {
          selectedAlias = aliasDetails[aliasToToggle.name];
        }
      }
    } catch (err) {
      console.error("Failed to toggle alias:", err);
      toasts.error(
        `Failed to toggle alias: ${err instanceof Error ? err.message : String(err)}`,
      );
    } finally {
      isProcessing = false;
      showToggleConfirmation = false;
      aliasToToggle = null;
    }
  }

  // Function to handle alias deletion
  function openDeleteConfirmation(alias: AliasDetails): void {
    aliasToDelete = alias;
    showDeleteConfirmation = true;
  }

  async function confirmDeleteAlias(): Promise<void> {
    if (!aliasToDelete) return;

    isProcessing = true;
    try {
      const result = await invoke<{ result: string }>("delete_alias", {
        uuid: aliasToDelete.uuid,
      });

      if (result.result === "deleted") {
        toasts.success(`Alias ${aliasToDelete.name} deleted successfully`);

        // If the currently open modal is for the deleted alias, close it
        if (selectedAlias && selectedAlias.uuid === aliasToDelete.uuid) {
          closeModal();
        }

        // Refresh the alias list
        await fetchAliasesAndDetails();
      }
    } catch (err) {
      console.error("Failed to delete alias:", err);

      // Check for the "in use" error message
      const errorMessage = String(err);
      if (
        errorMessage.includes("in use") ||
        errorMessage.includes("Currently in use")
      ) {
        toasts.error(
          `Cannot delete alias "${aliasToDelete.name}" because it is being used by firewall rules. You must first remove all references to this alias.`,
          6000,
        );
      } else {
        toasts.error(`Failed to delete alias: ${errorMessage}`);
      }
    } finally {
      isProcessing = false;
      showDeleteConfirmation = false;
      aliasToDelete = null;
    }
  }

  async function refreshAliasDetails(aliasName: string): Promise<void> {
    try {
      const freshAliasDetails =
        await invoke<AliasItemsResponse>("search_alias_items");

      if (freshAliasDetails && freshAliasDetails.rows) {
        const freshAlias = freshAliasDetails.rows.find(
          (item) => item.name === aliasName,
        );
        if (freshAlias && selectedAlias) {
          selectedAlias = freshAlias;
          aliasDetails[selectedAlias.name] = freshAlias;
        }
      }
    } catch (err) {
      console.error("Failed to refresh alias details:", err);
      toasts.error(
        `Failed to refresh alias details: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  }

  function closeModal(): void {
    isModalOpen = false;
    selectedAlias = null;
  }

  async function refreshAliases(): Promise<void> {
    await fetchAliasesAndDetails();
  }

  function handleFilterInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    filter = target.value;
    debouncedApplyFilter();
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (!isModalOpen && !showAddAliasModal) {
      const aliasArray = Object.values(filteredAliases);
      if (event.key === "ArrowDown") {
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, aliasArray.length - 1);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, -1);
      } else if (event.key === "Enter" && selectedIndex !== -1) {
        event.preventDefault();
        openAliasDetails(aliasArray[selectedIndex]);
      }
    }
  }

  function openAddAliasModal() {
    showAddAliasModal = true;
  }

  function handleAddAliasRefresh() {
    fetchAliasesAndDetails();
  }

  $: {
    filter;
    if (Object.keys(aliases).length > 0) {
      debouncedApplyFilter();
    }
  }
</script>

<AppLayout>
  <div class="max-w-6xl mx-auto">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold">Alias List</h2>

      <!-- Add Alias Button -->
      <button class="btn btn-primary" on:click={openAddAliasModal}>
        <svg class="w-5 h-5 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiPlus} />
        </svg>
        Add Alias
      </button>
    </div>

    <div class="mb-4 relative">
      <input
        type="text"
        placeholder="Filter aliases"
        class="input input-bordered w-full pl-10"
        on:input={handleFilterInput}
      />
      <svg
        class="w-6 h-6 absolute left-2 top-1/2 transform -translate-y-1/2 text-base-content opacity-60"
        viewBox="0 0 24 24"
      >
        <path fill="currentColor" d={mdiMagnify} />
      </svg>
    </div>

    {#if isLoading}
      <div class="text-center">
        <span class="loading loading-spinner loading-lg"></span>
        <p class="mt-4 text-base-content">Loading aliases...</p>
      </div>
    {:else if error}
      <p class="text-error">Error: {error}</p>
    {:else if Object.keys(filteredAliases).length === 0}
      <p class="text-base-content">No aliases found.</p>
    {:else}
      <div class="space-y-4">
        {#each Object.entries(filteredAliases) as [key, alias], index}
          {@const details = aliasDetails[alias.name]}
          <div
            class="card bg-base-100 shadow-xl hover:bg-base-200 cursor-pointer transition-colors duration-200"
            class:bg-base-300={index === selectedIndex}
            class:opacity-60={details && details.enabled === "0"}
          >
            <div class="card-body">
              <div class="flex justify-between">
                <div class="flex-1" on:click={() => openAliasDetails(alias)}>
                  <h3 class="card-title">
                    {alias.name}
                    {#if details && details.enabled === "0"}
                      <span class="badge badge-error">Disabled</span>
                    {:else}
                      <span class="badge badge-success">Enabled</span>
                    {/if}
                  </h3>
                  <p>{alias.description || "No description assigned."}</p>
                </div>
                <div class="flex space-x-2">
                  <!-- Toggle button -->
                  <button
                    class="btn btn-sm btn-ghost text-info"
                    on:click|stopPropagation={(e) => {
                      e.preventDefault();
                      if (details) openToggleConfirmation(details);
                    }}
                    title={details && details.enabled === "1"
                      ? "Disable"
                      : "Enable"}
                  >
                    <svg class="w-5 h-5" viewBox="0 0 24 24">
                      <path
                        fill="currentColor"
                        d={details && details.enabled === "1"
                          ? mdiToggleSwitch
                          : mdiToggleSwitchOffOutline}
                      />
                    </svg>
                  </button>

                  <!-- Delete button -->
                  <button
                    class="btn btn-sm btn-ghost text-error"
                    on:click|stopPropagation={(e) => {
                      e.preventDefault();
                      if (details) openDeleteConfirmation(details);
                    }}
                    title="Delete"
                  >
                    <svg class="w-5 h-5" viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiDelete} />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Floating Action Button -->
  <div class="fixed bottom-6 right-6">
    <button
      on:click={refreshAliases}
      class="btn btn-circle btn-lg btn-primary shadow-lg"
      title="Refresh Aliases"
    >
      <svg class="w-6 h-6" viewBox="0 0 24 24">
        <path fill="currentColor" d={mdiRefresh} />
      </svg>
    </button>
  </div>

  <!-- Alias Details Modal -->
  {#if isModalOpen && selectedAlias}
    <div
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
    >
      <div
        class="bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto"
      >
        <div class="mb-4">
          <h2 class="text-2xl font-bold flex items-center gap-2">
            {selectedAlias.name}
            {#if selectedAlias.enabled === "0"}
              <span class="badge badge-error">Disabled</span>
            {:else}
              <span class="badge badge-success">Enabled</span>
            {/if}
          </h2>
        </div>
        <p class="mb-4">
          {selectedAlias.description || "No description assigned."}
        </p>

        <h3 class="text-xl font-semibold mb-2">IP Addresses</h3>
        {#if selectedAlias.content}
          <div class="flex flex-wrap gap-2 mb-4">
            {#each selectedAlias.content
              .split("\n")
              .map((ip) => ip.trim()) as ip}
              {#if ip}
                <div class="badge badge-lg gap-2 p-3">
                  {ip}
                  <button
                    on:click|stopPropagation={() => showRemoveConfirmation(ip)}
                    class="btn btn-ghost btn-xs p-0 min-h-0 h-auto"
                  >
                    <svg class="w-4 h-4" viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiClose} />
                    </svg>
                  </button>
                </div>
              {/if}
            {/each}
          </div>
        {:else}
          <p class="mb-4">No IP addresses assigned to this alias.</p>
        {/if}

        <div class="flex items-center mb-6">
          <input
            type="text"
            placeholder="New IP Address"
            class="input input-bordered flex-grow mr-2"
            bind:value={newIpAddress}
          />
          <button
            on:click={addIpToAlias}
            class="btn btn-primary"
            disabled={isAddingIp}
          >
            {#if isAddingIp}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              <svg class="w-6 h-6" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiPlus} />
              </svg>
            {/if}
          </button>
        </div>

        <div class="flex flex-wrap gap-2 justify-end mt-4">
          <!-- Toggle button -->
          <button
            class="btn btn-outline btn-info"
            on:click={() => openToggleConfirmation(selectedAlias)}
            title={selectedAlias.enabled === "1" ? "Disable" : "Enable"}
          >
            <svg class="w-5 h-5 mr-1" viewBox="0 0 24 24">
              <path
                fill="currentColor"
                d={selectedAlias.enabled === "1"
                  ? mdiToggleSwitch
                  : mdiToggleSwitchOffOutline}
              />
            </svg>
            {selectedAlias.enabled === "1" ? "Disable" : "Enable"}
          </button>

          <!-- Delete button -->
          <button
            class="btn btn-outline btn-error"
            on:click={() => openDeleteConfirmation(selectedAlias)}
            title="Delete"
          >
            <svg class="w-5 h-5 mr-1" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiDelete} />
            </svg>
            Delete
          </button>

          <button on:click={closeModal} class="btn btn-outline">Close</button>
        </div>
      </div>
    </div>
  {/if}
  <!-- Remove IP Confirmation Modal -->
  {#if ipToRemove}
    <div
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    >
      <div class="bg-base-100 p-6 rounded-lg shadow-xl max-w-md w-full">
        <h3 class="text-lg font-bold mb-4">Confirm Removal</h3>
        <p class="mb-4">Are you sure you want to remove {ipToRemove}?</p>
        <div class="flex justify-end space-x-2">
          <button class="btn btn-outline" on:click={cancelRemoveIp}
            >Cancel</button
          >
          <button class="btn btn-error" on:click={confirmRemoveIp}
            >Remove</button
          >
        </div>
      </div>
    </div>
  {/if}

  <!-- Toggle Alias Confirmation Modal -->
  {#if showToggleConfirmation && aliasToToggle}
    <div
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    >
      <div class="bg-base-100 p-6 rounded-lg shadow-xl max-w-md w-full">
        <h3 class="text-lg font-bold mb-4">
          Confirm {aliasToToggle.enabled === "1" ? "Disable" : "Enable"}
        </h3>
        <p class="mb-4">
          Are you sure you want to {aliasToToggle.enabled === "1"
            ? "disable"
            : "enable"} the alias "{aliasToToggle.name}"?
          {#if aliasToToggle.enabled === "1"}
            <span class="block mt-2 text-error"
              >Disabling this alias may affect firewall rules that use it.</span
            >
          {/if}
        </p>
        <div class="flex justify-end space-x-2">
          <button
            class="btn btn-outline"
            on:click={() => {
              showToggleConfirmation = false;
              aliasToToggle = null;
            }}
            disabled={isProcessing}
          >
            Cancel
          </button>
          <button
            class="btn {aliasToToggle.enabled === '1'
              ? 'btn-error'
              : 'btn-success'}"
            on:click={confirmToggleAlias}
            disabled={isProcessing}
          >
            {#if isProcessing}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              <svg class="w-5 h-5 mr-1" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiCheck} />
              </svg>
            {/if}
            Confirm
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Delete Alias Confirmation Modal -->
  {#if showDeleteConfirmation && aliasToDelete}
    <div
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    >
      <div class="bg-base-100 p-6 rounded-lg shadow-xl max-w-md w-full">
        <h3 class="text-lg font-bold mb-4">Confirm Deletion</h3>
        <p class="mb-4">
          Are you sure you want to delete the alias "{aliasToDelete.name}"?
          <span class="block mt-2 text-error font-semibold">
            Warning: This action cannot be undone. Deleting this alias may break
            any firewall rules that reference it.
          </span>
        </p>
        <div class="flex justify-end space-x-2">
          <button
            class="btn btn-outline"
            on:click={() => {
              showDeleteConfirmation = false;
              aliasToDelete = null;
            }}
            disabled={isProcessing}
          >
            Cancel
          </button>
          <button
            class="btn btn-error"
            on:click={confirmDeleteAlias}
            disabled={isProcessing}
          >
            {#if isProcessing}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              <svg class="w-5 h-5 mr-1" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiDelete} />
              </svg>
            {/if}
            Delete
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Add Alias Modal -->
  <AddAliasModal
    bind:showModal={showAddAliasModal}
    existingAliases={aliasDetails}
    on:refresh={handleAddAliasRefresh}
  />
</AppLayout>

<style>
  .btn-circle {
    @apply rounded-full w-14 h-14 p-0 grid place-items-center;
  }

  .btn-lg {
    @apply w-16 h-16;
  }
</style>
