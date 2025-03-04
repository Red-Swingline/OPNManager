<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from '$lib/stores/toastStore';
  import { mdiClose } from '@mdi/js';

  export let showModal = false;

  const dispatch = createEventDispatcher();
  
  let isSubmitting = false;
  let isLoading = true;
  let ruleTemplate = null;
  let networkSelectOptions = null;
  
  // Rule data structure matching OPNsense format
  let ruleData = {
    enabled: "1",
    sequence: "1",
    action: "pass",
    quick: "1",
    interface: "",
    direction: "in",
    ipprotocol: "inet",
    protocol: "any",
    source_type: "network", // new field for source type selection
    source_net: "any",
    source_not: "0",
    source_port: "",
    destination_type: "network", // new field for destination type selection
    destination_net: "any",
    destination_not: "0",
    destination_port: "",
    gateway: "",
    log: "0",
    categories: "",
    description: ""
  };

  // Parsed options from template
  let interfaces = [];
  let protocols = [];
  let actions = [];
  let directions = [];
  let ipprotocols = [];
  let gateways = [];
  
  // Network select options
  let aliasOptions = {};
  let networkOptions = {};

  onMount(async () => {
    if (showModal) {
      await Promise.all([
        loadRuleTemplate(),
        loadNetworkSelectOptions()
      ]);
    }
  });

  $: if (showModal && !ruleTemplate) {
    loadRuleTemplate();
    loadNetworkSelectOptions();
  }

  async function loadRuleTemplate() {
    try {
      ruleTemplate = await invoke('get_rule_template');
      parseTemplate();
    } catch (error) {
      console.error('Failed to load rule template:', error);
      toasts.error(`Failed to load rule template: ${error}`);
    }
  }

  async function loadNetworkSelectOptions() {
    try {
      networkSelectOptions = await invoke('list_network_select_options');
      
      if (networkSelectOptions?.aliases?.items) {
        aliasOptions = networkSelectOptions.aliases.items;
      }
      
      if (networkSelectOptions?.networks?.items) {
        networkOptions = networkSelectOptions.networks.items;
      }
    } catch (error) {
      console.error('Failed to load network select options:', error);
      toasts.error(`Failed to load network options: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  function parseTemplate() {
    if (!ruleTemplate || !ruleTemplate.rule) return;
    
    if (ruleTemplate.rule.interface) {
      interfaces = Object.entries(ruleTemplate.rule.interface).map(([key, option]) => ({
        key,
        value: option.value,
        selected: option.selected === 1
      }));
    }

    if (ruleTemplate.rule.action) {
      actions = Object.entries(ruleTemplate.rule.action).map(([key, option]) => ({
        key,
        value: option.value,
        selected: option.selected === 1
      }));
      
      const defaultAction = actions.find(a => a.selected)?.key || 'pass';
      ruleData.action = defaultAction;
    }

    if (ruleTemplate.rule.protocol) {
      protocols = Object.entries(ruleTemplate.rule.protocol).map(([key, option]) => ({
        key,
        value: option.value,
        selected: option.selected === 1
      }));
    }

    if (ruleTemplate.rule.direction) {
      directions = Object.entries(ruleTemplate.rule.direction).map(([key, option]) => ({
        key,
        value: option.value,
        selected: option.selected === 1
      }));
    }

    if (ruleTemplate.rule.ipprotocol) {
      ipprotocols = Object.entries(ruleTemplate.rule.ipprotocol).map(([key, option]) => ({
        key,
        value: option.value,
        selected: option.selected === 1
      }));
    }

    if (ruleTemplate.rule.gateway) {
      gateways = Object.entries(ruleTemplate.rule.gateway).map(([key, option]) => ({
        key: key || "none",
        value: option.value,
        selected: option.selected === 1
      }));
    }
  }

  function closeModal() {
    showModal = false;
    resetForm();
    dispatch('close');
  }

  function resetForm() {
    ruleData = {
      enabled: "1",
      sequence: "1",
      action: "pass",
      quick: "1",
      interface: "",
      direction: "in",
      ipprotocol: "inet",
      protocol: "any",
      source_type: "network",
      source_net: "any",
      source_not: "0",
      source_port: "",
      destination_type: "network",
      destination_net: "any",
      destination_not: "0",
      destination_port: "",
      gateway: "",
      log: "0",
      categories: "",
      description: ""
    };

    if (ruleTemplate && ruleTemplate.rule) {
      const template = ruleTemplate.rule;
      ruleData.sequence = template.sequence || "1";
    }
  }

  async function handleSubmit() {
    if (!ruleData.interface) {
      toasts.error("Please select an interface");
      return;
    }

    isSubmitting = true;
    
    try {
      const payload = { rule: ruleData };
      const result = await invoke('add_firewall_rule', { ruleData: payload });
      toasts.success('Firewall rule added successfully');
      dispatch('refresh');
      closeModal();
    } catch (error) {
      console.error('Failed to add firewall rule:', error);
      toasts.error(`Failed to add firewall rule: ${error}`);
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if showModal}
<div class="modal modal-open">
  <div class="modal-box max-w-3xl">
    <button class="btn btn-sm btn-circle absolute right-2 top-2" on:click={closeModal}>
      <svg class="w-4 h-4" viewBox="0 0 24 24">
        <path fill="currentColor" d={mdiClose} />
      </svg>
    </button>
    
    <h3 class="font-bold text-lg mb-4">Add New Firewall Rule</h3>
    
    {#if isLoading}
      <div class="flex justify-center my-8">
        <span class="loading loading-spinner loading-lg"></span>
      </div>
    {:else}
      <form on:submit|preventDefault={handleSubmit} class="space-y-4">
        <!-- Enabled -->
        <div class="form-control">
          <label class="label justify-start cursor-pointer">
            <span class="label-text mr-4">Enabled</span>
            <input 
              type="checkbox" 
              checked={ruleData.enabled === "1"}
              on:change={e => ruleData.enabled = e.target.checked ? "1" : "0"}
              class="checkbox" 
            />
          </label>
        </div>

        <!-- Sequence -->
        <div class="form-control">
          <label class="label" for="sequence">
            <span class="label-text">Sequence</span>
          </label>
          <input 
            id="sequence" 
            type="number" 
            bind:value={ruleData.sequence} 
            class="input input-bordered w-full" 
            min="1"
          />
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- Action -->
          <div class="form-control">
            <label class="label" for="action">
              <span class="label-text">Action</span>
            </label>
            <select 
              id="action" 
              bind:value={ruleData.action} 
              class="select select-bordered w-full"
            >
              {#each actions as action}
                <option value={action.key}>{action.value}</option>
              {/each}
            </select>
          </div>
          
          <!-- Interface -->
          <div class="form-control">
            <label class="label" for="interface">
              <span class="label-text">Interface</span>
            </label>
            <select 
              id="interface" 
              bind:value={ruleData.interface} 
              class="select select-bordered w-full"
            >
              <option value="">Select Interface</option>
              {#each interfaces as iface}
                <option value={iface.key}>{iface.value}</option>
              {/each}
            </select>
          </div>
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- Direction -->
          <div class="form-control">
            <label class="label" for="direction">
              <span class="label-text">Direction</span>
            </label>
            <select 
              id="direction" 
              bind:value={ruleData.direction} 
              class="select select-bordered w-full"
            >
              {#each directions as direction}
                <option value={direction.key}>{direction.value}</option>
              {/each}
            </select>
          </div>
          
          <!-- IP Protocol -->
          <div class="form-control">
            <label class="label" for="ipprotocol">
              <span class="label-text">TCP/IP Version</span>
            </label>
            <select 
              id="ipprotocol" 
              bind:value={ruleData.ipprotocol} 
              class="select select-bordered w-full"
            >
              {#each ipprotocols as ipprotocol}
                <option value={ipprotocol.key}>{ipprotocol.value}</option>
              {/each}
            </select>
          </div>
        </div>

        <!-- Protocol -->
        <div class="form-control">
          <label class="label" for="protocol">
            <span class="label-text">Protocol</span>
          </label>
          <select 
            id="protocol" 
            bind:value={ruleData.protocol} 
            class="select select-bordered w-full"
          >
            {#each protocols as protocol}
              <option value={protocol.key}>{protocol.value}</option>
            {/each}
          </select>
        </div>
        
        <!-- Source Section -->
        <div class="card bg-base-200 p-4">
          <h4 class="font-semibold mb-2">Source</h4>
          
          <div class="form-control mb-2">
            <label class="label justify-start cursor-pointer">
              <span class="label-text mr-2">Invert Source</span>
              <input 
                type="checkbox" 
                checked={ruleData.source_not === "1"}
                on:change={e => ruleData.source_not = e.target.checked ? "1" : "0"}
                class="checkbox checkbox-sm" 
              />
            </label>
          </div>
          
          <div class="grid grid-cols-1 gap-4 mt-2">
            <div class="form-control">
              <label class="label" for="source-type">
                <span class="label-text">Source Type</span>
              </label>
              <select 
                id="source-type" 
                bind:value={ruleData.source_type}
                class="select select-bordered w-full"
              >
                <option value="network">Network</option>
                <option value="alias">Alias</option>
                <option value="host">Single Host</option>
              </select>
            </div>
            
            {#if ruleData.source_type === 'alias'}
              <!-- Alias Dropdown for Source -->
              <div class="form-control">
                <label class="label" for="source-alias">
                  <span class="label-text">Network Alias</span>
                </label>
                <select 
                  id="source-alias" 
                  bind:value={ruleData.source_net} 
                  class="select select-bordered w-full"
                >
                  <option value="">Select Alias</option>
                  {#each Object.entries(aliasOptions) as [key, value]}
                    <option value={key}>{value}</option>
                  {/each}
                </select>
              </div>
            {:else if ruleData.source_type === 'network' && Object.keys(networkOptions).length > 0}
              <!-- Network Dropdown for Source -->
              <div class="form-control">
                <label class="label" for="source-network-dropdown">
                  <span class="label-text">Predefined Network</span>
                </label>
                <select 
                  id="source-network-dropdown" 
                  bind:value={ruleData.source_net} 
                  class="select select-bordered w-full"
                >
                  <option value="any">any</option>
                  {#each Object.entries(networkOptions) as [key, value]}
                    <option value={key}>{value}</option>
                  {/each}
                </select>
              </div>
            {:else}
              <!-- Manual Input for Source -->
              <div class="form-control">
                <label class="label" for="source-network">
                  <span class="label-text">{ruleData.source_type === 'host' ? 'Host Address' : 'Network'}</span>
                </label>
                <input 
                  id="source-network" 
                  type="text" 
                  bind:value={ruleData.source_net} 
                  placeholder={ruleData.source_type === 'host' ? "e.g. 192.168.1.10" : "e.g. 192.168.1.0/24 or any"}
                  class="input input-bordered w-full" 
                />
              </div>
            {/if}

<style>
  /* Component-specific styles can go here */
</style>
          </div>
          
          <div class="form-control mt-2">
            <label class="label" for="source-port">
              <span class="label-text">Port</span>
            </label>
            <input 
              id="source-port" 
              type="text" 
              bind:value={ruleData.source_port} 
              placeholder="e.g. 80 or 80-443" 
              class="input input-bordered w-full" 
            />
          </div>
        </div>
        
        <!-- Destination Section -->
        <div class="card bg-base-200 p-4">
          <h4 class="font-semibold mb-2">Destination</h4>
          
          <div class="form-control mb-2">
            <label class="label justify-start cursor-pointer">
              <span class="label-text mr-2">Invert Destination</span>
              <input 
                type="checkbox" 
                checked={ruleData.destination_not === "1"}
                on:change={e => ruleData.destination_not = e.target.checked ? "1" : "0"}
                class="checkbox checkbox-sm" 
              />
            </label>
          </div>
          
          <div class="grid grid-cols-1 gap-4 mt-2">
            <div class="form-control">
              <label class="label" for="destination-type">
                <span class="label-text">Destination Type</span>
              </label>
              <select 
                id="destination-type" 
                bind:value={ruleData.destination_type}
                class="select select-bordered w-full"
              >
                <option value="network">Network</option>
                <option value="alias">Alias</option>
                <option value="host">Single Host</option>
              </select>
            </div>
            
            {#if ruleData.destination_type === 'alias'}
              <!-- Alias Dropdown for Destination -->
              <div class="form-control">
                <label class="label" for="destination-alias">
                  <span class="label-text">Network Alias</span>
                </label>
                <select 
                  id="destination-alias" 
                  bind:value={ruleData.destination_net} 
                  class="select select-bordered w-full"
                >
                  <option value="">Select Alias</option>
                  {#each Object.entries(aliasOptions) as [key, value]}
                    <option value={key}>{value}</option>
                  {/each}
                </select>
              </div>
            {:else if ruleData.destination_type === 'network' && Object.keys(networkOptions).length > 0}
              <!-- Network Dropdown for Destination -->
              <div class="form-control">
                <label class="label" for="destination-network-dropdown">
                  <span class="label-text">Predefined Network</span>
                </label>
                <select 
                  id="destination-network-dropdown" 
                  bind:value={ruleData.destination_net} 
                  class="select select-bordered w-full"
                >
                  <option value="any">any</option>
                  {#each Object.entries(networkOptions) as [key, value]}
                    <option value={key}>{value}</option>
                  {/each}
                </select>
              </div>
            {:else}
              <!-- Manual Input for Destination -->
              <div class="form-control">
                <label class="label" for="destination-network">
                  <span class="label-text">{ruleData.destination_type === 'host' ? 'Host Address' : 'Network'}</span>
                </label>
                <input 
                  id="destination-network" 
                  type="text" 
                  bind:value={ruleData.destination_net} 
                  placeholder={ruleData.destination_type === 'host' ? "e.g. 192.168.1.10" : "e.g. 192.168.1.0/24 or any"}
                  class="input input-bordered w-full" 
                />
              </div>
            {/if}
          </div>
          
          <div class="form-control mt-2">
            <label class="label" for="destination-port">
              <span class="label-text">Port</span>
            </label>
            <input 
              id="destination-port" 
              type="text" 
              bind:value={ruleData.destination_port} 
              placeholder="e.g. 80 or 80-443" 
              class="input input-bordered w-full" 
            />
          </div>
        </div>

        <!-- Gateway -->
        <div class="form-control">
          <label class="label" for="gateway">
            <span class="label-text">Gateway</span>
          </label>
          <select 
            id="gateway" 
            bind:value={ruleData.gateway} 
            class="select select-bordered w-full"
          >
            {#each gateways as gateway}
              <option value={gateway.key === "none" ? "" : gateway.key}>{gateway.value}</option>
            {/each}
          </select>
        </div>
        
        <!-- Log -->
        <div class="form-control">
          <label class="label justify-start cursor-pointer">
            <span class="label-text mr-4">Log</span>
            <input 
              type="checkbox" 
              checked={ruleData.log === "1"}
              on:change={e => ruleData.log = e.target.checked ? "1" : "0"}
              class="checkbox" 
            />
          </label>
        </div>
        
        <!-- Description -->
        <div class="form-control">
          <label class="label" for="description">
            <span class="label-text">Description</span>
          </label>
          <input 
            id="description" 
            type="text" 
            bind:value={ruleData.description} 
            placeholder="Enter rule description" 
            class="input input-bordered w-full" 
          />
        </div>
        
        <div class="modal-action">
          <button type="button" class="btn btn-outline" on:click={closeModal}>Cancel</button>
          <button 
            type="submit" 
            class="btn btn-primary" 
            disabled={isSubmitting}
          >
            {#if isSubmitting}
              <span class="loading loading-spinner loading-sm"></span>
            {/if}
            Add Rule
          </button>
        </div>
      </form>
    {/if}
  </div>
</div>
{/if}