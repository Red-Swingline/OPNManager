<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import Login from "$lib/components/forms/Login.svelte";
  import AppLayout from "./AppLayout.svelte";
  import InitialSetupForm from "$lib/components/forms/InitialSetupForm.svelte";
  import DashboardConfig from "$lib/components/dashboard/DashboardConfig.svelte";
  import InterfaceTrafficGraph from "$lib/components/dashboard/InterfaceTrafficGraph.svelte";
  import CpuTemperatureCard from "$lib/components/dashboard/CpuTemperatureCard.svelte";
  import { WolWidget } from "$lib/components/dashboard";
  import { toasts } from "$lib/stores/toastStore";
  import { authStore } from "$lib/stores/authStore";
  import { dashboardStore } from "$lib/stores/dashboardStore";
  import { cleanupDashboardResources } from "$lib/utils/dashboardCleanup";
  import {
    mdiRestart,
    mdiChevronDown,
    mdiChevronUp,
    mdiRouter,
    mdiServerNetwork,
    mdiEthernet,
    mdiServer,
    mdiHarddisk,
    mdiViewDashboardEditOutline,
    mdiClockOutline,
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

  interface TemperatureSensor {
    device: string;
    device_seq: string;
    temperature: string;
    sensor_type: string;
    sensor_type_translated: string;
  }

  interface SystemTemperature {
    sensors: TemperatureSensor[];
  }

  interface DashboardData {
    gatewayStatus: any;
    services: any;
    interfaceTraffic: InterfaceTraffic | null;
    systemResources?: SystemResources;
    systemDisk?: SystemDisk;
    systemTime?: any;
    systemTemperature?: SystemTemperature;
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
  const UPDATE_INTERVAL = 5000;

  $: memoryUsagePercent = dashboardData.systemResources
    ? Math.round(
        (dashboardData.systemResources.memory.used /
          parseInt(dashboardData.systemResources.memory.total)) *
          100,
      )
    : 0;

  $: mainDisk = dashboardData.systemDisk?.devices
    ? dashboardData.systemDisk.devices.find((d) => d.mountpoint === "/")
    : null;

  $: sortedInterfaces = dashboardData.interfaceTraffic
    ? Object.entries(dashboardData.interfaceTraffic.interfaces).sort(
        ([, a], [, b]) => a.name.localeCompare(b.name),
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
      const [
        gatewayStatus,
        services,
        interfaceTraffic,
        systemResources,
        systemDisk,
        systemTime,
        systemTemperature,
      ] = await Promise.all([
        invoke<any>("get_gateway_status"),
        invoke<any>("get_services"),
        invoke<InterfaceTraffic>("get_interface_traffic"),
        invoke<SystemResources>("get_system_resources"),
        invoke<SystemDisk>("get_system_disk"),
        invoke<any>("get_system_time"),
        invoke<SystemTemperature>("get_system_temperature").catch(() => ({ sensors: [] })),
      ]);

      dashboardData = {
        gatewayStatus,
        services,
        interfaceTraffic,
        systemResources,
        systemDisk,
        systemTime,
        systemTemperature,
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
        const [interfaceTraffic, systemResources, systemDisk, systemTime, systemTemperature] =
          await Promise.all([
            invoke<InterfaceTraffic>("get_interface_traffic"),
            invoke<SystemResources>("get_system_resources"),
            invoke<SystemDisk>("get_system_disk"),
            invoke<any>("get_system_time"),
            invoke<SystemTemperature>("get_system_temperature").catch(() => ({ sensors: [] })),
          ]);

        dashboardData.interfaceTraffic = interfaceTraffic;
        dashboardData.systemResources = systemResources;
        dashboardData.systemDisk = systemDisk;
        dashboardData.systemTime = systemTime;
        dashboardData.systemTemperature = systemTemperature;

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

  function toggleDashboardEditMode() {
    dashboardStore.toggleEditMode();
  }

  onMount(async () => {
    try {
      isFirstRun = await invoke<boolean>("check_first_run");
      if (!isFirstRun) {
        authStore.setConfigured(true);
        if ($authStore.isLoggedIn) {
          await dashboardStore.loadPreferences();
          await invoke("clear_traffic_cache");
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
    console.log("Dashboard page destroyed - cleaning up resources");
    if (pollInterval) window.clearInterval(pollInterval);
    if (progressInterval) window.clearInterval(progressInterval);
    
    // Clean up dashboard resources when navigating away from the dashboard
    cleanupDashboardResources().catch(error => {
      console.error("Failed to clean up dashboard resources:", error);
    });
  });

  async function handleInitialSetup(
    event: CustomEvent<{
      profileName: string;
      apiKey: string;
      apiSecret: string;
      apiUrl: string;
      port: number;
      pin: string;
    }>,
  ) {
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
    await dashboardStore.loadPreferences();
    await loadDashboardData();
    startPolling();
  }

  function isWidgetVisible(key: string): boolean {
    if (!$dashboardStore.isLoaded) return true;
    const widget = $dashboardStore.widgets.find((w) => w.widget_key === key);
    return widget ? widget.visible : true;
  }

  function getResourceWidgetOrder(): string[] {
    if (!$dashboardStore.isLoaded) {
      return ["uptime", "memory", "cpu_temp", "disk", "services"];
    }

    return $dashboardStore.widgets
      .filter((w) =>
        ["uptime", "memory", "cpu_temp", "disk", "services"].includes(w.widget_key),
      )
      .sort((a, b) => a.position - b.position)
      .map((w) => w.widget_key);
  }

  function getNetworkWidgetOrder(): string[] {
    if (!$dashboardStore.isLoaded) {
      return ["traffic_graph", "gateways", "interfaces", "wol"];
    }

    return $dashboardStore.widgets
      .filter((w) =>
        ["traffic_graph", "gateways", "interfaces", "wol"].includes(w.widget_key),
      )
      .sort((a, b) => a.position - b.position)
      .map((w) => w.widget_key);
  }

  function getAllWidgetsInOrder(): string[] {
    if (!$dashboardStore.isLoaded) {
      return [
        "uptime",
        "memory",
        "cpu_temp",
        "disk",
        "services", 
        "traffic_graph",
        "gateways",
        "interfaces",
        "wol",
      ];
    }

    return $dashboardStore.widgets
      .sort((a, b) => a.position - b.position)
      .map((w) => w.widget_key);
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
        <h1 class="text-3xl font-bold mb-2">Welcome to OPNManager</h1>
      </div>
      <div class="card bg-base-100 shadow-xl">
        <InitialSetupForm on:submit={handleInitialSetup} />
      </div>
    </div>
  </div>
{:else if !$authStore.isLoggedIn}
  <Login on:login={handleLogin} />
{:else}
  <AppLayout>
    <div class="p-6 max-w-7xl mx-auto">
      <div class="flex justify-between items-center mb-6">
        <h2 class="text-2xl font-bold">Dashboard</h2>

        <!-- Dashboard Edit Button -->
        <button
          class="btn btn-outline btn-sm gap-2"
          on:click={toggleDashboardEditMode}
          class:btn-primary={$dashboardStore.isEditing}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiViewDashboardEditOutline} />
          </svg>
          {$dashboardStore.isEditing ? "Done Editing" : "Edit Dashboard"}
        </button>
      </div>

      <!-- Dashboard Customization UI -->
      {#if $dashboardStore.isEditing}
        <DashboardConfig />
      {/if}

      {#if dashboardData.gatewayStatus && dashboardData.services && dashboardData.interfaceTraffic}
        <!-- Mobile View: Single Column with Widgets in Position Order -->
        <div class="lg:hidden flex flex-col space-y-6">
          {#each getAllWidgetsInOrder() as widgetKey}
            {#if isWidgetVisible(widgetKey)}
              {#if widgetKey === "uptime" && dashboardData.systemTime}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <h3 class="card-title text-lg flex items-center gap-2">
                      <svg class="w-5 h-5 text-primary" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiClockOutline} />
                      </svg>
                      System Status
                    </h3>

                    <div class="divider my-2"></div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div class="space-y-3">
                        <div>
                          <h4 class="text-sm font-medium opacity-70">Uptime</h4>
                          <p class="text-lg font-mono">
                            {dashboardData.systemTime.uptime}
                          </p>
                        </div>

                        <div>
                          <h4 class="text-sm font-medium opacity-70">
                            Load average
                          </h4>
                          <p class="text-lg font-mono">
                            {dashboardData.systemTime.loadavg}
                          </p>
                        </div>
                      </div>

                      <div class="space-y-3">
                        <div>
                          <h4 class="text-sm font-medium opacity-70">
                            Current date/time
                          </h4>
                          <p class="text-lg font-mono">
                            {dashboardData.systemTime.datetime}
                          </p>
                        </div>

                        <div>
                          <h4 class="text-sm font-medium opacity-70">
                            Last configuration change
                          </h4>
                          <p class="text-lg font-mono">
                            {dashboardData.systemTime.config}
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              {/if}

              {#if widgetKey === "memory" && dashboardData.systemResources}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <h3 class="card-title text-lg flex items-center gap-2">
                      <svg class="w-5 h-5 text-indigo-500" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiServer} />
                      </svg>
                      Memory Usage
                    </h3>
                    <div class="divider my-2"></div>
                    <div class="flex items-center justify-between">
                      <div
                        class="radial-progress text-indigo-500"
                        style="--value:{memoryUsagePercent}; --size:4rem; --thickness: 8px;"
                        role="progressbar"
                      >
                        {memoryUsagePercent}%
                      </div>
                      <div>
                        <p class="text-sm">
                          <span class="font-semibold">Used:</span>
                          {dashboardData.systemResources.memory.used_frmt}
                        </p>
                        <p class="text-sm">
                          <span class="font-semibold">Total:</span>
                          {dashboardData.systemResources.memory.total_frmt}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              {/if}
              
              {#if widgetKey === "cpu_temp" && dashboardData.systemTemperature}
                <CpuTemperatureCard temperatureData={dashboardData.systemTemperature} />
              {/if}

              {#if widgetKey === "disk" && mainDisk}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <h3 class="card-title text-lg flex items-center gap-2">
                      <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiHarddisk} />
                      </svg>
                      Disk Usage
                    </h3>
                    <div class="divider my-2"></div>
                    <div class="flex items-center justify-between">
                      <div
                        class="radial-progress text-accent"
                        style="--value:{mainDisk.used_pct}; --size:4rem; --thickness: 8px;"
                        role="progressbar"
                      >
                        {mainDisk.used_pct}%
                      </div>
                      <div>
                        <p class="text-sm">
                          <span class="font-semibold">Used:</span>
                          {mainDisk.used}
                        </p>
                        <p class="text-sm">
                          <span class="font-semibold">Available:</span>
                          {mainDisk.available}
                        </p>
                        <p class="text-sm">
                          <span class="font-semibold">Mountpoint:</span>
                          {mainDisk.mountpoint}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              {/if}

              {#if widgetKey === "services" && dashboardData.services}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex justify-between items-center">
                      <h3 class="card-title text-lg flex items-center gap-2">
                        <svg class="w-5 h-5 text-secondary" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiServerNetwork} />
                        </svg>
                        Services
                        <span class="badge badge-sm ml-2">
                          {dashboardData.services.rows.length}
                        </span>
                      </h3>
                      <button
                        class="btn btn-ghost btn-sm"
                        on:click={toggleServicesExpansion}
                      >
                        <svg class="w-5 h-5" viewBox="0 0 24 24">
                          <path
                            fill="currentColor"
                            d={expandedServices ? mdiChevronUp : mdiChevronDown}
                          />
                        </svg>
                      </button>
                    </div>

                    {#if expandedServices}
                      <div class="divider my-2"></div>
                      <div class="space-y-3">
                        {#each dashboardData.services.rows as service}
                          <div
                            class="border rounded-lg p-3
                            {service.running
                              ? 'bg-success/10 border-success/30'
                              : 'bg-error/10 border-error/30'}"
                          >
                            <div
                              class="grid grid-cols-[1fr,auto] gap-2 items-center"
                            >
                              <div class="min-w-0 overflow-hidden">
                                <div
                                  class="font-medium truncate"
                                  title={service.name}
                                >
                                  {service.name}
                                </div>
                                <div
                                  class="text-sm opacity-70 truncate"
                                  title={service.description}
                                >
                                  {service.description}
                                </div>
                              </div>
                              <div class="flex items-center space-x-2 shrink-0">
                                <span
                                  class="badge whitespace-nowrap {service.running
                                    ? 'badge-success'
                                    : 'badge-error'}"
                                >
                                  {service.running ? "Running" : "Stopped"}
                                </span>
                                <button
                                  class="btn btn-ghost btn-sm shrink-0"
                                  on:click={() => restartService(service.id)}
                                  title="Restart Service"
                                  aria-label="Restart {service.name} Service"
                                >
                                  <svg class="w-5 h-5" viewBox="0 0 24 24">
                                    <path fill="currentColor" d={mdiRestart} />
                                  </svg>
                                </button>
                              </div>
                            </div>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}

              {#if widgetKey === "traffic_graph"}
                <InterfaceTrafficGraph />
              {/if}

              <!-- Replace the gateways widget section in the mobile view with this code -->

              {#if widgetKey === "gateways" && dashboardData.gatewayStatus}
                <div class="card bg-base-100 shadow-xl w-full">
                  <div class="card-body">
                    <h3 class="card-title text-lg flex items-center gap-2">
                      <svg class="w-5 h-5 text-primary" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiRouter} />
                      </svg>
                      Gateways
                    </h3>
                    <div class="divider my-2"></div>
                    <div class="space-y-3">
                      {#each dashboardData.gatewayStatus.items as gateway}
                        <div class="border rounded-lg p-3">
                          <div
                            class="grid grid-cols-[1fr,auto] gap-2 items-center cursor-pointer"
                            on:click={() =>
                              toggleGatewayExpansion(gateway.name)}
                          >
                            <!-- Gateway info section - has min-width-0 to enable truncation -->
                            <div class="min-w-0 overflow-hidden">
                              <div class="font-medium flex items-center">
                                <span
                                  class="inline-block w-2 h-2 rounded-full mr-2 flex-shrink-0 {gateway.status_translated ===
                                  'Online'
                                    ? 'bg-success'
                                    : 'bg-error'}"
                                ></span>
                                <span class="truncate" title={gateway.name}>
                                  {gateway.name}
                                </span>
                              </div>
                              <div
                                class="text-sm opacity-70 truncate"
                                title={gateway.address}
                              >
                                {gateway.address}
                              </div>
                            </div>

                            <!-- Status section that won't shrink -->
                            <div class="flex items-center gap-1 flex-shrink-0">
                              <span
                                class="badge badge-sm whitespace-nowrap {gateway.status_translated ===
                                'Online'
                                  ? 'badge-success'
                                  : 'badge-error'}"
                              >
                                {gateway.status_translated}
                              </span>
                              <svg
                                class="w-5 h-5 flex-shrink-0"
                                viewBox="0 0 24 24"
                              >
                                <path
                                  fill="currentColor"
                                  d={expandedGateway === gateway.name
                                    ? mdiChevronUp
                                    : mdiChevronDown}
                                />
                              </svg>
                            </div>
                          </div>
                          {#if expandedGateway === gateway.name}
                            <div
                              class="mt-3 text-sm grid grid-cols-3 gap-2 pt-2 border-t border-base-300"
                            >
                              <div>
                                <span class="font-medium">RTT:</span>
                                <span class="ml-1"
                                  >{gateway.delay === "~"
                                    ? "-"
                                    : gateway.delay}</span
                                >
                              </div>
                              <div>
                                <span class="font-medium">RTTd:</span>
                                <span class="ml-1"
                                  >{gateway.stddev === "~"
                                    ? "-"
                                    : gateway.stddev}</span
                                >
                              </div>
                              <div>
                                <span class="font-medium">Loss:</span>
                                <span class="ml-1"
                                  >{gateway.loss === "~"
                                    ? "-"
                                    : gateway.loss}</span
                                >
                              </div>
                            </div>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                </div>
              {/if}

              {#if widgetKey === "interfaces" && sortedInterfaces.length > 0}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <h3 class="card-title text-lg flex items-center gap-2">
                      <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiEthernet} />
                      </svg>
                      Interface Traffic
                    </h3>
                    <div class="divider my-2"></div>
                    <div class="overflow-x-auto">
                      <table class="table table-zebra w-full">
                        <thead>
                          <tr>
                            <th>Interface</th>
                            <th>Received</th>
                            <th>Transmitted</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each sortedInterfaces as [key, interfaceData]}
                            <tr>
                              <td class="font-medium">
                                <div class="flex items-center gap-2">
                                  <div
                                    class="w-2 h-2 rounded-full bg-accent"
                                  ></div>
                                  {interfaceData.name}
                                </div>
                                <div class="text-xs opacity-50">
                                  {interfaceData.device}
                                </div>
                              </td>
                              <td>
                                <span class="font-mono"
                                  >{formatBytes(
                                    interfaceData["bytes received"],
                                  )}</span
                                >
                              </td>
                              <td>
                                <span class="font-mono"
                                  >{formatBytes(
                                    interfaceData["bytes transmitted"],
                                  )}</span
                                >
                              </td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  </div>
                </div>
              {/if}
              
              {#if widgetKey === "wol"}
                <WolWidget />
              {/if}

            {/if}
          {/each}
        </div>

        <!-- Desktop View: Two Column Layout -->
        <div class="hidden lg:grid lg:grid-cols-2 lg:gap-6">
          <!-- Network Column -->
          <div class="space-y-6 order-first">
            {#each getNetworkWidgetOrder() as widgetKey}
              {#if isWidgetVisible(widgetKey)}
                {#if widgetKey === "traffic_graph"}
                  <InterfaceTrafficGraph />
                {/if}

                {#if widgetKey === "gateways" && dashboardData.gatewayStatus}
                  <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                      <h3 class="card-title text-lg flex items-center gap-2">
                        <svg class="w-5 h-5 text-primary" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiRouter} />
                        </svg>
                        Gateways
                      </h3>
                      <div class="divider my-2"></div>
                      <div class="space-y-3">
                        {#each dashboardData.gatewayStatus.items as gateway}
                          <div class="border rounded-lg p-3">
                            <div
                              class="flex justify-between items-center cursor-pointer"
                              on:click={() =>
                                toggleGatewayExpansion(gateway.name)}
                            >
                              <div>
                                <div class="font-medium flex items-center">
                                  <span
                                    class="inline-block w-2 h-2 rounded-full mr-2 {gateway.status_translated ===
                                    'Online'
                                      ? 'bg-success'
                                      : 'bg-error'}"
                                  ></span>
                                  {gateway.name}
                                </div>
                                <div class="text-sm opacity-70">
                                  {gateway.address}
                                </div>
                              </div>
                              <div class="flex items-center gap-2">
                                <span
                                  class="badge badge-sm {gateway.status_translated ===
                                  'Online'
                                    ? 'badge-success'
                                    : 'badge-error'}"
                                >
                                  {gateway.status_translated}
                                </span>
                                <svg class="w-5 h-5" viewBox="0 0 24 24">
                                  <path
                                    fill="currentColor"
                                    d={expandedGateway === gateway.name
                                      ? mdiChevronUp
                                      : mdiChevronDown}
                                  />
                                </svg>
                              </div>
                            </div>
                            {#if expandedGateway === gateway.name}
                              <div
                                class="mt-3 text-sm grid grid-cols-3 gap-2 pt-2 border-t border-base-300"
                              >
                                <div>
                                  <span class="font-medium">RTT:</span>
                                  <span class="ml-1"
                                    >{gateway.delay === "~"
                                      ? "-"
                                      : gateway.delay}</span
                                  >
                                </div>
                                <div>
                                  <span class="font-medium">RTTd:</span>
                                  <span class="ml-1"
                                    >{gateway.stddev === "~"
                                      ? "-"
                                      : gateway.stddev}</span
                                  >
                                </div>
                                <div>
                                  <span class="font-medium">Loss:</span>
                                  <span class="ml-1"
                                    >{gateway.loss === "~"
                                      ? "-"
                                      : gateway.loss}</span
                                  >
                                </div>
                              </div>
                            {/if}
                          </div>
                        {/each}
                      </div>
                    </div>
                  </div>
                {/if}

                {#if widgetKey === "interfaces" && sortedInterfaces.length > 0}
                  <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                      <h3 class="card-title text-lg flex items-center gap-2">
                        <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiEthernet} />
                        </svg>
                        Interface Traffic
                      </h3>
                      <div class="divider my-2"></div>
                      <div class="overflow-x-auto">
                        <table class="table table-zebra w-full">
                          <thead>
                            <tr>
                              <th>Interface</th>
                              <th>Received</th>
                              <th>Transmitted</th>
                            </tr>
                          </thead>
                          <tbody>
                            {#each sortedInterfaces as [key, interfaceData]}
                              <tr>
                                <td class="font-medium">
                                  <div class="flex items-center gap-2">
                                    <div
                                      class="w-2 h-2 rounded-full bg-accent"
                                    ></div>
                                    {interfaceData.name}
                                  </div>
                                  <div class="text-xs opacity-50">
                                    {interfaceData.device}
                                  </div>
                                </td>
                                <td>
                                  <span class="font-mono"
                                    >{formatBytes(
                                      interfaceData["bytes received"],
                                    )}</span
                                  >
                                </td>
                                <td>
                                  <span class="font-mono"
                                    >{formatBytes(
                                      interfaceData["bytes transmitted"],
                                    )}</span
                                  >
                                </td>
                              </tr>
                            {/each}
                          </tbody>
                        </table>
                      </div>
                    </div>
                  </div>
                {/if}
                
                {#if widgetKey === "wol"}
                  <WolWidget />
                {/if}

              {/if}
            {/each}
          </div>

          <!-- System Resources Column -->
          <div class="space-y-6 order-last">
            {#each getResourceWidgetOrder() as widgetKey}
              {#if isWidgetVisible(widgetKey)}
                {#if widgetKey === "uptime" && dashboardData.systemTime}
                  <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                      <h3 class="card-title text-lg flex items-center gap-2">
                        <svg class="w-5 h-5 text-primary" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiClockOutline} />
                        </svg>
                        System Status
                      </h3>

                      <div class="divider my-2"></div>

                      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div class="space-y-3">
                          <div>
                            <h4 class="text-sm font-medium opacity-70">
                              Uptime
                            </h4>
                            <p class="text-lg font-mono">
                              {dashboardData.systemTime.uptime}
                            </p>
                          </div>

                          <div>
                            <h4 class="text-sm font-medium opacity-70">
                              Load average
                            </h4>
                            <p class="text-lg font-mono">
                              {dashboardData.systemTime.loadavg}
                            </p>
                          </div>
                        </div>

                        <div class="space-y-3">
                          <div>
                            <h4 class="text-sm font-medium opacity-70">
                              Current date/time
                            </h4>
                            <p class="text-lg font-mono">
                              {dashboardData.systemTime.datetime}
                            </p>
                          </div>

                          <div>
                            <h4 class="text-sm font-medium opacity-70">
                              Last configuration change
                            </h4>
                            <p class="text-lg font-mono">
                              {dashboardData.systemTime.config}
                            </p>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                {/if}

                {#if widgetKey === "memory" && dashboardData.systemResources}
                  <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                      <h3 class="card-title text-lg flex items-center gap-2">
                        <svg
                          class="w-5 h-5 text-indigo-500"
                          viewBox="0 0 24 24"
                        >
                          <path fill="currentColor" d={mdiServer} />
                        </svg>
                        Memory Usage
                      </h3>
                      <div class="divider my-2"></div>
                      <div class="flex items-center justify-between">
                        <div
                          class="radial-progress text-indigo-500"
                          style="--value:{memoryUsagePercent}; --size:4rem; --thickness: 8px;"
                          role="progressbar"
                        >
                          {memoryUsagePercent}%
                        </div>
                        <div>
                          <p class="text-sm">
                            <span class="font-semibold">Used:</span>
                            {dashboardData.systemResources.memory.used_frmt}
                          </p>
                          <p class="text-sm">
                            <span class="font-semibold">Total:</span>
                            {dashboardData.systemResources.memory.total_frmt}
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                {/if}
                
                {#if widgetKey === "cpu_temp" && dashboardData.systemTemperature}
                  <CpuTemperatureCard temperatureData={dashboardData.systemTemperature} />
                {/if}

                {#if widgetKey === "disk" && mainDisk}
                  <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                      <h3 class="card-title text-lg flex items-center gap-2">
                        <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24">
                          <path fill="currentColor" d={mdiHarddisk} />
                        </svg>
                        Disk Usage
                      </h3>
                      <div class="divider my-2"></div>
                      <div class="flex items-center justify-between">
                        <div
                          class="radial-progress text-accent"
                          style="--value:{mainDisk.used_pct}; --size:4rem; --thickness: 8px;"
                          role="progressbar"
                        >
                          {mainDisk.used_pct}%
                        </div>
                        <div>
                          <p class="text-sm">
                            <span class="font-semibold">Used:</span>
                            {mainDisk.used}
                          </p>
                          <p class="text-sm">
                            <span class="font-semibold">Available:</span>
                            {mainDisk.available}
                          </p>
                          <p class="text-sm">
                            <span class="font-semibold">Mountpoint:</span>
                            {mainDisk.mountpoint}
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                {/if}

                {#if widgetKey === "services" && dashboardData.services}
                  <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                      <div class="flex justify-between items-center">
                        <h3 class="card-title text-lg flex items-center gap-2">
                          <svg
                            class="w-5 h-5 text-secondary"
                            viewBox="0 0 24 24"
                          >
                            <path fill="currentColor" d={mdiServerNetwork} />
                          </svg>
                          Services
                          <span class="badge badge-sm ml-2">
                            {dashboardData.services.rows.length}
                          </span>
                        </h3>
                        <button
                          class="btn btn-ghost btn-sm"
                          on:click={toggleServicesExpansion}
                        >
                          <svg class="w-5 h-5" viewBox="0 0 24 24">
                            <path
                              fill="currentColor"
                              d={expandedServices
                                ? mdiChevronUp
                                : mdiChevronDown}
                            />
                          </svg>
                        </button>
                      </div>

                      {#if expandedServices}
                        <div class="divider my-2"></div>
                        <div class="space-y-3">
                          {#each dashboardData.services.rows as service}
                            <div
                              class="border rounded-lg p-3
                              {service.running
                                ? 'bg-success/10 border-success/30'
                                : 'bg-error/10 border-error/30'}"
                            >
                              <div
                                class="grid grid-cols-[1fr,auto] gap-2 items-center"
                              >
                                <div class="min-w-0 overflow-hidden">
                                  <div
                                    class="font-medium truncate"
                                    title={service.name}
                                  >
                                    {service.name}
                                  </div>
                                  <div
                                    class="text-sm opacity-70 truncate"
                                    title={service.description}
                                  >
                                    {service.description}
                                  </div>
                                </div>
                                <div
                                  class="flex items-center space-x-2 shrink-0"
                                >
                                  <span
                                    class="badge whitespace-nowrap {service.running
                                      ? 'badge-success'
                                      : 'badge-error'}"
                                  >
                                    {service.running ? "Running" : "Stopped"}
                                  </span>
                                  <button
                                    class="btn btn-ghost btn-sm shrink-0"
                                    on:click={() => restartService(service.id)}
                                    title="Restart Service"
                                    aria-label="Restart {service.name} Service"
                                  >
                                    <svg class="w-5 h-5" viewBox="0 0 24 24">
                                      <path
                                        fill="currentColor"
                                        d={mdiRestart}
                                      />
                                    </svg>
                                  </button>
                                </div>
                              </div>
                            </div>
                          {/each}
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}
              {/if}
            {/each}
          </div>
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

  @media (max-width: 640px) {
    /* Mobile style improvements */
    .card {
      width: 100% !important;
      max-width: 100% !important;
      margin-left: auto;
      margin-right: auto;
    }

    .card-body {
      padding: 1rem;
    }

    /* Ensure proper truncation */
    .truncate {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      display: block;
    }

    /* Improve spacing for gateway cards */
    .badge {
      padding-left: 0.5rem;
      padding-right: 0.5rem;
    }

    /* Reduce padding in portrait mode */
    .p-6 {
      padding: 1rem;
    }

    /* Service and gateway cards improvements */
    .grid-cols-\[1fr\,auto\] {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
    }
  }

  /* iPhone SE and smaller screens */
  @media (max-width: 375px) {
    .card-body {
      padding: 0.75rem;
    }

    .badge {
      font-size: 0.65rem;
    }

    .p-6 {
      padding: 0.75rem;
    }
  }
</style>
