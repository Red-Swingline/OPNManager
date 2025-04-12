<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { invoke } from "@tauri-apps/api/core";
    import { toasts } from '$lib/stores/toastStore';
    
    export let showModal = false;
    export let existingAliases: Record<string, any> = {};
    
    const dispatch = createEventDispatcher();
    
    // Form fields
    let name = "";
    let description = "";
    let content = "";
    let aliasType = "host"; // Default to host type
    let enabled = true;
    let isProcessing = false;
    
    // Available alias types
    const aliasTypes = [
      { value: "host", label: "Host(s)" },
      { value: "network", label: "Network(s)" },
      { value: "port", label: "Port(s)" },
      { value: "url", label: "URL (IPs)" },
      { value: "urltable", label: "URL Table (IPs)" },
      { value: "geoip", label: "GeoIP" },
      { value: "networkgroup", label: "Network group" },
      { value: "mac", label: "MAC address" },
      { value: "asn", label: "BGP ASN" },
      { value: "dynipv6host", label: "Dynamic IPv6 Host" },
      { value: "authgroup", label: "OpenVPN group" },
    ];
    
    const getExistingAliasOptions = () => {
      const options = [];
      for (const key in existingAliases) {
        if (!key.startsWith('__') && !key.startsWith('bogons') && !key.startsWith('virus') && !key.startsWith('ssh')) {
          options.push(key);
        }
      }
      return options;
    };
    
    const existingAliasOptions = getExistingAliasOptions();
    
    let selectedExistingAlias = "";
    
    function addExistingAlias() {
      if (selectedExistingAlias && !content.includes(selectedExistingAlias)) {
        content = content ? `${content}, ${selectedExistingAlias}` : selectedExistingAlias;
      }
    }
    
    async function addAlias() {
      if (!name) {
        toasts.error("Name is required");
        return;
      }
      
      // Validate name (starts with letter or underscore, only alphanumeric and underscore, max 32 chars)
      const nameRegex = /^[a-zA-Z_][a-zA-Z0-9_]{0,31}$/;
      if (!nameRegex.test(name)) {
        toasts.error("Invalid name format. Must start with a letter or underscore, contain only alphanumeric characters and underscore, and be max 32 characters.");
        return;
      }
      
      if (!content) {
        toasts.error("Content is required");
        return;
      }
      
      isProcessing = true;
      try {
        const result = await invoke("add_alias", {
          name,
          aliasType,
          content,
          description,
          enabled
        });
        
        if (result && result.result === "saved") {
          toasts.success(`Alias "${name}" created successfully`);
          closeModal(true);
        } else {
          toasts.error("Failed to create alias");
        }
      } catch (error) {
        console.error("Error adding alias:", error);
        toasts.error(`Failed to create alias: ${error}`);
      } finally {
        isProcessing = false;
      }
    }
    
    function closeModal(refresh = false) {
      // Reset form
      name = "";
      description = "";
      content = "";
      aliasType = "host";
      enabled = true;
      
      // Close modal and optionally trigger refresh
      showModal = false;
      if (refresh) {
        dispatch('refresh');
      }
    }
  </script>
  
  {#if showModal}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-3xl max-h-[90vh] overflow-y-auto">
      <h2 class="text-2xl font-bold mb-6">Add New Alias</h2>
      
      <form on:submit|preventDefault={addAlias} class="space-y-6">
        <!-- Enabled -->
        <div class="form-control">
          <label class="label cursor-pointer justify-start">
            <span class="label-text text-base mr-4">Enabled</span>
            <input type="checkbox" bind:checked={enabled} class="toggle toggle-primary" />
          </label>
          <span class="text-sm text-base-content/70">Enable this alias</span>
        </div>
        
        <!-- Name -->
        <div class="form-control">
          <label class="label" for="name">
            <span class="label-text text-base">Name</span>
          </label>
          <input 
            id="name"
            type="text" 
            bind:value={name}
            class="input input-bordered w-full" 
            placeholder="Enter alias name"
          />
          <span class="text-sm text-base-content/70 mt-1">
            The name must start with a letter or underscore, be less than 32 characters and only consist
            of alphanumeric characters or underscores.
          </span>
        </div>
        
        <!-- Type -->
        <div class="form-control">
          <label class="label" for="type">
            <span class="label-text text-base">Type</span>
          </label>
          <select id="type" bind:value={aliasType} class="select select-bordered w-full">
            {#each aliasTypes as type}
              <option value={type.value}>{type.label}</option>
            {/each}
          </select>
        </div>
        
        <!-- Content -->
        <div class="form-control">
          <label class="label" for="content">
            <span class="label-text text-base">Content</span>
          </label>
          
          <!-- Existing Alias Selection (if available) -->
          {#if existingAliasOptions.length > 0}
            <div class="flex mb-2">
              <select 
                bind:value={selectedExistingAlias} 
                class="select select-bordered w-full mr-2"
              >
                <option value="">Select existing alias...</option>
                {#each existingAliasOptions as alias}
                  <option value={alias}>{alias}</option>
                {/each}
              </select>
              <button 
                type="button" 
                class="btn btn-primary" 
                on:click={addExistingAlias}
                disabled={!selectedExistingAlias}
              >
                Add
              </button>
            </div>
          {/if}
          
          <textarea 
            id="content" 
            bind:value={content}
            class="textarea textarea-bordered w-full h-32" 
            placeholder="Enter content (IPs, networks, ports, etc.)"
          ></textarea>
          <span class="text-sm text-base-content/70 mt-1">
            Enter multiple items separated by commas or new lines.
          </span>
        </div>
        
        <!-- Description -->
        <div class="form-control">
          <label class="label" for="description">
            <span class="label-text text-base">Description</span>
          </label>
          <input 
            id="description"
            type="text" 
            bind:value={description}
            class="input input-bordered w-full" 
            placeholder="Enter description (optional)"
          />
        </div>
        
        <!-- Buttons -->
        <div class="flex justify-end space-x-3 mt-6">
          <button 
            type="button" 
            class="btn btn-outline" 
            on:click={() => closeModal(false)}
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
              <span class="loading loading-spinner loading-sm mr-2"></span>
            {/if}
            Save & Apply
          </button>
        </div>
      </form>
    </div>
  </div>
  {/if}