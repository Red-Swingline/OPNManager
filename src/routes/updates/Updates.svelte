<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from '$lib/stores/toastStore';
  import { mdiRefresh, mdiPackageVariant, mdiCog, mdiAlertCircle, mdiChevronDown, mdiChevronUp } from '@mdi/js';

  let firmwareStatus: any = null;
  let isChecking = false;
  let isUpdating = false;
  let showChangelogButton = false;
  let showUpgradeButton = false;
  let changelog = '';
  let showChangelog = false;
  let hasMajorUpgrade = false;
  let majorUpgradeVersion = '';
  let majorUpgradeMessage = '';
  let changelogVersion = '';
  let isMajorMessageCollapsed = true;

  onMount(async () => {
    await getFirmwareStatus();
  });

  async function getFirmwareStatus() {
    try {
      firmwareStatus = await invoke<any>('get_current_firmware_status');
      console.log('Current firmware status:', firmwareStatus);
      
      // Reset major upgrade status on initial load - we'll only show it after explicit check
      hasMajorUpgrade = false;
      majorUpgradeVersion = '';
      majorUpgradeMessage = '';
    } catch (error) {
      console.error('Failed to get current firmware status:', error);
      toasts.error('Failed to get current firmware status. Please try again.');
    }
  }

  function decodeHtmlEntities(html) {
    const textarea = document.createElement('textarea');
    textarea.innerHTML = html;
    return textarea.value;
  }

  async function checkForUpdates() {
    isChecking = true;
    showChangelogButton = false;
    showUpgradeButton = false;
    try {
      const result = await invoke<any>('check_for_updates');
      console.log('Check for updates result:', result);

      // First check for major upgrades (high priority)
      if (result.has_major_upgrade && result.major_upgrade_version) {
        hasMajorUpgrade = true;
        majorUpgradeVersion = result.major_upgrade_version;
        majorUpgradeMessage = decodeHtmlEntities(result.major_upgrade_message || '');
        changelogVersion = majorUpgradeVersion;
        showChangelogButton = true;
        showUpgradeButton = true;
        toasts.success(`Major update to version ${majorUpgradeVersion} is available.`);
      } 
      // Then check for minor upgrades
      else if (result.has_minor_upgrade && result.minor_upgrade_to) {
        console.log("Minor upgrade detected:", result.minor_upgrade_from, "→", result.minor_upgrade_to);
        showChangelogButton = true;
        showUpgradeButton = true;
        // Use the target_version or minor_upgrade_to for changelog
        changelogVersion = result.target_version || result.minor_upgrade_to;
        
        // Parse versions to compare major.minor.patch vs build
        const currentVersion = result.minor_upgrade_from || '';
        const newVersion = result.minor_upgrade_to || '';
        
        // Split into semantic parts (X.Y.Z) and build part (_N)
        const [currentBase, currentBuild = ''] = currentVersion.split('_');
        const [newBase, newBuild = ''] = newVersion.split('_');
        
        // Split base into semantic components
        const currentParts = currentBase.split('.').map(Number);
        const newParts = newBase.split('.').map(Number);
        
        console.log(`Comparing versions: ${currentVersion} → ${newVersion}`);
        console.log(`Semantic parts: ${currentParts} → ${newParts}`);
        
        // Check if this is a semantic version change (X.Y.Z) or just a build change (_N)
        if (currentBase === newBase) {
          // Same base version, just a build update
          toasts.success(`Update to build ${newVersion} is available.`);
        } 
        // Major version change (X)
        else if (newParts[0] !== currentParts[0]) {
          toasts.success(`Major update to version ${newBase} is available.`);
        }
        // Minor version change (Y)
        else if (newParts[1] !== currentParts[1]) {
          toasts.success(`Minor update to version ${newBase} is available.`);
        }
        // Patch version change (Z)
        else if (newParts[2] !== currentParts[2]) {
          toasts.success(`Patch update to version ${newBase} is available.`);
        }
        // Fallback for any other case
        else {
          toasts.success(`Update to version ${newVersion} is available.`);
        }
      }
      // Check generic update status
      else if (result.status === "update" || result.updates_available === true) {
        const currentVersion = result.product_version || 
                              (result.product?.product_version) || 
                              "unknown";
        
        console.log("Current installed version:", currentVersion);

        let updateVersion = null;

        if (result.target_version) {
          updateVersion = result.target_version;
          console.log("Found target_version:", updateVersion);
        } else if (result.product?.product_latest) {
          updateVersion = result.product.product_latest;
          console.log("Found product.product_latest:", updateVersion);
        }

        if (updateVersion && currentVersion) {
          const cleanCurrentVersion = currentVersion.split('_')[0];
          const cleanUpdateVersion = updateVersion.split('_')[0];
          
          console.log(`Comparing versions: current=${cleanCurrentVersion}, update=${cleanUpdateVersion}`);
          
          if (cleanCurrentVersion !== cleanUpdateVersion) {
            changelogVersion = updateVersion;
            showChangelogButton = true;
            showUpgradeButton = true;
            console.log(`Update available: ${cleanCurrentVersion} → ${cleanUpdateVersion}`);
            toasts.success(`Update to version ${cleanUpdateVersion} is available.`);
          } else {
            console.log(`No update needed: current version ${cleanCurrentVersion} matches available version`);
            toasts.success('Your system is already on the latest version.');
          }
        } else {
          console.warn('Cannot determine current or update version', { 
            currentVersion, 
            updateVersion, 
            result 
          });

          if (result.status === "update") {
            toasts.warning('Updates may be available, but version information is unclear.');
          } else {
            toasts.success('Your system appears to be up to date.');
          }
        }
      } else {
        toasts.success('Your system is up to date.');
      }
      
      firmwareStatus = result;
    } catch (error) {
      console.error('Failed to check for updates:', error);
      toasts.error(`Failed to check for updates: ${error}`);
    } finally {
      isChecking = false;
    }
  }

  async function getChangelog() {

    let version = firmwareStatus?.target_version;

    if (!version) {
      if (hasMajorUpgrade && majorUpgradeVersion) {
        version = majorUpgradeVersion;
      } else if (changelogVersion) {
        version = changelogVersion;
      } else if (firmwareStatus?.product?.product_latest) {
        version = firmwareStatus.product.product_latest;
      }
    }
    
    if (!version) {
      console.error('No version available for changelog');
      toasts.error('Could not determine version for changelog. Please try checking for updates again.');
      return;
    }
    
    console.log('Getting changelog for version:', version);
    
    try {
      changelog = await invoke<string>('get_changelog', { version });
      if (!changelog) {
        console.error('Empty changelog returned');
        toasts.error('No changelog available for this version.');
        return;
      }

      changelogVersion = version;
      showChangelog = true;
    } catch (error) {
      console.error('Failed to get changelog:', error);
      toasts.error(`Failed to get changelog: ${error}`);
    }
  }

  async function startUpdate() {
    isUpdating = true;
    try {
      const result = await invoke<string>('start_update');
      console.log('Update result:', result);
      toasts.success(result);
      showChangelogButton = false;
      showUpgradeButton = false;
      hasMajorUpgrade = false;
      await getFirmwareStatus();
    } catch (error) {
      console.error('Failed to start update:', error);
      toasts.error(`Failed to start update: ${error}`);
    } finally {
      isUpdating = false;
    }
  }
</script>

<div class="p-4 max-w-3xl mx-auto">
  <h2 class="text-2xl font-bold mb-4">Firmware Status</h2>

  {#if hasMajorUpgrade && majorUpgradeVersion && majorUpgradeMessage}
    <div class="mb-6 alert alert-warning">
      <div class="flex items-start w-full">
        <svg class="w-6 h-6 mr-2 flex-shrink-0" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiAlertCircle} />
        </svg>
        <div class="flex-1">
          <div class="flex justify-between items-center mb-1">
            <h3 class="font-bold">Major Update Available: {majorUpgradeVersion}</h3>
            <button 
              class="btn btn-sm btn-ghost" 
              on:click={() => isMajorMessageCollapsed = !isMajorMessageCollapsed}
            >
              <svg class="w-5 h-5" viewBox="0 0 24 24">
                <path fill="currentColor" d={isMajorMessageCollapsed ? mdiChevronDown : mdiChevronUp} />
              </svg>
            </button>
          </div>
          {#if !isMajorMessageCollapsed}
            <div class="prose prose-sm max-w-none">
              {@html majorUpgradeMessage}
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if firmwareStatus}
    <div class="card bg-base-100 shadow-xl mb-6 overflow-x-auto">
      <div class="card-body p-4">
        <table class="table w-full">
          <tbody>
            <tr>
              <td class="font-semibold whitespace-nowrap">Version</td>
              <td class="break-all">{firmwareStatus.product_version ?? 'Unknown'}</td>
            </tr>
            <tr>
              <td class="font-semibold whitespace-nowrap">Architecture</td>
              <td class="break-all">{firmwareStatus.product?.product_arch ?? 'Unknown'}</td>
            </tr>
            <tr>
              <td class="font-semibold whitespace-nowrap">Commit</td>
              <td class="break-all">{firmwareStatus.product?.product_hash ?? 'Unknown'}</td>
            </tr>
            <tr>
              <td class="font-semibold whitespace-nowrap">Mirror</td>
              <td class="break-all">{firmwareStatus.product?.product_mirror ?? 'Unknown'}</td>
            </tr>
            <tr>
              <td class="font-semibold whitespace-nowrap">Repositories</td>
              <td class="break-all">{firmwareStatus.product?.product_repos ?? 'Unknown'}</td>
            </tr>
            <tr>
              <td class="font-semibold whitespace-nowrap">Updated on</td>
              <td class="break-all">{firmwareStatus.product?.product_time ?? 'Unknown'}</td>
            </tr>
            <tr>
              <td class="font-semibold whitespace-nowrap">Checked on</td>
              <td class="break-all">{firmwareStatus.last_check ?? 'N/A'}</td>
            </tr>
            {#if hasMajorUpgrade && majorUpgradeVersion}
              <tr class="bg-warning bg-opacity-20">
                <td class="font-semibold whitespace-nowrap">Available Major Upgrade</td>
                <td class="break-all font-bold">{majorUpgradeVersion}</td>
              </tr>
            {/if}
            {#if firmwareStatus?.has_minor_upgrade && firmwareStatus?.minor_upgrade_to}
              <tr class="bg-info bg-opacity-20">
                <td class="font-semibold whitespace-nowrap">Available Minor Upgrade</td>
                <td class="break-all font-bold">{firmwareStatus.minor_upgrade_to}</td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>
    </div>
  {:else}
    <p class="mb-6">Loading current status...</p>
  {/if}

  <div class="flex flex-wrap gap-4">
    <button
      class="btn btn-primary flex-grow sm:flex-grow-0"
      on:click={checkForUpdates}
      disabled={isChecking || isUpdating}
    >
      {#if isChecking}
        <span class="loading loading-spinner"></span>
      {:else}
        <svg class="w-5 h-5 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiRefresh} />
        </svg>
      {/if}
      Check for Updates
    </button>

    {#if showChangelogButton}
      <button class="btn btn-secondary flex-grow sm:flex-grow-0" on:click={getChangelog} disabled={isUpdating}>
        <svg class="w-5 h-5 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiPackageVariant} />
        </svg>
        View Changelog
      </button>
    {/if}

    {#if showUpgradeButton}
      <button class="btn btn-accent flex-grow sm:flex-grow-0" on:click={startUpdate} disabled={isUpdating}>
        {#if isUpdating}
          <span class="loading loading-spinner"></span>
        {:else}
          <svg class="w-5 h-5 mr-2" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiCog} />
          </svg>
        {/if}
        Upgrade to {changelogVersion || firmwareStatus?.target_version || 
          (hasMajorUpgrade ? majorUpgradeVersion : 
          (firmwareStatus?.product?.product_latest || 'latest version'))}
      </button>
    {/if}
  </div>

  {#if isUpdating}
    <div class="mt-4 p-4 bg-base-200 rounded-lg">
      <h3 class="text-lg font-semibold mb-2">Update in Progress</h3>
      <p>The system is being updated. This may take several minutes and the system may reboot.</p>
      <progress class="progress progress-primary w-full mt-2" max="100"></progress>
    </div>
  {/if}

  {#if showChangelog}
    <div class="modal modal-open">
      <div class="modal-box max-w-3xl w-full">
        <h3 class="font-bold text-lg mb-4">Changelog for {changelogVersion}</h3>
        <div class="py-4 max-h-96 overflow-y-auto">
          {@html changelog}
        </div>
        <div class="modal-action">
          <button class="btn" on:click={() => showChangelog = false}>Close</button>
        </div>
      </div>
    </div>
  {/if}
</div>