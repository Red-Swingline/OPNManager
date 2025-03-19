<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { toasts } from "$lib/stores/toastStore";
    import AppLayout from "../AppLayout.svelte";
    import { mdiRefresh, mdiCogOutline, mdiClockOutline } from "@mdi/js";
  
    let isLoading = true;
    let isApplying = false;
    let isCronLoading = false;
  
    // DNSBL Settings
    let dnsblEnabled = false;
    let safeSearch = false;
    let selectedBlocklistTypes = [];
    let dnsblLists = [];
    let whitelists = [];
    let blocklists = [];
    let wildcards = [];
    let dnsblAddress = "";
    let nxdomain = false;
  
    // Cron Job Settings
    let cronEnabled = false;
    let cronMinutes = "0";
    let cronHours = "0";
    let cronDays = "*";
    let cronMonths = "*";
    let cronWeekdays = "*";
    let existingCronJob = null;
  
    // Available blocklist options
    let blocklistOptions = [];
    
    async function loadSettings() {
      isLoading = true;
      try {
        const settings = await invoke("get_unbound_settings");
        console.log("Received settings:", settings);
        
        // Parse DNSBL settings
        if (settings?.unbound?.dnsbl) {
          const dnsbl = settings.unbound.dnsbl;
          dnsblEnabled = dnsbl.enabled === "1";
          safeSearch = dnsbl.safesearch === "1";
          
          // Reset selected types
          selectedBlocklistTypes = [];
          
          // First check if our backend added the active_types field
          if (dnsbl.active_types) {
            selectedBlocklistTypes = dnsbl.active_types.split(',');
          }
          // Otherwise handle different formats for blocklist type
          else if (typeof dnsbl.type === "string" && dnsbl.type) {
            selectedBlocklistTypes = dnsbl.type.split(',');
          } else {
            // If it's an object of options, find all selected ones
            if (dnsbl.type && typeof dnsbl.type === "object") {
              for (const [key, option] of Object.entries(dnsbl.type)) {
                if (option.selected === 1) {
                  selectedBlocklistTypes.push(key);
                }
              }
            }
          }
          
          dnsblLists = dnsbl.lists || [];
          whitelists = dnsbl.whitelists || [];
          blocklists = dnsbl.blocklists || [];
          wildcards = dnsbl.wildcards || [];
          dnsblAddress = dnsbl.address || "";
          nxdomain = dnsbl.nxdomain === "1";
        }
  
        // Get available blocklist options
        if (settings?.unbound?.dnsbl?.type) {
          blocklistOptions = Object.entries(settings.unbound.dnsbl.type).map(([key, option]) => {
            const isSelected = selectedBlocklistTypes.includes(key);
            return {
              key,
              value: option.value,
              selected: isSelected
            };
          });
          console.log("Available blocklist options:", blocklistOptions);
          console.log("Currently selected types:", selectedBlocklistTypes);
        }
  
        // Check for existing cron job
        loadCronJob();
        
      } catch (error) {
        console.error("Failed to load Unbound settings:", error);
        toasts.error(`Failed to load Unbound settings: ${error}`);
      } finally {
        isLoading = false;
      }
    }
  
    async function loadCronJob() {
      isCronLoading = true;
      try {
        existingCronJob = await invoke("get_dnsbl_cron_job");
        
        if (existingCronJob) {
          cronEnabled = true;
          cronMinutes = existingCronJob.minutes;
          cronHours = existingCronJob.hours;
          cronDays = existingCronJob.days;
          cronMonths = existingCronJob.months;
          cronWeekdays = existingCronJob.weekdays;
        }
      } catch (error) {
        console.error("Failed to check for existing cron job:", error);
        toasts.error(`Failed to check for existing cron job: ${error}`);
      } finally {
        isCronLoading = false;
      }
    }
  
    async function saveDnsblSettings() {
      isApplying = true;
      try {
        // Save DNSBL settings
        const result = await invoke("set_dnsbl_settings", {
          enabled: dnsblEnabled,
          safesearch: safeSearch,
          blocklistTypes: selectedBlocklistTypes,
          lists: dnsblLists,
          whitelists,
          blocklists,
          wildcards,
          address: dnsblAddress,
          nxdomain
        });
        
        console.log("DNSBL settings result:", result);
  
        // Apply settings
        const applyResult = await invoke("apply_dnsbl_settings");
        console.log("Apply DNSBL result:", applyResult);
  
        // Handle cron job if enabled
        if (dnsblEnabled && cronEnabled) {
          await invoke("add_dnsbl_cron_job", {
            minutes: cronMinutes,
            hours: cronHours,
            days: cronDays,
            months: cronMonths,
            weekdays: cronWeekdays
          });
        } else if (existingCronJob && existingCronJob.uuid) {
          // Remove cron job if DNSBL is disabled or cron is disabled
          await invoke("delete_dnsbl_cron_job", {
            uuid: existingCronJob.uuid
          });
        }
  
        toasts.success("Unbound DNSBL settings applied successfully!");
        loadSettings(); // Reload settings to confirm changes
      } catch (error) {
        console.error("Failed to apply Unbound DNSBL settings:", error);
        toasts.error(`Failed to apply Unbound DNSBL settings: ${error}`);
      } finally {
        isApplying = false;
      }
    }
  
    function parseTextToArray(text) {
      if (!text) return [];
      return text.split('\n')
        .map(line => line.trim())
        .filter(line => line.length > 0);
    }
  
    function arrayToText(arr) {
      if (!arr || arr.length === 0) return "";
      return arr.join('\n');
    }
  
    let whitelistText = "";
    let blocklistText = "";
    let wildcardText = "";
  
    $: whitelistText = arrayToText(whitelists);
    $: blocklistText = arrayToText(blocklists);
    $: wildcardText = arrayToText(wildcards);
  
    function updateWhitelists(event) {
      whitelists = parseTextToArray(event.target.value);
    }
  
    function updateBlocklists(event) {
      blocklists = parseTextToArray(event.target.value);
    }
  
    function updateWildcards(event) {
      wildcards = parseTextToArray(event.target.value);
    }
  
    onMount(() => {
      loadSettings();
    });
  </script>
  
  <AppLayout>
    <div class="p-4 max-w-6xl mx-auto">
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-4 gap-4">
        <h2 class="text-2xl font-bold">Unbound DNS Blocklist</h2>
        <button
          class="btn btn-primary w-full sm:w-auto"
          on:click={saveDnsblSettings}
          disabled={isApplying}
        >
          {#if isApplying}
            <span class="loading loading-spinner loading-sm"></span>
            Applying...
          {:else}
            Apply Settings
            <svg class="w-5 h-5 ml-1" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiRefresh} />
            </svg>
          {/if}
        </button>
      </div>
  
      <p class="text-base-content/70 mb-6">Configure DNS blocklists to protect your network from unwanted content and threats.</p>
  
      {#if isLoading}
        <div class="flex justify-center items-center h-64">
          <span class="loading loading-spinner loading-lg"></span>
        </div>
      {:else}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h3 class="card-title flex items-center text-primary">
              <svg class="w-6 h-6 mr-2" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiCogOutline} />
              </svg>
              DNSBL Configuration
            </h3>
            
            <div class="divider mt-0 mb-2"></div>
            
            <div class="form-control">
              <label class="label flex-wrap cursor-pointer gap-2">
                <span class="label-text font-medium text-lg">Enable DNS Blocklist</span>
                <input 
                  type="checkbox" 
                  class="toggle toggle-primary mt-1" 
                  bind:checked={dnsblEnabled}
                />
              </label>
              <p class="text-sm text-base-content/70 mt-1">Enable the usage of DNS blocklists to block unwanted domains</p>
            </div>
  
            <div class="form-control mt-4">
              <label class="label flex-wrap cursor-pointer gap-2">
                <span class="label-text font-medium text-lg">Force SafeSearch</span>
                <input 
                  type="checkbox" 
                  class="toggle toggle-primary mt-1" 
                  bind:checked={safeSearch}
                  disabled={!dnsblEnabled}
                />
              </label>
              <p class="text-sm text-base-content/70 mt-1">Force the usage of SafeSearch on Google, DuckDuckGo, Bing, Qwant, PixaBay and YouTube</p>
            </div>
  
            <div class="form-control mt-6">
              <label class="label">
                <span class="label-text font-medium text-lg">Types of DNSBL</span>
              </label>
              <div class="dropdown w-full">
                <label 
                  tabindex="0" 
                  class="w-full btn {!dnsblEnabled ? 'btn-disabled' : 'btn-outline'}"
                >
                  {selectedBlocklistTypes.length === 0 
                    ? '-- Select blocklists --' 
                    : selectedBlocklistTypes.length === 1 
                      ? blocklistOptions.find(o => o.key === selectedBlocklistTypes[0])?.value || 'Selected blocklist'
                      : `${selectedBlocklistTypes.length} blocklists selected`
                  }
                  <svg class="w-4 h-4 ml-2" viewBox="0 0 24 24">
                    <path fill="currentColor" d="M7,10L12,15L17,10H7Z" />
                  </svg>
                </label>
                <div 
                  tabindex="0" 
                  class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full max-h-96 overflow-y-auto"
                >
                  <div class="p-2 space-y-1">
                    {#each blocklistOptions as option}
                      <div class="form-control">
                        <label class="label cursor-pointer justify-start py-1 hover:bg-base-200 rounded px-2">
                          <input 
                            type="checkbox" 
                            class="checkbox checkbox-primary mr-3" 
                            checked={selectedBlocklistTypes.includes(option.key)}
                            disabled={!dnsblEnabled}
                            on:change={(e) => {
                              if (e.target.checked) {
                                if (!selectedBlocklistTypes.includes(option.key)) {
                                  selectedBlocklistTypes = [...selectedBlocklistTypes, option.key];
                                }
                              } else {
                                selectedBlocklistTypes = selectedBlocklistTypes.filter(t => t !== option.key);
                              }
                            }}
                          />
                          <span class="label-text">{option.value}</span>
                        </label>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
              <p class="text-sm text-base-content/70 mt-1">Select which kinds of DNSBL you want to use. You can select multiple.</p>
            </div>
  
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
              <div class="form-control">
                <label class="label">
                  <span class="label-text font-medium text-lg">Whitelist Domains</span>
                </label>
                <textarea 
                  class="textarea textarea-bordered h-32 font-mono text-sm" 
                  placeholder="Enter domains to whitelist (one per line)"
                  value={whitelistText}
                  on:input={updateWhitelists}
                  disabled={!dnsblEnabled}
                ></textarea>
                <p class="text-sm text-base-content/70 mt-1">List of domains to whitelist. You can use regular expressions.</p>
              </div>
  
              <div class="form-control">
                <label class="label">
                  <span class="label-text font-medium text-lg">Blocklist Domains</span>
                </label>
                <textarea 
                  class="textarea textarea-bordered h-32 font-mono text-sm" 
                  placeholder="Enter domains to blocklist (one per line)"
                  value={blocklistText}
                  on:input={updateBlocklists}
                  disabled={!dnsblEnabled}
                ></textarea>
                <p class="text-sm text-base-content/70 mt-1">List of domains to blocklist. Only exact matches are supported.</p>
              </div>
            </div>
  
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mt-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text font-medium text-lg">Wildcard Domains</span>
                </label>
                <textarea 
                  class="textarea textarea-bordered h-32 font-mono text-sm" 
                  placeholder="Enter wildcard domains to blocklist (one per line)"
                  value={wildcardText}
                  on:input={updateWildcards}
                  disabled={!dnsblEnabled}
                ></textarea>
                <p class="text-sm text-base-content/70 mt-1">List of wildcard domains to blocklist. All subdomains of the given domain will be blocked.</p>
              </div>
  
              <div>
                <div class="form-control mb-4">
                  <label class="label">
                    <span class="label-text font-medium text-lg">Destination Address</span>
                  </label>
                  <input 
                    type="text" 
                    class="input input-bordered" 
                    placeholder="Leave empty for default"
                    bind:value={dnsblAddress}
                    disabled={!dnsblEnabled}
                  />
                  <p class="text-sm text-base-content/70 mt-1">Destination IP address for blocked domains (leave empty for default)</p>
                </div>
  
                <div class="form-control mt-6">
                  <label class="label flex-wrap cursor-pointer gap-2">
                    <span class="label-text font-medium text-lg">Return NXDOMAIN</span>
                    <input 
                      type="checkbox" 
                      class="toggle toggle-primary mt-1" 
                      bind:checked={nxdomain}
                      disabled={!dnsblEnabled}
                    />
                  </label>
                  <p class="text-sm text-base-content/70 mt-1">Return NXDOMAIN instead of the destination address for blocked domains</p>
                </div>
              </div>
            </div>
  
            <div class="divider mt-10 mb-4">Automatic Updates</div>
  
            <h3 class="font-semibold flex items-center text-primary mb-4">
              <svg class="w-6 h-6 mr-2" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiClockOutline} />
              </svg>
              Blocklist Update Schedule
            </h3>
  
            {#if isCronLoading}
              <div class="flex justify-center items-center h-10 mt-4">
                <span class="loading loading-spinner loading-md"></span>
              </div>
            {:else}
              <div class="form-control">
                <label class="label flex-wrap cursor-pointer gap-2">
                  <span class="label-text font-medium text-lg">Enable Automatic Updates</span>
                  <input 
                    type="checkbox" 
                    class="toggle toggle-primary mt-1" 
                    bind:checked={cronEnabled}
                    disabled={!dnsblEnabled}
                  />
                </label>
                <p class="text-sm text-base-content/70 mt-1">Schedule automatic updates for the blocklists with cron</p>
              </div>
  
              {#if cronEnabled && dnsblEnabled}
                <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 mt-4">
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-medium">Minutes</span>
                    </label>
                    <input 
                      type="text" 
                      class="input input-bordered input-sm" 
                      placeholder="e.g., 0 or */5"
                      bind:value={cronMinutes}
                    />
                  </div>
  
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-medium">Hours</span>
                    </label>
                    <input 
                      type="text" 
                      class="input input-bordered input-sm" 
                      placeholder="e.g., 0 or */12"
                      bind:value={cronHours}
                    />
                  </div>
  
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-medium">Days</span>
                    </label>
                    <input 
                      type="text" 
                      class="input input-bordered input-sm" 
                      placeholder="e.g., * or 1-15"
                      bind:value={cronDays}
                    />
                  </div>
  
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-medium">Months</span>
                    </label>
                    <input 
                      type="text" 
                      class="input input-bordered input-sm" 
                      placeholder="e.g., * or 1,3,5"
                      bind:value={cronMonths}
                    />
                  </div>
  
                  <div class="form-control sm:col-span-2 md:col-span-2">
                    <label class="label">
                      <span class="label-text font-medium">Weekdays</span>
                    </label>
                    <div class="flex items-center">
                      <input 
                        type="text" 
                        class="input input-bordered input-sm flex-grow" 
                        placeholder="e.g., * or 1-5"
                        bind:value={cronWeekdays}
                      />
                      
                      <div class="dropdown dropdown-end ml-2">
                        <label tabindex="0" class="btn btn-sm btn-circle btn-ghost">
                          <svg class="w-5 h-5" viewBox="0 0 24 24">
                            <path fill="currentColor" d="M13,9H11V7H13M13,17H11V11H13M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z" />
                          </svg>
                        </label>
                        <div class="dropdown-content card card-compact w-64 p-2 shadow bg-base-200 text-base-content z-[1] mt-1">
                          <div class="card-body">
                            <h3 class="card-title text-sm">Cron Expression Help</h3>
                            <ul class="text-xs list-disc ml-4 space-y-1">
                              <li><span class="font-bold">*</span> - all values</li>
                              <li><span class="font-bold">*/n</span> - every n values</li>
                              <li><span class="font-bold">1-5</span> - range (Mon-Fri)</li>
                              <li><span class="font-bold">1,3,5</span> - specific values</li>
                            </ul>
                            <p class="text-xs mt-1">Default <span class="bg-base-300 px-1 rounded">0 0 * * *</span> runs at midnight every day.</p>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              {/if}
            {/if}
          </div>
        </div>
  
        <div class="mt-6 flex flex-col sm:flex-row justify-end gap-3">
          <button
            class="btn btn-primary"
            on:click={saveDnsblSettings}
            disabled={isApplying}
          >
            {#if isApplying}
              <span class="loading loading-spinner loading-sm"></span>
              Applying...
            {:else}
              Apply Settings
              <svg class="w-5 h-5 ml-1" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiRefresh} />
              </svg>
            {/if}
          </button>
        </div>
      {/if}
    </div>
  </AppLayout>