<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AppLayout from "../AppLayout.svelte";
  import AddFirewallRuleModal from "$lib/components/firewall/AddFirewallRuleModal.svelte";
  import EditFirewallRuleModal from "$lib/components/firewall/EditFirewallRuleModal.svelte";
  import { authStore } from "$lib/stores/authStore";
  import { toasts } from "$lib/stores/toastStore";
  import {
    mdiRefresh,
    mdiPlus,
    mdiCheck,
    mdiClose,
    mdiDelete,
    mdiPencil,
  } from "@mdi/js";

  interface FirewallRule {
    uuid: string;
    enabled: string;
    sequence: string;
    description: string;
    isToggling?: boolean;
  }

  let rules: FirewallRule[] = [];
  let isLoading = true;
  let error: string | null = null;
  let refreshInterval: number;
  let showAddRuleModal = false;
  let showEditRuleModal = false;
  let showDeleteConfirmation = false;
  let selectedRule: FirewallRule | null = null;
  let editRuleUuid = "";
  const REFRESH_INTERVAL = 30000; // 30 seconds

  onMount(async () => {
    if ($authStore.isLoggedIn) {
      await fetchRules();
      startPeriodicRefresh();
    }
  });

  onDestroy(() => {
    stopPeriodicRefresh();
  });

  function startPeriodicRefresh() {
    refreshInterval = setInterval(updateRules, REFRESH_INTERVAL);
  }

  function stopPeriodicRefresh() {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  }

  async function fetchRules() {
    isLoading = true;
    error = null;
    try {
      const response = await invoke<{ rows: FirewallRule[] }>(
        "get_firewall_rules",
      );
      rules = response.rows;
    } catch (err) {
      console.error("Failed to fetch firewall rules:", err);
      error =
        err instanceof Error ? err.message : "An unexpected error occurred";
    } finally {
      isLoading = false;
    }
  }

  async function toggleRule(rule: FirewallRule) {
    const originalStatus = rule.enabled;
    try {
      rule.isToggling = true;
      const toggleResponse = await invoke<{ result: string; changed: boolean }>(
        "toggle_firewall_rule",
        { uuid: rule.uuid },
      );

      if (toggleResponse.changed) {
        const applyResponse = await invoke<{ status: string }>(
          "apply_firewall_changes",
        );
        if (applyResponse.status.trim() === "OK") {
          await updateRules();
        } else {
          throw new Error("Failed to apply changes");
        }
      }
    } catch (err) {
      console.error("Failed to toggle firewall rule:", err);
      rule.enabled = originalStatus;
      toasts.error("Failed to toggle firewall rule");
    } finally {
      rule.isToggling = false;
    }
  }

  async function updateRules() {
    try {
      const response = await invoke<{ rows: FirewallRule[] }>(
        "get_firewall_rules",
      );
      const newRules = response.rows;

      let hasChanges = false;
      newRules.forEach((newRule, index) => {
        if (
          index < rules.length &&
          JSON.stringify(newRule) !== JSON.stringify(rules[index])
        ) {
          hasChanges = true;
          rules[index] = newRule;
        }
      });

      if (hasChanges || newRules.length !== rules.length) {
        rules = [...newRules];
      }
    } catch (err) {
      console.error("Failed to update firewall rules:", err);
    }
  }

  function manualRefresh() {
    stopPeriodicRefresh();
    fetchRules().then(() => startPeriodicRefresh());
  }

  function openAddRuleModal() {
    showAddRuleModal = true;
  }

  function openEditRuleModal(rule: FirewallRule) {
    editRuleUuid = rule.uuid;
    showEditRuleModal = true;
  }

  function handleRuleAdded() {
    fetchRules();
  }

  function handleRuleEdited() {
    fetchRules();
  }

  function openDeleteConfirmation(rule: FirewallRule) {
    selectedRule = rule;
    showDeleteConfirmation = true;
  }

  function closeDeleteConfirmation() {
    showDeleteConfirmation = false;
    selectedRule = null;
  }

  async function deleteRule() {
    if (!selectedRule) return;

    try {
      await invoke("delete_firewall_rule", { uuid: selectedRule.uuid });
      await invoke("apply_firewall_changes");
      await fetchRules();
      toasts.success("Rule deleted successfully");
      closeDeleteConfirmation();
    } catch (error) {
      console.error("Failed to delete rule:", error);
      toasts.error(`Failed to delete rule: ${error}`);
    }
  }
</script>

<AppLayout>
  <div class="max-w-6xl mx-auto">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-2xl font-bold">Firewall Rules</h2>

      <!-- Add Rule Button -->
      <button class="btn btn-primary" on:click={openAddRuleModal}>
        <svg class="w-5 h-5 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiPlus} />
        </svg>
        Add Rule
      </button>
    </div>

    {#if isLoading}
      <div class="text-center">
        <span class="loading loading-spinner loading-lg"></span>
        <p class="mt-4 text-base-content">Loading firewall rules...</p>
      </div>
    {:else if error}
      <p class="text-error">Error: {error}</p>
    {:else if rules.length === 0}
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
          <h3 class="card-title text-lg">No Automation Rules Found</h3>
          <div class="divider my-2"></div>

          <div class="space-y-4">
            <p>
              Your rules list is empty because OPNsense only exposes <strong
                >Automation Rules</strong
              > through its API, not the regular interface firewall rules.
            </p>

            <div class="alert alert-info shadow-lg">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                class="stroke-current shrink-0 w-6 h-6"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                ></path>
              </svg>
              <div>
                <span class="font-medium"
                  >This is not an error with the application.</span
                > This is a limitation of the OPNsense API.
              </div>
            </div>

            <h4 class="font-medium text-md">What you need to know:</h4>
            <ul class="list-disc pl-5 space-y-2">
              <li>
                OPNsense has two separate rule systems:
                <ul class="list-circle pl-5 mt-1">
                  <li>
                    <strong>Interface Rules</strong> - Created through the UI but
                    not accessible via API
                  </li>
                  <li>
                    <strong>Automation Rules</strong> - Created specifically for
                    API access and automation
                  </li>
                </ul>
              </li>
              <li>
                Rules created in the OPNsense web interface are not shown here
              </li>
              <li>
                Rules created through this app will appear in the "Automation"
                section of your OPNsense firewall
              </li>
            </ul>

            <div class="bg-base-200 p-4 rounded-lg">
              <h4 class="font-medium">To create your first automation rule:</h4>
              <ol class="list-decimal pl-5 mt-2 space-y-1">
                <li>Click the "Add Rule" button above</li>
                <li>Configure your rule settings</li>
                <li>Save the rule to see it appear in this list</li>
              </ol>
            </div>

            <p>
              <a
                href="https://docs.opnsense.org/development/api/core/firewall.html"
                target="_blank"
                rel="noopener noreferrer"
                class="link link-primary"
              >
                Learn more about OPNsense Firewall API limitations
              </a>
            </p>
          </div>
        </div>
      </div>
    {:else}
      <div class="space-y-4">
        {#each rules as rule (rule.uuid)}
          <div class="card bg-base-100 shadow-xl">
            <div
              class="card-body p-4 flex flex-row items-center justify-between"
            >
              <div class="flex-1">
                <div class="flex items-start gap-3">
                  <div>
                    <span class="text-sm opacity-70">#{rule.sequence}</span>
                    <h3 class="text-lg font-medium">
                      {rule.description || "Unnamed Rule"}
                    </h3>
                  </div>
                </div>
              </div>
              <div class="flex gap-2">
                <!-- Enable/Disable Button -->
                <button
                  class="btn btn-sm {rule.enabled === '1'
                    ? 'btn-success'
                    : 'btn-error'}"
                  on:click={() => toggleRule(rule)}
                  disabled={rule.isToggling}
                  title={rule.enabled === "1"
                    ? "Rule Enabled (click to disable)"
                    : "Rule Disabled (click to enable)"}
                >
                  {#if rule.isToggling}
                    <span class="loading loading-spinner loading-xs"></span>
                  {:else}
                    <svg class="w-4 h-4 sm:mr-1" viewBox="0 0 24 24">
                      <path
                        fill="currentColor"
                        d={rule.enabled === "1" ? mdiCheck : mdiClose}
                      />
                    </svg>
                    <span class="hidden sm:inline"
                      >{rule.enabled === "1" ? "Enabled" : "Disabled"}</span
                    >
                  {/if}
                </button>

                <!-- Edit Button -->
                <button
                  class="btn btn-sm btn-ghost"
                  on:click={() => openEditRuleModal(rule)}
                  title="Edit Rule"
                >
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="currentColor" d={mdiPencil} />
                  </svg>
                </button>

                <!-- Delete Button -->
                <button
                  class="btn btn-sm btn-ghost text-error"
                  on:click={() => openDeleteConfirmation(rule)}
                  title="Delete Rule"
                >
                  <svg class="w-4 h-4" viewBox="0 0 24 24">
                    <path fill="currentColor" d={mdiDelete} />
                  </svg>
                </button>
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
      on:click={manualRefresh}
      class="btn btn-circle btn-lg btn-primary shadow-lg"
      title="Refresh Rules"
    >
      <svg class="w-6 h-6" viewBox="0 0 24 24">
        <path fill="currentColor" d={mdiRefresh} />
      </svg>
    </button>
  </div>

  <!-- Add Rule Modal -->
  <AddFirewallRuleModal
    bind:showModal={showAddRuleModal}
    on:refresh={handleRuleAdded}
  />

  <!-- Edit Rule Modal -->
  <EditFirewallRuleModal
    bind:showModal={showEditRuleModal}
    ruleUuid={editRuleUuid}
    on:refresh={handleRuleEdited}
  />

  <!-- Delete Confirmation Modal -->
  {#if showDeleteConfirmation && selectedRule}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg mb-2">Confirm Delete</h3>
        <p>
          Are you sure you want to delete the rule{selectedRule.description
            ? ` "${selectedRule.description}"`
            : ""}?
        </p>
        <p class="text-sm text-error mt-2">This action cannot be undone.</p>

        <div class="modal-action">
          <button class="btn btn-outline" on:click={closeDeleteConfirmation}>
            Cancel
          </button>
          <button class="btn btn-error" on:click={deleteRule}> Delete </button>
        </div>
      </div>
    </div>
  {/if}
</AppLayout>

<style>
  .btn-circle {
    @apply rounded-full w-14 h-14 p-0 grid place-items-center;
  }

  .btn-lg {
    @apply w-16 h-16;
  }
</style>
