<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import AppLayout from '../AppLayout.svelte';
  import SettingsForm from '$lib/components/forms/SettingsForm.svelte';
  import Login from '$lib/components/forms/Login.svelte';
  import { toasts } from '$lib/stores/toastStore';
  import { authStore } from '$lib/stores/authStore';

  let apiKey = "";
  let apiSecret = "";
  let apiUrl = "";
  let port = 443;
  let pin = "";
  let currentPin = "";
  let newPin = "";
  let confirmNewPin = "";
  let activeTab: 'api' | 'pin' = 'api';
  let isFirstRun = false;
  let isUpdatingPin = false;

  onMount(async () => {
    if ($authStore.isLoggedIn) {
      await loadApiInfo();
    }
    isFirstRun = await invoke<boolean>("check_first_run");
  });

  async function loadApiInfo() {
    try {
      const apiInfo = await invoke<{
        api_key: string;
        api_secret: string;
        api_url: string;
        port: number;
      }>("get_api_info");
      if (apiInfo) {
        apiKey = apiInfo.api_key;
        apiSecret = apiInfo.api_secret;
        apiUrl = apiInfo.api_url;
        port = apiInfo.port;
      }
    } catch (error) {
      console.error("Failed to fetch API info:", error);
      toasts.error("Failed to load settings. Please try again.");
    }
  }

  async function handleApiSubmit(event: CustomEvent<{profileName: string, apiKey: string, apiSecret: string, apiUrl: string, port: number, pin: string}>) {
  const { profileName, apiKey, apiSecret, apiUrl, port, pin } = event.detail;
  try {
    if (isFirstRun) {
      await invoke("save_initial_config", { profileName, apiKey, apiSecret, apiUrl, port: Number(port), pin });
      isFirstRun = false;
      authStore.setConfigured(true);
      toasts.success("Initial configuration saved successfully!");
    } else {
      await invoke("update_api_info", { 
        profileName, 
        apiKey, 
        apiSecret, 
        apiUrl, 
        port: Number(port),
        isDefault: true
      });
      
      // Add this line to clear the traffic cache after updating API info
      await invoke("clear_traffic_cache");
      
      toasts.success("API information updated successfully!");
    }
    await loadApiInfo();
  } catch (error) {
    console.error("Failed to update API info:", error);
    toasts.error(`Failed to update API info: ${error}`);
  }
}

  async function handlePinSubmit() {
    if (isUpdatingPin) return; // Prevent multiple submissions
    
    if (newPin !== confirmNewPin) {
      toasts.error("New PINs do not match.");
      return;
    }

    if (!/^\d+$/.test(newPin)) {
      toasts.error("PIN must contain only numbers.");
      return;
    }

    isUpdatingPin = true; // Set this before the async operation
    
    try {
      // Use setTimeout to give the UI a chance to update before starting the CPU-intensive operation
      setTimeout(async () => {
        try {
          await invoke("update_pin", { currentPin, newPin, confirmNewPin });
          
          // Clear form values
          currentPin = "";
          newPin = "";
          confirmNewPin = "";
          
          toasts.success("PIN updated successfully! Please log in with your new PIN.");
          
          // Give the UI time to show the success message
          setTimeout(() => {
            handleLogout();
          }, 1500);
        } catch (error) {
          console.error("Failed to update PIN:", error);
          isUpdatingPin = false; // Important to reset this in case of error
          
          if (error instanceof Error) {
            toasts.error(error.message);
          } else {
            toasts.error("An unknown error occurred.");
          }
        }
      }, 50); // Small delay to allow the UI to update first
    } catch (error) {
      isUpdatingPin = false; // Reset in case of error
      console.error("Unexpected error:", error);
      toasts.error("An unexpected error occurred.");
    }
  }

  function handleLogout() {
    authStore.logout();
    goto('/');
  }

  function setActiveTab(tab: 'api' | 'pin') {
    activeTab = tab;
  }

  function handleFormError(event: CustomEvent<{ message: string }>) {
    toasts.error(event.detail.message);
  }

  function handleFormSuccess(event: CustomEvent<{ message: string }>) {
    toasts.success(event.detail.message);
  }
</script>

{#if $authStore.isLoggedIn || isFirstRun}
  <AppLayout>
    <div class="max-w-4xl mx-auto">
      <h2 class="text-2xl font-bold mb-6">Settings</h2>
      
      {#if !isFirstRun}
        <div class="tabs tabs-boxed mb-6">
          <button 
            type="button"
            class="tab {activeTab === 'api' ? 'tab-active' : ''}" 
            on:click={() => setActiveTab('api')}
            disabled={isUpdatingPin}
          >
            API Settings
          </button>
          <button 
            type="button"
            class="tab {activeTab === 'pin' ? 'tab-active' : ''}" 
            on:click={() => setActiveTab('pin')}
            disabled={isUpdatingPin}
          >
            Change PIN
          </button>
        </div>
      {/if}

      {#if isFirstRun || activeTab === 'api'}
        <div class="bg-base-100 p-6 rounded-lg shadow-lg">
          <h3 class="text-xl font-semibold mb-4">
            {isFirstRun ? 'Initial Configuration' : 'API Configuration'}
          </h3>
          <SettingsForm 
            {apiKey}
            {apiSecret}
            {apiUrl}
            {port}
            {pin}
            showPin={isFirstRun}
            on:submit={handleApiSubmit}
            on:error={handleFormError}
          />
        </div>
      {:else if activeTab === 'pin'}
        <div class="bg-base-100 p-6 rounded-lg shadow-lg">
          <h3 class="text-xl font-semibold mb-4">Change PIN</h3>
          <form on:submit|preventDefault={handlePinSubmit} class="space-y-4">
            <div class="form-control">
              <label class="label" for="currentPin">
                <span class="label-text">Current PIN</span>
              </label>
              <input 
                id="currentPin"
                bind:value={currentPin}
                type="password"
                inputmode="numeric"
                pattern="\d*"
                placeholder="Enter current PIN"
                class="input input-bordered w-full"
                required
                disabled={isUpdatingPin}
              />
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label" for="newPin">
                  <span class="label-text">New PIN</span>
                </label>
                <input 
                  id="newPin"
                  bind:value={newPin}
                  type="password"
                  inputmode="numeric"
                  pattern="\d*"
                  placeholder="Enter new PIN"
                  class="input input-bordered w-full"
                  required
                  disabled={isUpdatingPin}
                />
              </div>
              <div class="form-control">
                <label class="label" for="confirmNewPin">
                  <span class="label-text">Confirm New PIN</span>
                </label>
                <input 
                  id="confirmNewPin"
                  bind:value={confirmNewPin}
                  type="password"
                  inputmode="numeric"
                  pattern="\d*"
                  placeholder="Confirm new PIN"
                  class="input input-bordered w-full"
                  required
                  disabled={isUpdatingPin}
                />
              </div>
            </div>
            <div class="flex justify-end mt-6">
              <button type="submit" class="btn btn-primary" disabled={isUpdatingPin}>
                {#if isUpdatingPin}
                  <span class="loading loading-spinner loading-sm mr-2"></span>
                  Updating PIN...
                {:else}
                  Update PIN
                {/if}
              </button>
            </div>
            
            {#if isUpdatingPin}
              <div class="mt-4 text-sm text-info">
                <p>PIN update in progress. This might take a moment. Please wait...</p>
              </div>
            {/if}
          </form>
        </div>
      {/if}
    </div>
  </AppLayout>
{:else}
  <Login />
{/if}