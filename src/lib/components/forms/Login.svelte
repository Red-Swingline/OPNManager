<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from '$lib/stores/toastStore';

  const dispatch = createEventDispatcher();

  let pin = "";
  let isLoading = false;

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
      <div class="flex justify-center mb-8">
        <img 
          src="/logo.png" 
          alt="OPNsense Manager Logo" 
          class="w-32 h-32 object-contain"
        />
      </div>
      
      <h2 class="text-2xl font-bold text-center mb-2">Welcome Back</h2>
      <p class="text-center text-base-content/70 mb-6">Enter your PIN to access OPNManager</p>
      
      <form on:submit|preventDefault={handleSubmit} class="space-y-6">
        <div class="card bg-base-200 p-4 rounded-lg">
          <label for="pin" class="block text-sm font-medium mb-2">
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
          />
        </div>
        
        <div>
          <button 
            type="submit" 
            class="btn btn-primary w-full shadow-md text-white hover:opacity-90"
            disabled={isLoading}
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
    <div class="p-4 text-center text-xs text-base-content/50 bg-base-200 border-t border-base-300">
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