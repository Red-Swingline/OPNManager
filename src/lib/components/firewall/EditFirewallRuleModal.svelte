<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { invoke } from "@tauri-apps/api/core";
    import { toasts } from '$lib/stores/toastStore';
    import { mdiClose } from '@mdi/js';
  
    export let showModal = false;
    export let ruleUuid = '';
  
    const dispatch = createEventDispatcher();
    
    let isSubmitting = false;
    let isLoading = true;
    let ruleTemplate = null;
    let networkSelectOptions = null;
    let currentRule = null;
    let isNewApiVersion = false; // Flag to indicate if we're using the newer v25+ API
    
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
      source_type: "network", // field for source type selection
      source_net: "any",
      source_not: "0",
      source_port: "",
      destination_type: "network", // field for destination type selection
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
      if (showModal && ruleUuid) {
        await loadData();
      }
    });
  
    $: if (showModal && ruleUuid && !currentRule) {
      loadData();
    }
  
    async function loadData() {
      isLoading = true;
      try {
        // First, check the API version
        isNewApiVersion = await invoke('check_api_version');
        
        // Load data in parallel for efficiency
        const [ruleTemplateResult, networkSelectOptionsResult, currentRuleResult] = await Promise.all([
          invoke('get_rule_template'),
          invoke('list_network_select_options'),
          invoke('get_rule', { uuid: ruleUuid })
        ]);
        
        ruleTemplate = ruleTemplateResult;
        networkSelectOptions = networkSelectOptionsResult;
        currentRule = currentRuleResult;
        
        console.log("API Version:", isNewApiVersion ? "v25+" : "v24");
        
        // Parse network options
        if (networkSelectOptions?.aliases?.items) {
          aliasOptions = networkSelectOptions.aliases.items;
        }
        
        if (networkSelectOptions?.networks?.items) {
          networkOptions = networkSelectOptions.networks.items;
        }
        
        // Parse template options
        parseTemplate();
        
        // Populate rule data from current rule
        populateRuleData();
      } catch (error) {
        console.error('Failed to load rule data:', error);
        toasts.error(`Failed to load rule data: ${error}`);
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
      }
  
      // Include all protocols from the template without filtering
      if (ruleTemplate.rule.protocol) {
        protocols = Object.entries(ruleTemplate.rule.protocol).map(([key, option]) => ({
          key,
          value: option.value,
          selected: option.selected === 1
        }));
        
        // Make sure we at least have the 'any' protocol
        if (!protocols.some(p => p.key.toLowerCase() === 'any')) {
          protocols.unshift({
            key: 'any',
            value: 'Any',
            selected: false
          });
        }
      } else {
        // Default protocols if none were provided
        protocols = [
          { key: 'any', value: 'Any', selected: true },
          { key: 'tcp', value: 'TCP', selected: false },
          { key: 'udp', value: 'UDP', selected: false },
          { key: 'icmp', value: 'ICMP', selected: false }
        ];
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
  
    function populateRuleData() {
      if (!currentRule || !currentRule.rule) return;
      
      const rule = currentRule.rule;
      
      // Determine source and destination types based on the values
      let sourceType = 'network';
      let destType = 'network';
      
      // Check if source contains commas, indicating multiple values
      if (rule.source_net && rule.source_net.includes(',')) {
        // Multiple values - check the first one to determine type
        const sourceValues = rule.source_net.split(',');
        
        // Check if these are aliases
        if (aliasOptions && sourceValues.every(val => Object.keys(aliasOptions).includes(val))) {
          sourceType = 'alias';
        } 
        // Otherwise assume they're networks
        else {
          sourceType = 'network';
        }
      }
      // Check if single source is an alias
      else if (aliasOptions && rule.source_net && Object.keys(aliasOptions).includes(rule.source_net)) {
        sourceType = 'alias';
      } 
      // Check if source is likely to be a single host (has /32 or is an IP without subnet)
      else if (rule.source_net && (rule.source_net.endsWith('/32') || /^\d+\.\d+\.\d+\.\d+$/.test(rule.source_net))) {
        sourceType = 'host';
      }
      
      // Check if destination contains commas, indicating multiple values
      if (rule.destination_net && rule.destination_net.includes(',')) {
        // Multiple values - check the first one to determine type
        const destValues = rule.destination_net.split(',');
        
        // Check if these are aliases
        if (aliasOptions && destValues.every(val => Object.keys(aliasOptions).includes(val))) {
          destType = 'alias';
        } 
        // Otherwise assume they're networks
        else {
          destType = 'network';
        }
      }
      // Check if single destination is an alias
      else if (aliasOptions && rule.destination_net && Object.keys(aliasOptions).includes(rule.destination_net)) {
        destType = 'alias';
      } 
      // Check if destination is likely to be a single host
      else if (rule.destination_net && (rule.destination_net.endsWith('/32') || /^\d+\.\d+\.\d+\.\d+$/.test(rule.destination_net))) {
        destType = 'host';
      }
      
      // Populate the form data
      // Extract the protocol value - preserve the original value
      let protocolValue = "any";
      try {
        if (rule.protocol) {
          // Extract the protocol value and keep it exactly as is
          protocolValue = Object.keys(rule.protocol).find(key => rule.protocol[key].selected === 1) || "any";
          console.log(`Found protocol: ${protocolValue}`);
        }
      } catch (error) {
        console.error("Error parsing protocol:", error);
        protocolValue = "any";
      }
      
      // Make sure categories is loaded in the correct format (array)
      const categoriesValue = Array.isArray(rule.categories) ? rule.categories : [];
      
      // Handle source and destination values - they could be comma-separated strings
      // For multi-select to work, we need to convert to arrays
      let sourceNetValue = rule.source_net || "any";
      let destNetValue = rule.destination_net || "any";
      
      // If these are comma-separated strings, convert to arrays for the multi-select
      if (sourceType === 'alias' || sourceType === 'network') {
        if (typeof sourceNetValue === 'string' && sourceNetValue.includes(',')) {
          sourceNetValue = sourceNetValue.split(',');
        }
      }
      
      if (destType === 'alias' || destType === 'network') {
        if (typeof destNetValue === 'string' && destNetValue.includes(',')) {
          destNetValue = destNetValue.split(',');
        }
      }
      
      ruleData = {
        enabled: rule.enabled || "1",
        sequence: rule.sequence || "1",
        action: Object.keys(rule.action || {}).find(key => rule.action[key].selected === 1) || "pass",
        quick: rule.quick || "1",
        interface: Object.keys(rule.interface || {}).find(key => rule.interface[key].selected === 1) || "",
        direction: Object.keys(rule.direction || {}).find(key => rule.direction[key].selected === 1) || "in",
        ipprotocol: Object.keys(rule.ipprotocol || {}).find(key => rule.ipprotocol[key].selected === 1) || "inet",
        protocol: protocolValue,
        source_type: sourceType,
        source_net: sourceNetValue,
        source_not: rule.source_not || "0",
        source_port: rule.source_port || "",
        destination_type: destType,
        destination_net: destNetValue,
        destination_not: rule.destination_not || "0",
        destination_port: rule.destination_port || "",
        gateway: Object.keys(rule.gateway || {}).find(key => rule.gateway[key].selected === 1) || "",
        log: rule.log || "0",
        categories: categoriesValue,
        description: rule.description || ""
      };
    }
  
    function closeModal() {
      showModal = false;
      resetForm();
      dispatch('close');
    }
  
    function resetForm() {
      currentRule = null;
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
    }
  
    async function handleSubmit() {
      isSubmitting = true;
      
      try {
        // Get current interface and protocol from the rule
        let currentInterface = "";
        let currentProtocol = "";
        
        if (currentRule && currentRule.rule) {
          // Extract interface
          if (currentRule.rule.interface) {
            for (const [key, val] of Object.entries(currentRule.rule.interface)) {
              if (val.selected === 1) {
                currentInterface = key;
                break;
              }
            }
          }
          
          // Extract protocol
          if (currentRule.rule.protocol) {
            for (const [key, val] of Object.entries(currentRule.rule.protocol)) {
              if (val.selected === 1) {
                currentProtocol = key;
                break;
              }
            }
          }
        }
        
        // We need to use the updated values from the form while keeping the format right
        // First, we need the original rule to get the format right
        if (!currentRule || !currentRule.rule) {
          throw new Error("Original rule data not available");
        }
        
        // Extract the original rule for reference
        const originalRule = currentRule.rule;
        
        // Remove UI-specific fields from ruleData
        const cleanedRuleData = {...ruleData};
        delete cleanedRuleData.source_type;
        delete cleanedRuleData.destination_type;
        
        // Ensure boolean-like values are strings "0" or "1"
        Object.keys(cleanedRuleData).forEach(key => {
          if (key === "enabled" || key === "quick" || key === "log" || 
              key === "source_not" || key === "destination_not") {
            cleanedRuleData[key] = cleanedRuleData[key] ? "1" : "0";
          }
          
          // Ensure sequence is a string
          if (key === "sequence" && typeof cleanedRuleData[key] !== "string") {
            cleanedRuleData[key] = String(cleanedRuleData[key]);
          }
        });
        
        // Create a baseline payload from the working curl format
        // We need to capture all fields from the form while maintaining the working structure
        const basePayload = {
          enabled: "1",
          sequence: "1",
          categories: "",
          nosync: "0",
          description: "",
          interfacenot: "0",
          interface: "opt3",
          quick: "1",
          action: "pass",
          allowopts: "0",
          direction: "in",
          ipprotocol: "inet",
          protocol: "VISA",
          source_not: "0",
          source_net: "any",
          source_port: "",
          destination_not: "0",
          destination_net: "any",
          destination_port: "",
          log: "0",
          tcpflags1: "",
          tcpflags2: "",
          sched: "",
          statetype: "keep", // The key fix - always use "keep" not "keep state"
          "state-policy": "",
          statetimeout: "",
          adaptivestart: "",
          adaptiveend: "",
          max: "",
          "max-src-nodes": "",
          "max-src-states": "",
          "max-src-conn": "",
          "max-src-conn-rate": "",
          "max-src-conn-rates": "",
          overload: "",
          nopfsync: "0", 
          shaper1: "",
          shaper2: "",
          gateway: "",
          disablereplyto: "0",
          replyto: "",
          prio: "",
          "set-prio": "",
          "set-prio-low": "",
          tos: "",
          tag: "",
          tagged: ""
        };
        
        // Handle source_net and destination_net which could be arrays for multi-select
        let sourceNet = cleanedRuleData.source_net || "any";
        let destNet = cleanedRuleData.destination_net || "any";
        
        // Convert arrays to comma-separated strings (which is what OPNsense API expects for v25+)
        if (Array.isArray(sourceNet)) {
          sourceNet = sourceNet.join(',');
        }
        
        if (Array.isArray(destNet)) {
          destNet = destNet.join(',');
        }
        
        // Create different payloads based on API version
        let payload;
        
        if (isNewApiVersion) {
          // v25+ API - Complex format with all fields and "keep" statetype
          // Now update the baseline with values from the form
          // This preserves the structure while letting users edit values
          const updatedPayload = {
            ...basePayload,
            enabled: cleanedRuleData.enabled || "1",
            sequence: cleanedRuleData.sequence || "1",
            description: cleanedRuleData.description || "",
            interface: cleanedRuleData.interface || basePayload.interface,
            quick: cleanedRuleData.quick || "1", 
            action: cleanedRuleData.action || "pass",
            direction: cleanedRuleData.direction || "in",
            ipprotocol: cleanedRuleData.ipprotocol || "inet",
            protocol: cleanedRuleData.protocol || basePayload.protocol,
            source_not: cleanedRuleData.source_not || "0",
            source_net: sourceNet,
            source_port: cleanedRuleData.source_port || "",
            destination_not: cleanedRuleData.destination_not || "0",
            destination_net: destNet,
            destination_port: cleanedRuleData.destination_port || "",
            log: cleanedRuleData.log || "0",
            // This is IMPORTANT for v25+: Keep statetype as "keep" no matter what
            statetype: "keep",
            gateway: cleanedRuleData.gateway || ""
          };
          
          // Final payload with the rule wrapper
          payload = {
            rule: updatedPayload
          };
        } else {
          // v24 API - Simpler format with just the fields we need
          payload = {
            rule: {
              enabled: cleanedRuleData.enabled || "1",
              sequence: cleanedRuleData.sequence || "1",
              action: cleanedRuleData.action || "pass",
              quick: cleanedRuleData.quick || "1",
              interface: cleanedRuleData.interface || "",
              direction: cleanedRuleData.direction || "in",
              ipprotocol: cleanedRuleData.ipprotocol || "inet",
              protocol: cleanedRuleData.protocol || "any",
              source_net: sourceNet,
              source_port: cleanedRuleData.source_port || "",
              source_not: cleanedRuleData.source_not || "0",
              destination_net: destNet,
              destination_port: cleanedRuleData.destination_port || "",
              destination_not: cleanedRuleData.destination_not || "0",
              gateway: cleanedRuleData.gateway || "",
              log: cleanedRuleData.log || "0",
              categories: "",
              description: cleanedRuleData.description || ""
            }
          };
        }
        
        // For debugging
        console.log("Using exact payload from curl request:", JSON.stringify(payload, null, 2));
        
        // Send the update to the backend with the payload
        const result = await invoke('set_rule', { uuid: ruleUuid, ruleData: payload });
        console.log("Rule update result:", result);
        
        // Apply the changes
        await invoke('apply_firewall_changes');
        toasts.success('Firewall rule updated successfully');
        dispatch('refresh');
        closeModal();
      } catch (error) {
        console.error('Failed to update firewall rule:', error);
        toasts.error(`Failed to update firewall rule: ${error}`);
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
      
      <h3 class="font-bold text-lg mb-4">Edit Firewall Rule</h3>
      
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
                <span class="label-text">Interface (optional - leave blank for any)</span>
              </label>
              <select 
                id="interface" 
                bind:value={ruleData.interface} 
                class="select select-bordered w-full"
              >
                <option value="">Any Interface</option>
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
                <div class="form-control">
                  <label class="label" for="source-alias-display">
                    <span class="label-text">Network Alias</span>
                  </label>
                  
                  {#if isNewApiVersion}
                    <!-- v25+ API: Custom Multi-select Dropdown UI for Source -->
                    <div class="dropdown w-full">
                      <!-- Display field showing selected items -->
                      <input 
                        id="source-alias-display" 
                        type="text" 
                        readonly 
                        placeholder="Select aliases..."
                        value={Array.isArray(ruleData.source_net) ? 
                          ruleData.source_net.map(k => aliasOptions[k] || k).join(', ') : 
                          (aliasOptions[ruleData.source_net] || ruleData.source_net || '')}
                        class="input input-bordered w-full cursor-pointer"
                        tabindex="0"
                      />
                      
                      <!-- Dropdown menu with checkboxes -->
                      <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full max-h-60 overflow-auto">
                        {#each Object.entries(aliasOptions) as [key, value]}
                          <li>
                            <label class="flex items-center p-2 hover:bg-base-200 rounded">
                              <input 
                                type="checkbox" 
                                class="checkbox checkbox-sm mr-2"
                                checked={Array.isArray(ruleData.source_net) ? 
                                  ruleData.source_net.includes(key) : 
                                  ruleData.source_net === key}
                                on:change={(e) => {
                                  // Initialize as array if not already
                                  if (!Array.isArray(ruleData.source_net)) {
                                    ruleData.source_net = ruleData.source_net ? [ruleData.source_net] : [];
                                  }
                                  
                                  // Add or remove based on checkbox state
                                  if (e.target.checked) {
                                    if (!ruleData.source_net.includes(key)) {
                                      ruleData.source_net = [...ruleData.source_net, key];
                                    }
                                  } else {
                                    ruleData.source_net = ruleData.source_net.filter(k => k !== key);
                                  }
                                }}
                              />
                              <span>{value}</span>
                            </label>
                          </li>
                        {/each}
                      </ul>
                    </div>
                  {:else}
                    <!-- v24 API: Simple Single-select Dropdown UI for Source -->
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
                  {/if}
                </div>
              {:else if ruleData.source_type === 'network' && Object.keys(networkOptions).length > 0}
                <!-- Custom Network Dropdown for Source -->
                <div class="form-control">
                  <label class="label" for="source-network-display">
                    <span class="label-text">Predefined Network</span>
                  </label>
                  
                  <!-- Custom Dropdown UI -->
                  <div class="dropdown w-full">
                    <!-- Display field showing selected items -->
                    <input 
                      id="source-network-display" 
                      type="text" 
                      readonly 
                      placeholder="Select networks..."
                      value={Array.isArray(ruleData.source_net) ? 
                        ruleData.source_net.map(k => k === 'any' ? 'any' : (networkOptions[k] || k)).join(', ') : 
                        (ruleData.source_net === 'any' ? 'any' : (networkOptions[ruleData.source_net] || ruleData.source_net || ''))}
                      class="input input-bordered w-full cursor-pointer"
                      tabindex="0"
                    />
                    
                    <!-- Dropdown menu with checkboxes -->
                    <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full max-h-60 overflow-auto">
                      <li>
                        <label class="flex items-center p-2 hover:bg-base-200 rounded">
                          <input 
                            type="checkbox" 
                            class="checkbox checkbox-sm mr-2"
                            checked={Array.isArray(ruleData.source_net) ? 
                              ruleData.source_net.includes('any') : 
                              ruleData.source_net === 'any'}
                            on:change={(e) => {
                              if (e.target.checked) {
                                // If "any" is selected, clear other selections
                                ruleData.source_net = ['any'];
                              } else {
                                // Initialize empty array if unchecking "any"
                                ruleData.source_net = [];
                              }
                            }}
                          />
                          <span>any</span>
                        </label>
                      </li>
                      {#each Object.entries(networkOptions) as [key, value]}
                        <li>
                          <label class="flex items-center p-2 hover:bg-base-200 rounded">
                            <input 
                              type="checkbox" 
                              class="checkbox checkbox-sm mr-2"
                              checked={Array.isArray(ruleData.source_net) ? 
                                ruleData.source_net.includes(key) : 
                                ruleData.source_net === key}
                              on:change={(e) => {
                                // Initialize as array if not already
                                if (!Array.isArray(ruleData.source_net)) {
                                  ruleData.source_net = ruleData.source_net ? [ruleData.source_net] : [];
                                }
                                
                                // If selecting a network and "any" is currently selected, remove "any"
                                if (e.target.checked && ruleData.source_net.includes('any')) {
                                  ruleData.source_net = ruleData.source_net.filter(k => k !== 'any');
                                }
                                
                                // Add or remove based on checkbox state
                                if (e.target.checked) {
                                  if (!ruleData.source_net.includes(key)) {
                                    ruleData.source_net = [...ruleData.source_net, key];
                                  }
                                } else {
                                  ruleData.source_net = ruleData.source_net.filter(k => k !== key);
                                }
                              }}
                            />
                            <span>{value}</span>
                          </label>
                        </li>
                      {/each}
                    </ul>
                  </div>
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
                <div class="form-control">
                  <label class="label" for="destination-alias-display">
                    <span class="label-text">Network Alias</span>
                  </label>
                  
                  {#if isNewApiVersion}
                    <!-- v25+ API: Custom Multi-select Dropdown UI for Destination -->
                    <div class="dropdown w-full">
                      <!-- Display field showing selected items -->
                      <input 
                        id="destination-alias-display" 
                        type="text" 
                        readonly 
                        placeholder="Select aliases..."
                        value={Array.isArray(ruleData.destination_net) ? 
                          ruleData.destination_net.map(k => aliasOptions[k] || k).join(', ') : 
                          (aliasOptions[ruleData.destination_net] || ruleData.destination_net || '')}
                        class="input input-bordered w-full cursor-pointer"
                        tabindex="0"
                      />
                      
                      <!-- Dropdown menu with checkboxes -->
                      <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full max-h-60 overflow-auto">
                        {#each Object.entries(aliasOptions) as [key, value]}
                          <li>
                            <label class="flex items-center p-2 hover:bg-base-200 rounded">
                              <input 
                                type="checkbox" 
                                class="checkbox checkbox-sm mr-2"
                                checked={Array.isArray(ruleData.destination_net) ? 
                                  ruleData.destination_net.includes(key) : 
                                  ruleData.destination_net === key}
                                on:change={(e) => {
                                  // Initialize as array if not already
                                  if (!Array.isArray(ruleData.destination_net)) {
                                    ruleData.destination_net = ruleData.destination_net ? [ruleData.destination_net] : [];
                                  }
                                  
                                  // Add or remove based on checkbox state
                                  if (e.target.checked) {
                                    if (!ruleData.destination_net.includes(key)) {
                                      ruleData.destination_net = [...ruleData.destination_net, key];
                                    }
                                  } else {
                                    ruleData.destination_net = ruleData.destination_net.filter(k => k !== key);
                                  }
                                }}
                              />
                              <span>{value}</span>
                            </label>
                          </li>
                        {/each}
                      </ul>
                    </div>
                  {:else}
                    <!-- v24 API: Simple Single-select Dropdown UI for Destination -->
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
                  {/if}
                </div>
              {:else if ruleData.destination_type === 'network' && Object.keys(networkOptions).length > 0}
                <!-- Custom Network Dropdown for Destination -->
                <div class="form-control">
                  <label class="label" for="destination-network-display">
                    <span class="label-text">Predefined Network</span>
                  </label>
                  
                  <!-- Custom Dropdown UI -->
                  <div class="dropdown w-full">
                    <!-- Display field showing selected items -->
                    <input 
                      id="destination-network-display" 
                      type="text" 
                      readonly 
                      placeholder="Select networks..."
                      value={Array.isArray(ruleData.destination_net) ? 
                        ruleData.destination_net.map(k => k === 'any' ? 'any' : (networkOptions[k] || k)).join(', ') : 
                        (ruleData.destination_net === 'any' ? 'any' : (networkOptions[ruleData.destination_net] || ruleData.destination_net || ''))}
                      class="input input-bordered w-full cursor-pointer"
                      tabindex="0"
                    />
                    
                    <!-- Dropdown menu with checkboxes -->
                    <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full max-h-60 overflow-auto">
                      <li>
                        <label class="flex items-center p-2 hover:bg-base-200 rounded">
                          <input 
                            type="checkbox" 
                            class="checkbox checkbox-sm mr-2"
                            checked={Array.isArray(ruleData.destination_net) ? 
                              ruleData.destination_net.includes('any') : 
                              ruleData.destination_net === 'any'}
                            on:change={(e) => {
                              if (e.target.checked) {
                                // If "any" is selected, clear other selections
                                ruleData.destination_net = ['any'];
                              } else {
                                // Initialize empty array if unchecking "any"
                                ruleData.destination_net = [];
                              }
                            }}
                          />
                          <span>any</span>
                        </label>
                      </li>
                      {#each Object.entries(networkOptions) as [key, value]}
                        <li>
                          <label class="flex items-center p-2 hover:bg-base-200 rounded">
                            <input 
                              type="checkbox" 
                              class="checkbox checkbox-sm mr-2"
                              checked={Array.isArray(ruleData.destination_net) ? 
                                ruleData.destination_net.includes(key) : 
                                ruleData.destination_net === key}
                              on:change={(e) => {
                                // Initialize as array if not already
                                if (!Array.isArray(ruleData.destination_net)) {
                                  ruleData.destination_net = ruleData.destination_net ? [ruleData.destination_net] : [];
                                }
                                
                                // If selecting a network and "any" is currently selected, remove "any"
                                if (e.target.checked && ruleData.destination_net.includes('any')) {
                                  ruleData.destination_net = ruleData.destination_net.filter(k => k !== 'any');
                                }
                                
                                // Add or remove based on checkbox state
                                if (e.target.checked) {
                                  if (!ruleData.destination_net.includes(key)) {
                                    ruleData.destination_net = [...ruleData.destination_net, key];
                                  }
                                } else {
                                  ruleData.destination_net = ruleData.destination_net.filter(k => k !== key);
                                }
                              }}
                            />
                            <span>{value}</span>
                          </label>
                        </li>
                      {/each}
                    </ul>
                  </div>
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
              Save Changes
            </button>
          </div>
        </form>
      {/if}
    </div>
  </div>
  {/if}
  
  <style>
    /* Component-specific styles can go here */
  </style>