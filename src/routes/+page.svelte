<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Login from "$lib/components/forms/Login.svelte";
  import AppLayout from "./AppLayout.svelte";
  import InitialSetupForm from "$lib/components/forms/InitialSetupForm.svelte";
  import SystemResourcesColumn from "$lib/components/dashboard/SystemResourcesColumn.svelte";
  import NetworkInformationColumn from "$lib/components/dashboard/NetworkInformationColumn.svelte";
  import { toasts } from "$lib/stores/toastStore";
  import { authStore } from "$lib/stores/authStore";
  import { goto } from "$app/navigation";
  import {
    mdiArrowUp,
    mdiArrowDown,
    mdiRestart,
    mdiChevronDown,
    mdiChevronUp,
    mdiRouter,
    mdiServerNetwork,
    mdiEthernet,
    mdiServer,
    mdiHarddisk
  } from "@mdi/js";

  interface InterfaceData {
    name: string;
    "bytes received": string;
    "bytes transmitted": string;
    device: string;
    driver: string;
  }

  interface InterfaceTraffic {
    interfaces: Record<string, InterfaceData>;
    time: number;
  }

  interface SystemResources {
    memory: {
      total: string;
      total_frmt: string;
      used: number;
      used_frmt: string;
    };
  }

  interface DiskDevice {
    device: string;
    device_type: string;
    blocks: string;
    used: string;
    available: string;
    used_pct: number;
    mountpoint: string;
  }

  interface SystemDisk {
    devices: DiskDevice[];
  }

  interface DashboardData {
    gatewayStatus: any;
    services: any;
    interfaceTraffic: InterfaceTraffic | null;
    systemResources?: SystemResources;
    systemDisk?: SystemDisk;
  }

  let isFirstRun: boolean | null = null;
  let isLoading = true;
  let expandedServices = false;
  let expandedGateway: string | null = null;
  let dashboardData: DashboardData = {
    gatewayStatus: null,
    services: null,
    interfaceTraffic: null,
  };

  let pollInterval: number;
  let progressInterval: number;
  let progress = 0;
  const UPDATE_INTERVAL = 5000; // 5 seconds

  // Reactive variables for memory and disk usage
  $: memoryUsagePercent = dashboardData.systemResources
    ? Math.round(
        (dashboardData.systemResources.memory.used / 
         parseInt(dashboardData.systemResources.memory.total)) * 100
      )
    : 0;

  $: mainDisk = dashboardData.systemDisk?.devices
    ? dashboardData.systemDisk.devices.find((d) => d.mountpoint === "/")
    : null;

  $: sortedInterfaces = dashboardData.interfaceTraffic
    ? Object.entries(dashboardData.interfaceTraffic.interfaces).sort(
        ([, a], [, b]) => a.name.localeCompare(b.name)
      )
    : [];

  function toggleServicesExpansion() {
    expandedServices = !expandedServices;
  }

  function formatBytes(bytes: string): string {
    const parsedBytes = parseInt(bytes);
    if (isNaN(parsedBytes)) return "0 B";
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    if (parsedBytes === 0) return "0 B";
    const i = Math.floor(Math.log(parsedBytes) / Math.log(1024));
    return Math.round(parsedBytes / Math.pow(1024, i)) + " " + sizes[i];
  }

  async function loadDashboardData() {
    try {
      const [gatewayStatus, services, interfaceTraffic, systemResources, systemDisk] = 
        await Promise.all([
          invoke<any>("get_gateway_status"),
          invoke<any>("get_services"),
          invoke<InterfaceTraffic>("get_interface_traffic"),
          invoke<SystemResources>("get_system_resources"),
          invoke<SystemDisk>("get_system_disk")
        ]);

      dashboardData = { 
        gatewayStatus, 
        services, 
        interfaceTraffic,
        systemResources,
        systemDisk
      };
    } catch (error) {
      console.error("Failed to fetch dashboard data:", error);
      toasts.error("Failed to load dashboard data. Please try again.");
    }
  }

  async function restartService(serviceId: string) {
    try {
      await invoke("restart_service", { serviceId });
      toasts.success(`Service ${serviceId} restarted successfully`);
      await loadDashboardData();
    } catch (error) {
      console.error(`Failed to restart service ${serviceId}:`, error);
      toasts.error(`Failed to restart service ${serviceId}. Please try again.`);
    }
  }

  function startPolling() {
    pollInterval = window.setInterval(async () => {
      try {
        const [interfaceTraffic, systemResources, systemDisk] = await Promise.all([
          invoke<InterfaceTraffic>("get_interface_traffic"),
          invoke<SystemResources>("get_system_resources"),
          invoke<SystemDisk>("get_system_disk")
        ]);

        dashboardData.interfaceTraffic = interfaceTraffic;
        dashboardData.systemResources = systemResources;
        dashboardData.systemDisk = systemDisk;

        progress = 0;
      } catch (error) {
        console.error("Failed to fetch data:", error);
      }
    }, UPDATE_INTERVAL);

    progressInterval = window.setInterval(() => {
      progress += 100 / (UPDATE_INTERVAL / 100);
      if (progress >= 100) progress = 0;
    }, 100);
  }

  function toggleGatewayExpansion(gatewayName: string) {
    expandedGateway = expandedGateway === gatewayName ? null : gatewayName;
  }

  onMount(async () => {
    try {
      isFirstRun = await invoke<boolean>("check_first_run");
      if (!isFirstRun) {
        authStore.setConfigured(true);
        if ($authStore.isLoggedIn) {
          await loadDashboardData();
          startPolling();
        }
      }
    } catch (error) {
      console.error("Failed to check if first run:", error);
      toasts.error("Failed to initialize application. Please try again.");
    } finally {
      isLoading = false;
    }
  });

  onDestroy(() => {
    if (pollInterval) window.clearInterval(pollInterval);
    if (progressInterval) window.clearInterval(progressInterval);
  });

  async function handleInitialSetup(event: CustomEvent<{
    profileName: string;
    apiKey: string;
    apiSecret: string;
    apiUrl: string;
    port: number;
    pin: string;
  }>) {
    const { profileName, apiKey, apiSecret, apiUrl, port, pin } = event.detail;
    try {
      await invoke("save_initial_config", {
        config: {
          profile_name: profileName,
          api_key: apiKey,
          api_secret: apiSecret,
          api_url: apiUrl,
          port,
          pin,
        },
      });

      isFirstRun = false;
      authStore.setConfigured(true);
      toasts.success("Configuration saved successfully!");
      setTimeout(() => goto("/"), 100);
    } catch (error) {
      console.error("Failed to save configuration:", error);
      toasts.error(`Failed to save configuration: ${error}`);
    }
  }

  async function handleLogin() {
    authStore.login();
    await loadDashboardData();
    startPolling();
  }
</script>

{#if isLoading}
  <div class="min-h-screen flex items-center justify-center bg-base-200">
    <div class="text-center">
      <span class="loading loading-spinner loading-lg"></span>
      <p class="mt-4 text-base-content">Loading...</p>
    </div>
  </div>
{:else if isFirstRun}
  <div class="min-h-screen bg-base-200 p-4">
    <div class="max-w-md mx-auto space-y-8">
      <div class="text-center">
        <h1 class="text-3xl font-bold mb-2">Welcome to OPNsense Manager</h1>
        <p class="text-base-content">
          Please enter your API information and create a PIN to get started.
        </p>
      </div>
      <div class="card bg-base-100 shadow-xl">
        <InitialSetupForm
          on:submit={handleInitialSetup}
        />
      </div>
    </div>
  </div>
{:else if !$authStore.isLoggedIn}
  <Login on:login={handleLogin} />
{:else}
  <AppLayout>
    <div class="p-6 max-w-7xl mx-auto">
      <h2 class="text-2xl font-bold mb-6">Dashboard</h2>

      {#if dashboardData.gatewayStatus && dashboardData.services && dashboardData.interfaceTraffic}
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <SystemResourcesColumn 
            systemResources={dashboardData.systemResources} 
            mainDisk={mainDisk}
            services={dashboardData.services.rows}
            {expandedServices}
            {toggleServicesExpansion}
            {restartService}
          />

          <NetworkInformationColumn 
            gatewayStatus={dashboardData.gatewayStatus.items}
            {expandedGateway}
            {toggleGatewayExpansion}
            {sortedInterfaces}
            {formatBytes}
          />
        </div>
      {:else}
        <div class="flex justify-center items-center h-64">
          <span class="loading loading-spinner loading-lg"></span>
        </div>
      {/if}
    </div>
  </AppLayout>
{/if}

<style>
  @keyframes rotate {
    100% {
      transform: rotate(360deg);
    }
  }

  .progress-ring {
    animation: rotate 5s linear infinite;
  }
</style>