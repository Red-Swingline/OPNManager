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
    mdiFilterMenu,
  } from "@mdi/js";

  interface FirewallRule {
    uuid: string;
    enabled: string;
    sequence: string;
    description: string;
    interface?: string;
    isToggling?: boolean;
  }

  interface InterfaceItem {
    value: string;
    label: string;
    selected: boolean;
  }

  interface InterfaceGroup {
    label: string;
    icon: string;
    items: InterfaceItem[];
  }

  interface InterfaceListResponse {
    floating: InterfaceGroup;
    groups: InterfaceGroup;
    interfaces: InterfaceGroup;
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

  // For interface filtering
  let isV2Api = false;
  let interfaces: InterfaceListResponse | null = null;
  let loadingInterfaces = false;
  let selectedInterface = "";
  let showInterfaceSelector = false;
  let interfaceRuleCount: Record<string, number> = {};

  onMount(async () => {
    if ($authStore.isLoggedIn) {
      // Check which API version we're using
      await checkApiVersion();
      await fetchRules();
      startPeriodicRefresh();
    }
  });

  onDestroy(() => {
    stopPeriodicRefresh();
  });

  async function checkApiVersion() {
    try {
      isV2Api = await invoke<boolean>("check_api_version");
      if (isV2Api) {
        await loadInterfaceList();
        if (Object.keys(interfaceRuleCount).length > 0) {
          // Auto-select the first interface with rules
          selectFirstInterfaceWithRules();
        }
      }
    } catch (err) {
      console.error("Failed to check API version:", err);
      isV2Api = false;
    }
  }

  async function loadInterfaceList() {
    if (!isV2Api) return;
    
    loadingInterfaces = true;
    try {
      interfaces = await invoke<InterfaceListResponse>("get_interface_list");
      
      // Prefetch rules for each interface to find which ones have rules
      await prefetchInterfaceRules();
    } catch (err) {
      console.error("Failed to load interface list:", err);
      toasts.error("Failed to load interfaces");
    } finally {
      loadingInterfaces = false;
    }
  }
  
  async function prefetchInterfaceRules() {
    if (!interfaces) return;
    
    interfaceRuleCount = {};
    
    // Get all interfaces, starting with interfaces from the API response
    const allInterfaces = [
      ...interfaces.floating.items,
      ...interfaces.groups.items,
      ...interfaces.interfaces.items
    ];
    
    // Try parallel approach first
    try {
      console.log("Attempting parallel prefetch of interface rules...");
      
      // Create an array of promises for all interfaces
      const requests = allInterfaces.map(iface => {
        // For each interface, create a promise that resolves to an object with interface and rule count
        return invoke<{ rows: FirewallRule[] }>("get_firewall_rules", { interface: iface.value })
          .then(response => ({ 
            interface: iface.value, 
            count: response.rows.length,
            success: true 
          }))
          .catch(err => ({ 
            interface: iface.value, 
            count: 0,
            success: false,
            error: err 
          }));
      });
      
      // Wait for all requests to complete (successful or not)
      const results = await Promise.all(requests);
      
      // Process results
      results.forEach(result => {
        if (result.success && result.count > 0) {
          interfaceRuleCount[result.interface] = result.count;
        }
      });
      
      // Check if we had any successful requests
      const anySuccessful = results.some(r => r.success);
      if (!anySuccessful) {
        throw new Error("All parallel requests failed, falling back to sequential");
      }
      
      // Log success message
      console.log("Successfully fetched interface rules in parallel");
      console.log("Interface rule counts:", interfaceRuleCount);
    } 
    catch (error) {
      // If parallel approach fails, fall back to sequential
      console.warn("Parallel prefetch failed, falling back to sequential:", error);
      
      // Reset interface rule count
      interfaceRuleCount = {};
      
      // Sequential approach
      for (const iface of allInterfaces) {
        try {
          const response = await invoke<{ rows: FirewallRule[] }>(
            "get_firewall_rules",
            { interface: iface.value }
          );
          
          if (response.rows.length > 0) {
            interfaceRuleCount[iface.value] = response.rows.length;
          }
        } catch (err) {
          console.error(`Failed to prefetch rules for interface ${iface.value}:`, err);
        }
      }
      
      console.log("Sequential prefetch completed");
      console.log("Interface rule counts:", interfaceRuleCount);
    }
  }
  
  function selectFirstInterfaceWithRules() {
    const interfacesWithRules = Object.keys(interfaceRuleCount);
    
    if (interfacesWithRules.length > 0) {
      // Prioritize real interfaces over groups or floating
      const regularInterfaces = interfaces?.interfaces.items
        .map(i => i.value)
        .filter(val => interfaceRuleCount[val] > 0);
      
      if (regularInterfaces && regularInterfaces.length > 0) {
        selectedInterface = regularInterfaces[0];
      } else if (interfacesWithRules.includes("")) {
        // Next, try floating rules
        selectedInterface = "";
      } else {
        // Finally, use whatever has rules
        selectedInterface = interfacesWithRules[0];
      }
      
      console.log(`Auto-selected interface: ${selectedInterface} with ${interfaceRuleCount[selectedInterface]} rules`);
      fetchRules();
    }
  }

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
        { interface: selectedInterface }
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
        { interface: selectedInterface }
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

  async function handleRuleAdded() {
    // Fetch rules for current interface
    await fetchRules();
    
    // If we're using the V2 API, update the interface rule counts
    if (isV2Api && interfaces) {
      // Update the rule count for the current interface
      const updateCurrentInterfaceCount = async () => {
        try {
          const response = await invoke<{ rows: FirewallRule[] }>(
            "get_firewall_rules",
            { interface: selectedInterface }
          );
          
          interfaceRuleCount[selectedInterface] = response.rows.length;
        } catch (err) {
          console.error("Failed to update interface rule count:", err);
        }
      };
      
      // If a rule was added to a new interface, refresh all interface counts
      if (!interfaceRuleCount[selectedInterface]) {
        await prefetchInterfaceRules();
      } else {
        await updateCurrentInterfaceCount();
      }
    }
  }

  async function handleRuleEdited() {
    // Save the current interface to restore it later if needed
    const currentInterface = selectedInterface;
    
    // Fetch rules for current interface
    await fetchRules();
    
    // If we're using the V2 API, update the interface rule counts
    // (rule edits might change interface assignments)
    if (isV2Api && interfaces) {
      await prefetchInterfaceRules();
      
      // Check if we need to switch interfaces
      // (in case the edited rule changed interfaces and there are no more rules on the current interface)
      if (rules.length === 0 && Object.keys(interfaceRuleCount).length > 0) {
        // Switch to an interface that has rules
        selectFirstInterfaceWithRules();
      }
    }
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
      
      // Fetch updated rules for current interface
      await fetchRules();
      
      // If we're using the V2 API, update the interface rule counts
      if (isV2Api && interfaces) {
        await prefetchInterfaceRules();
      }
      
      toasts.success("Rule deleted successfully");
      closeDeleteConfirmation();
    } catch (error) {
      console.error("Failed to delete rule:", error);
      toasts.error(`Failed to delete rule: ${error}`);
    }
  }

  function selectInterface(interfaceValue: string) {
    selectedInterface = interfaceValue;
    showInterfaceSelector = false;
    fetchRules();
  }

  function toggleInterfaceSelector() {
    showInterfaceSelector = !showInterfaceSelector;
    
    if (showInterfaceSelector) {
      // Add click outside listener
      setTimeout(() => {
        const clickOutsideHandler = (event) => {
          const dropdown = document.querySelector('.interface-dropdown');
          const button = document.querySelector('.interface-selector-button');
          
          if (dropdown && button && 
              !dropdown.contains(event.target) && 
              !button.contains(event.target)) {
            showInterfaceSelector = false;
            window.removeEventListener('click', clickOutsideHandler);
          }
        };
        
        window.addEventListener('click', clickOutsideHandler);
      }, 0);
    }
  }

  function getInterfaceDisplayName(value: string): string {
    if (!interfaces) return value;
    
    // First check floating
    const floatingItem = interfaces.floating.items.find(i => i.value === value);
    if (floatingItem) return floatingItem.label;

    // Then check groups
    const groupItem = interfaces.groups.items.find(i => i.value === value);
    if (groupItem) return groupItem.label;

    // Finally check interfaces
    const interfaceItem = interfaces.interfaces.items.find(i => i.value === value);
    if (interfaceItem) return interfaceItem.label;

    // If empty string and we didn't find it in floating items, it's likely "Floating"
    if (value === "") return "Floating";
    
    // Fallback
    return value;
  }
</script>

<AppLayout>
  <div class="max-w-6xl mx-auto">
    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-3 mb-6">
      <h2 class="text-2xl font-bold">Firewall Rules</h2>
      
      <div class="flex flex-1 sm:flex-none items-center gap-2 justify-between sm:justify-end">
        {#if isV2Api && interfaces}
          <!-- Interface Selector -->
          <div class="relative flex-1 sm:flex-none">
            <button 
              class="btn btn-sm interface-selector-button {selectedInterface ? 'btn-primary text-primary-content' : 'btn-outline'} flex items-center gap-2 w-full sm:w-auto justify-between" 
              on:click={toggleInterfaceSelector}
              title="Filter by Interface"
            >
              <div class="flex items-center gap-1">
                <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24">
                  <path fill="currentColor" d={mdiFilterMenu} />
                </svg>
                <span class="max-w-[120px] truncate">
                  {getInterfaceDisplayName(selectedInterface)}
                </span>
              </div>
              <div class="flex items-center gap-1">
                {#if interfaceRuleCount[selectedInterface || '']}
                  <span class="badge badge-sm badge-outline bg-white/30">
                    {interfaceRuleCount[selectedInterface || '']}
                  </span>
                {/if}
                <svg class="w-3 h-3 opacity-70 flex-shrink-0" viewBox="0 0 24 24">
                  <path fill="currentColor" d="M7,10L12,15L17,10H7Z" />
                </svg>
              </div>
            </button>
            
            {#if showInterfaceSelector}
              <div class="absolute interface-dropdown z-50 mt-1 w-full sm:w-72 left-0 right-0 sm:left-auto sm:right-auto bg-base-100 shadow-xl rounded-md overflow-hidden border border-base-300">
                <div class="p-3 bg-base-200 border-b border-base-300">
                  <h4 class="font-medium">Filter by Interface</h4>
                </div>
                <div class="max-h-80 overflow-y-auto py-2">
                  <!-- No "All Interfaces" option as it doesn't exist in OPNsense web UI -->
                  
                  <!-- Floating -->
                  <div class="mt-3 mb-2">
                    <div class="px-3 py-1.5 bg-base-200 border-y border-base-300 flex items-center gap-2">
                      <div class="w-1 h-5 bg-primary rounded-full opacity-60"></div>
                      <span class="text-sm font-medium">{interfaces.floating.label}</span>
                    </div>
                    <div class="mt-1 px-3">
                      {#each interfaces.floating.items as item}
                        <button 
                          class="w-full text-left px-3 py-2 hover:bg-base-200 rounded-md mb-1 {selectedInterface === item.value ? 'bg-primary/10 text-primary border border-primary/20' : ''} {!interfaceRuleCount[item.value] ? 'opacity-60' : ''}"
                          on:click={() => selectInterface(item.value)}
                        >
                          <div class="flex justify-between items-center">
                            <span>{item.label}</span>
                            {#if interfaceRuleCount[item.value]}
                              <span class="badge badge-sm badge-primary">{interfaceRuleCount[item.value]}</span>
                            {/if}
                          </div>
                        </button>
                      {/each}
                    </div>
                  </div>
                  
                  <!-- Groups -->
                  {#if interfaces.groups.items.length > 0}
                    <div class="mb-2">
                      <div class="px-3 py-1.5 bg-base-200 border-y border-base-300 flex items-center gap-2">
                        <div class="w-1 h-5 bg-warning rounded-full opacity-60"></div>
                        <span class="text-sm font-medium">{interfaces.groups.label}</span>
                      </div>
                      <div class="mt-1 px-3">
                        {#each interfaces.groups.items as item}
                          <button 
                            class="w-full text-left px-3 py-2 hover:bg-base-200 rounded-md mb-1 {selectedInterface === item.value ? 'bg-primary/10 text-primary border border-primary/20' : ''} {!interfaceRuleCount[item.value] ? 'opacity-60' : ''}"
                            on:click={() => selectInterface(item.value)}
                          >
                            <div class="flex justify-between items-center">
                              <span>{item.label}</span>
                              {#if interfaceRuleCount[item.value]}
                                <span class="badge badge-sm badge-primary">{interfaceRuleCount[item.value]}</span>
                              {/if}
                            </div>
                          </button>
                        {/each}
                      </div>
                    </div>
                  {/if}
                  
                  <!-- Interfaces -->
                  <div>
                    <div class="px-3 py-1.5 bg-base-200 border-y border-base-300 flex items-center gap-2">
                      <div class="w-1 h-5 bg-info rounded-full opacity-60"></div>
                      <span class="text-sm font-medium">{interfaces.interfaces.label}</span>
                    </div>
                    <div class="mt-1 px-3">
                      {#each interfaces.interfaces.items as item}
                        <button 
                          class="w-full text-left px-3 py-2 hover:bg-base-200 rounded-md mb-1 {selectedInterface === item.value ? 'bg-primary/10 text-primary border border-primary/20' : ''} {!interfaceRuleCount[item.value] ? 'opacity-60' : ''}"
                          on:click={() => selectInterface(item.value)}
                        >
                          <div class="flex justify-between items-center">
                            <span>{item.label}</span>
                            {#if interfaceRuleCount[item.value]}
                              <span class="badge badge-sm badge-primary">{interfaceRuleCount[item.value]}</span>
                            {/if}
                          </div>
                        </button>
                      {/each}
                    </div>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Add Rule Button -->
        <button class="btn btn-primary sm:ml-2" on:click={openAddRuleModal}>
          <svg class="w-5 h-5 mr-1" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiPlus} />
          </svg>
          <span class="hidden xs:inline">Add Rule</span>
          <span class="xs:hidden">Add</span>
        </button>
      </div>
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
          <h3 class="card-title text-lg">
            {#if isV2Api}
              No Rules Found for {getInterfaceDisplayName(selectedInterface)}
            {:else}
              No Automation Rules Found
            {/if}
          </h3>
          <div class="divider my-2"></div>

          <div class="space-y-4">
            {#if isV2Api}
              <!-- V2 API with interface-specific rules -->
              <p>
                There are no firewall rules defined for {getInterfaceDisplayName(selectedInterface)}.
              </p>
              
              {#if Object.keys(interfaceRuleCount).length > 0}
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
                    <span class="font-medium">Try selecting a different interface</span> from the dropdown above. Some interfaces have rules.
                  </div>
                </div>
              {:else}
                <p>You can create rules for this interface by clicking the "Add Rule" button.</p>
              {/if}
              
              <div class="bg-base-200 p-4 rounded-lg">
                <h4 class="font-medium">To create your first rule for {getInterfaceDisplayName(selectedInterface)}:</h4>
                <ol class="list-decimal pl-5 mt-2 space-y-1">
                  <li>Click the "Add Rule" button above</li>
                  <li>Configure your rule settings</li>
                  <li>Save the rule to see it appear in this list</li>
                </ol>
              </div>
            {:else}
              <!-- Legacy API with only automation rules -->
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
            {/if}

            <p>
              <a
                href="https://docs.opnsense.org/development/api/core/firewall.html"
                target="_blank"
                rel="noopener noreferrer"
                class="link link-primary"
              >
                Learn more about OPNsense Firewall API
              </a>
            </p>
          </div>
        </div>
      </div>
    {:else}
      <div>
        {#each rules as rule (rule.uuid)}
          <div class="card bg-base-100 shadow-sm hover:shadow-md transition-shadow border-l-4 {rule.enabled === '1' ? 'border-success' : 'border-error'} my-2">
            <div class="card-body p-3">
              <!-- Rule Information Row -->
              <div class="flex justify-between items-center mb-1.5">
                <div class="flex items-center gap-2">
                  <span class="badge badge-sm">#{rule.sequence}</span>
                  {#if rule.interface}
                    <span class="badge badge-sm badge-outline">
                      {rule.interface}
                    </span>
                  {/if}
                </div>
                <span
                  class="badge badge-sm {rule.enabled === '1'
                    ? 'badge-success'
                    : 'badge-error'}"
                >
                  {rule.enabled === '1' ? 'Enabled' : 'Disabled'}
                </span>
              </div>
              
              <!-- Description -->
              <h3 class="text-sm sm:text-base font-medium line-clamp-2 mb-2">
                {rule.description || "Unnamed Rule"}
              </h3>
              
              <!-- Actions Row -->
              <div class="flex items-center justify-end gap-2 mt-auto">
                <!-- Toggle Button -->
                <button
                  class="btn btn-xs sm:btn-sm {rule.enabled === '1' ? 'btn-success' : 'btn-error'} btn-outline"
                  on:click={() => toggleRule(rule)}
                  disabled={rule.isToggling}
                  aria-label={rule.enabled === "1" ? "Disable Rule" : "Enable Rule"}
                >
                  {#if rule.isToggling}
                    <span class="loading loading-spinner loading-xs"></span>
                  {:else}
                    <svg class="w-3.5 h-3.5 sm:w-4 sm:h-4" viewBox="0 0 24 24">
                      <path
                        fill="currentColor"
                        d={rule.enabled === "1" ? mdiCheck : mdiClose}
                      />
                    </svg>
                  {/if}
                </button>

                <!-- Edit Button -->
                <button
                  class="btn btn-xs sm:btn-sm btn-outline"
                  on:click={() => openEditRuleModal(rule)}
                  aria-label="Edit Rule"
                >
                  <svg class="w-3.5 h-3.5 sm:w-4 sm:h-4" viewBox="0 0 24 24">
                    <path fill="currentColor" d={mdiPencil} />
                  </svg>
                </button>

                <!-- Delete Button -->
                <button
                  class="btn btn-xs sm:btn-sm btn-outline btn-error"
                  on:click={() => openDeleteConfirmation(rule)}
                  aria-label="Delete Rule"
                >
                  <svg class="w-3.5 h-3.5 sm:w-4 sm:h-4" viewBox="0 0 24 24">
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
    selectedInterface={isV2Api ? selectedInterface : undefined}
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
