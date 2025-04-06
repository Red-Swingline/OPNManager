<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let showPin = false;

  interface Profile {
    profile_name: string;
    is_default: boolean;
  }

  interface ApiInfo {
    api_key: string;
    api_secret: string;
    api_url: string;
    port: number;
  }

  let profiles: Profile[] = [];
  let selectedProfileName = "";
  let apiKey = "";
  let apiSecret = "";
  let apiUrl = "";
  let port = 443;
  let pin = "";
  let showDeleteConfirmation = false;
  let showAddProfileModal = false;
  let newProfileName = "";
  let newApiKey = "";
  let newApiSecret = "";
  let newApiUrl = "";
  let newPort = 443;
  
  // Add validation state
  let errors = {
    apiUrl: "",
    newApiUrl: ""
  };

  const dispatch = createEventDispatcher<{
    error: { message: string };
    success: { message: string };
    submit: {
      profileName: string;
      apiKey: string;
      apiSecret: string;
      apiUrl: string;
      port: number;
      pin: string;
    };
    profileChanged: { profileName: string };
  }>();

  onMount(async () => {
    await loadProfiles();
  });

  async function loadProfiles(): Promise<void> {
    try {
      profiles = await invoke<Profile[]>("get_api_profiles");
      if (profiles.length > 0) {
        selectedProfileName =
          profiles.find((p) => p.is_default)?.profile_name ||
          profiles[0].profile_name;
        await loadProfileInfo(selectedProfileName);
      }
    } catch (error) {
      console.error("Failed to load profiles:", error);
      dispatch("error", { message: "Failed to load profiles" });
    }
  }

  async function loadProfileInfo(profileName: string): Promise<void> {
    try {
      const profileInfo = await invoke<ApiInfo>("get_api_info", {
        profileName,
      });
      if (profileInfo) {
        apiKey = profileInfo.api_key;
        apiSecret = profileInfo.api_secret;
        apiUrl = profileInfo.api_url;
        port = profileInfo.port;
      }
    } catch (error) {
      console.error("Failed to load profile info:", error);
      dispatch("error", { message: "Failed to load profile information" });
    }
  }

  async function handleProfileChange(): Promise<void> {
    await setDefaultProfile(selectedProfileName);
    await loadProfileInfo(selectedProfileName);
  }

  async function setDefaultProfile(profileName: string): Promise<void> {
    try {
      await invoke("set_default_profile", { profileName });
      profiles = profiles.map((p) => ({
        ...p,
        is_default: p.profile_name === profileName,
      }));
      dispatch("profileChanged", { profileName });
      dispatch("success", { message: "Default profile updated successfully" });

      // Reload profile info for the newly set default profile
      await loadProfileInfo(profileName);
    } catch (error) {
      console.error("Failed to set default profile:", error);
      dispatch("error", { message: "Failed to set default profile" });
    }
  }
  
  function validateUrl(url: string): { isValid: boolean; error: string; formattedUrl: string } {
    // Reset error for this field
    let error = "";
    let formattedUrl = url;
    
    try {
      // Basic URL validation
      if (!url.trim()) {
        return { isValid: false, error: "Firewall address is required", formattedUrl };
      }

      // Ensure URL has http:// or https:// prefix
      if (!url.startsWith("https://") && !url.startsWith("http://")) {
        return { isValid: false, error: "URL must start with http:// or https://", formattedUrl };
      }

      // Parse the URL to validate it
      const urlObj = new URL(url);
      
      // Don't allow paths in the URL - only the hostname and protocol
      if (urlObj.pathname !== "/" && urlObj.pathname !== "") {
        return { isValid: false, error: "Please enter only the base URL (no paths)", formattedUrl };
      }
      
      // Don't allow port numbers in the URL field - we have a separate port field
      if (urlObj.port) {
        return { isValid: false, error: "Please enter the port number in the Port field", formattedUrl };
      }
      
      // Clean up the URL - remove trailing slash if present
      formattedUrl = `${urlObj.protocol}//${urlObj.hostname}`;
      return { isValid: true, error: "", formattedUrl };
      
    } catch (e) {
      return { isValid: false, error: "Please enter a valid URL", formattedUrl };
    }
  }

  async function handleSubmit(): Promise<void> {
    // Validate URL
    const urlValidation = validateUrl(apiUrl);
    if (!urlValidation.isValid) {
      errors.apiUrl = urlValidation.error;
      return;
    }
    
    // Use the properly formatted URL
    apiUrl = urlValidation.formattedUrl;
    
    try {
      await invoke("update_api_info", {
        profileName: selectedProfileName,
        apiKey,
        apiSecret,
        apiUrl,
        port: Number(port),
        isDefault: true, // Always set to true for the selected profile
      });
      dispatch("submit", {
        profileName: selectedProfileName,
        apiKey,
        apiSecret,
        apiUrl,
        port,
        pin,
      });
      // Don't dispatch success here, let the parent component handle it
    } catch (error) {
      dispatch("error", { message: "Failed to update API information" });
    }
  }
  
  function formatUrl(event: Event, isNew = false) {
    const input = event.target as HTMLInputElement;
    const url = input.value;
    
    if (url.trim()) {
      try {
        const urlObj = new URL(url);
        const formattedUrl = `${urlObj.protocol}//${urlObj.hostname}`;
        
        if (isNew) {
          newApiUrl = formattedUrl;
          errors.newApiUrl = "";
        } else {
          apiUrl = formattedUrl;
          errors.apiUrl = "";
        }
      } catch (e) {
        // Invalid URL, let form validation handle it
      }
    }
  }
  
  function validateNewUrl(): boolean {
    const urlValidation = validateUrl(newApiUrl);
    errors.newApiUrl = urlValidation.error;
    newApiUrl = urlValidation.formattedUrl;
    return urlValidation.isValid;
  }

  function openDeleteConfirmation(): void {
    showDeleteConfirmation = true;
  }

  function closeDeleteConfirmation(): void {
    showDeleteConfirmation = false;
  }

  async function deleteProfile(): Promise<void> {
    try {
      await invoke("delete_api_profile", { profileName: selectedProfileName });
      await loadProfiles();
      closeDeleteConfirmation();
      dispatch("success", { message: "Profile deleted successfully" });
    } catch (error) {
      console.error("Failed to delete profile:", error);
      dispatch("error", { message: error as string });
      closeDeleteConfirmation();
    }
  }
  
  function openAddProfileModal(): void {
    showAddProfileModal = true;
    
    // Reset fields and errors
    newProfileName = "";
    newApiKey = "";
    newApiSecret = "";
    newApiUrl = "";
    newPort = 443;
    errors.newApiUrl = "";
  }

  function closeAddProfileModal(): void {
    showAddProfileModal = false;
    newProfileName = "";
    newApiKey = "";
    newApiSecret = "";
    newApiUrl = "";
    newPort = 443;
    errors.newApiUrl = "";
  }

  async function addNewProfile(): Promise<void> {
    // Validate URL
    if (!validateNewUrl()) {
      return;
    }
    
    try {
      await invoke("add_api_profile", {
        profile: {
          profile_name: newProfileName,
          api_key: newApiKey,
          api_secret: newApiSecret,
          api_url: newApiUrl,
          port: Number(newPort),
        },
      });
      await loadProfiles();
      closeAddProfileModal();
      dispatch("success", { message: "New profile added successfully" });
    } catch (error) {
      console.error("Failed to add new profile:", error);
      dispatch("error", { message: "Failed to add new profile" });
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-6">
  <div class="form-control">
    <label class="label" for="profileSelect">
      <span class="label-text">Select Profile</span>
    </label>
    <div class="flex space-x-2">
      <select
        id="profileSelect"
        bind:value={selectedProfileName}
        on:change={handleProfileChange}
        class="select select-bordered flex-grow"
      >
        {#each profiles as profile}
          <option value={profile.profile_name}>
            {profile.profile_name}
            {profile.is_default ? "(Default)" : ""}
          </option>
        {/each}
      </select>
      <button
        type="button"
        class="btn btn-primary"
        on:click={openAddProfileModal}
      >
        Add Profile
      </button>
      {#if profiles.length > 1}
        <button
          type="button"
          class="btn btn-error"
          on:click={openDeleteConfirmation}
        >
          Delete
        </button>
      {/if}
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="form-control">
      <label class="label" for="apiKey">
        <span class="label-text">API Key</span>
      </label>
      <input
        id="apiKey"
        bind:value={apiKey}
        type="text"
        placeholder="API Key"
        class="input input-bordered w-full"
        required
      />
    </div>

    <div class="form-control">
      <label class="label" for="apiSecret">
        <span class="label-text">API Secret</span>
      </label>
      <input
        id="apiSecret"
        bind:value={apiSecret}
        type="password"
        placeholder="API Secret"
        class="input input-bordered w-full"
        required
      />
    </div>

    <div class="form-control">
      <label class="label" for="apiUrl">
        <span class="label-text">API URL</span>
      </label>
      <input
        id="apiUrl"
        bind:value={apiUrl}
        type="url"
        placeholder="API URL (ex: https://192.168.1.1 or http://192.168.1.1)"
        class="input input-bordered w-full {errors.apiUrl ? 'input-error' : ''}"
        on:blur={(e) => formatUrl(e)}
        required
      />
      {#if errors.apiUrl}
        <label class="label">
          <span class="label-text-alt text-error">{errors.apiUrl}</span>
        </label>
      {:else}
        <label class="label">
          <span class="label-text-alt text-xs">Enter only the base URL without trailing slashes or port number</span>
        </label>
      {/if}
    </div>

    <div class="form-control">
      <label class="label" for="port">
        <span class="label-text">Port</span>
      </label>
      <input
        id="port"
        bind:value={port}
        type="number"
        placeholder="Port"
        class="input input-bordered w-full"
        required
      />
    </div>

    {#if showPin}
      <div class="form-control">
        <label class="label" for="pin">
          <span class="label-text">PIN</span>
        </label>
        <input
          id="pin"
          bind:value={pin}
          type="password"
          inputmode="numeric"
          pattern="\d*"
          placeholder="Enter PIN"
          class="input input-bordered w-full"
          required
        />
      </div>
    {/if}
  </div>

  <div class="flex justify-end">
    <button type="submit" class="btn btn-primary">
      {showPin ? "Save Configuration" : "Update API Settings"}
    </button>
  </div>
</form>

{#if showDeleteConfirmation}
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
  >
    <div class="bg-base-100 p-6 rounded-lg max-w-sm w-full">
      <h3 class="text-lg font-bold mb-4">Confirm Deletion</h3>
      <p class="mb-4">
        Are you sure you want to delete the profile "{selectedProfileName}"?
      </p>
      <div class="flex justify-end space-x-2">
        <button class="btn btn-ghost" on:click={closeDeleteConfirmation}
          >Cancel</button
        >
        <button class="btn btn-error" on:click={deleteProfile}>Delete</button>
      </div>
    </div>
  </div>
{/if}

{#if showAddProfileModal}
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
  >
    <div class="bg-base-100 p-6 rounded-lg max-w-md w-full">
      <h3 class="text-lg font-bold mb-4">Add New Profile</h3>
      <form on:submit|preventDefault={addNewProfile} class="space-y-4">
        <div class="form-control">
          <label class="label" for="newProfileName">
            <span class="label-text">Profile Name</span>
          </label>
          <input
            id="newProfileName"
            bind:value={newProfileName}
            type="text"
            placeholder="Enter profile name"
            class="input input-bordered w-full"
            required
          />
        </div>
        <div class="form-control">
          <label class="label" for="newApiKey">
            <span class="label-text">API Key</span>
          </label>
          <input
            id="newApiKey"
            bind:value={newApiKey}
            type="text"
            placeholder="Enter API Key"
            class="input input-bordered w-full"
            required
          />
        </div>
        <div class="form-control">
          <label class="label" for="newApiSecret">
            <span class="label-text">API Secret</span>
          </label>
          <input
            id="newApiSecret"
            bind:value={newApiSecret}
            type="password"
            placeholder="Enter API Secret"
            class="input input-bordered w-full"
            required
          />
        </div>
        <div class="form-control">
          <label class="label" for="newApiUrl">
            <span class="label-text">API URL</span>
          </label>
          <input
            id="newApiUrl"
            bind:value={newApiUrl}
            type="url"
            placeholder="Enter API URL (ex: https://192.168.1.1 or http://192.168.1.1)"
            class="input input-bordered w-full {errors.newApiUrl ? 'input-error' : ''}"
            on:blur={(e) => formatUrl(e, true)}
            required
          />
          {#if errors.newApiUrl}
            <label class="label">
              <span class="label-text-alt text-error">{errors.newApiUrl}</span>
            </label>
          {:else}
            <label class="label">
              <span class="label-text-alt text-xs">Enter only the base URL without trailing slashes or port number</span>
            </label>
          {/if}
        </div>
        <div class="form-control">
          <label class="label" for="newPort">
            <span class="label-text">Port</span>
          </label>
          <input
            id="newPort"
            bind:value={newPort}
            type="number"
            placeholder="Enter Port"
            class="input input-bordered w-full"
            required
          />
        </div>
        <div class="flex justify-end space-x-2 mt-6">
          <button
            type="button"
            class="btn btn-ghost"
            on:click={closeAddProfileModal}>Cancel</button
          >
          <button type="submit" class="btn btn-primary">Add Profile</button>
        </div>
      </form>
    </div>
  </div>
{/if}