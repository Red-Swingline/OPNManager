<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from '$lib/stores/toastStore';

  const dispatch = createEventDispatcher();

  let pin = "";
  let isLoading = false;
  let loadingProfiles = true;
  let profiles = [];
  let selectedProfileName = "";

  onMount(async () => {
    await loadProfiles();
  });

  async function loadProfiles(): Promise<void> {
    try {
      loadingProfiles = true;
      profiles = await invoke("get_api_profiles");
      
      if (profiles.length > 0) {
        // Find the default profile, or use the first one if no default is set
        const defaultProfile = profiles.find(p => p.is_default);
        selectedProfileName = defaultProfile ? defaultProfile.profile_name : profiles[0].profile_name;
      }
    } catch (error) {
      console.error("Failed to load profiles:", error);
      toasts.error("Failed to load API profiles.");
    } finally {
      loadingProfiles = false;
    }
  }

  async function handleProfileChange(): Promise<void> {
    try {
      isLoading = true;
      await invoke("set_default_profile", { profileName: selectedProfileName });
      // No toast message when changing profile on login screen
    } catch (error) {
      console.error("Failed to set default profile:", error);
      toasts.error("Failed to update default profile.");
    } finally {
      isLoading = false;
    }
  }

  async function handleSubmit() {
    if (!/^\d+$/.test(pin)) {
      toasts.error("PIN must contain only numbers.");
      return;
    }

    isLoading = true;
    try {
      const result = await invoke("verify_pin", { pin });
      if (result) {
        dispatch('login');
      } else {
        toasts.error("Invalid PIN. Please try again.");
      }
    } catch (error) {
      console.error("Failed to verify PIN:", error);
      toasts.error("An error occurred. Please try again.");
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-base-200 p-4 relative overflow-hidden">
  <!-- Background pattern elements -->
  <div class="absolute inset-0 overflow-hidden opacity-10">
    <div 
      class="absolute -right-20 -top-20 w-96 h-96 rounded-full bg-primary"
    ></div>
    <div 
      class="absolute -left-20 -bottom-20 w-80 h-80 rounded-full bg-primary" 
    ></div>
    <div 
      class="absolute left-1/3 top-1/4 w-40 h-40 rounded-full bg-primary"
    ></div>
  </div>

  <div class="w-full max-w-md bg-base-100 rounded-lg shadow-2xl overflow-hidden relative z-10 border border-base-300">
    <!-- Header accent bar -->
    <div class="h-2 w-full bg-primary"></div>
    
    <div class="p-8">
      <div class="flex justify-center mb-6">
        <img 
          src="/logo.png" 
          alt="OPNsense Manager Logo" 
          class="w-24 h-24 object-contain"
        />
      </div>
      
      <h2 class="text-2xl font-bold text-center mb-2">Welcome Back</h2>
      <p class="text-center text-base-content/70 mb-6">Enter your PIN to access OPNManager</p>
      
      <form on:submit|preventDefault={handleSubmit} class="space-y-4">
        <!-- Profile Selection -->
        <div class="card bg-base-200 p-3 rounded-lg">
          <label for="profileSelect" class="block text-sm font-medium mb-1">
            Select Profile
          </label>
          {#if loadingProfiles}
            <div class="flex items-center justify-center py-1">
              <span class="loading loading-spinner loading-sm"></span>
            </div>
          {:else}
            <select
              id="profileSelect"
              bind:value={selectedProfileName}
              on:change={handleProfileChange}
              class="select select-bordered w-full bg-base-100"
              disabled={isLoading}
            >
              {#each profiles as profile}
                <option value={profile.profile_name}>
                  {profile.profile_name}
                  {profile.is_default ? " (Default)" : ""}
                </option>
              {/each}
            </select>
          {/if}
        </div>
        
        <!-- PIN Input -->
        <div class="card bg-base-200 p-3 rounded-lg">
          <label for="pin" class="block text-sm font-medium mb-1">
            Enter your PIN
          </label>
          <input 
            id="pin"
            bind:value={pin}
            type="password"
            inputmode="numeric"
            pattern="\d*"
            placeholder="••••"
            class="input input-bordered w-full bg-base-100"
            required
            disabled={isLoading}
          />
        </div>
        
        <div>
          <button 
            type="submit" 
            class="btn btn-primary w-full shadow-md text-white hover:opacity-90 mt-2"
            disabled={isLoading || loadingProfiles}
          >
            {#if isLoading}
              <span class="loading loading-spinner loading-sm mr-2"></span>
            {/if}
            Login
          </button>
        </div>
      </form>
    </div>
    
    <!-- Footer with subtle branding -->
    <div class="p-3 text-center text-xs text-base-content/50 bg-base-200 border-t border-base-300">
      OPNManager • Secure Network Administration
    </div>
  </div>
</div>

<style>
  /* Custom styling for the login button */
  button[type="submit"] {
    transition: opacity 0.3s ease;
  }
  
  button[type="submit"]:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>