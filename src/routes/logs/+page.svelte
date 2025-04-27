<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { mdiPlay, mdiPause, mdiInformation, mdiFilter, mdiTextBoxSearch, mdiRefresh } from "@mdi/js";
  import AppLayout from "../AppLayout.svelte";

  interface FirewallLog {
    rulenr?: string;
    interface?: string;
    src?: string;
    dst?: string;
    srcport?: string;
    dstport?: string;
    protoname?: string;
    action?: string;
    __timestamp__?: string;
    label?: string;
    digest?: string;
  }

  interface LogFilters {
    action: string[];
    interface_name: string[];
    dir: string[];
  }

  interface InterfaceNames {
    [key: string]: string;
  }

  let logs: FirewallLog[] = [];
  let rawLogs: FirewallLog[] = []; // Store unfiltered logs from backend
  let filters: LogFilters | null = null;
  let interfaceNames: InterfaceNames | null = null;
  let isPlaying = true;
  let selectedLog: FirewallLog | null = null;
  let showModal = false;
  let showFilters = false;
  let showRawJson = false;
  let isLoading = true;
  let autoScroll = true;
  let logContainer: HTMLElement;
  
  // Web worker for log processing
  let logWorker: Worker;
  let isWorkerProcessing = false;

  let selectedAction = "";
  let selectedInterface = "";
  let selectedDirection = "";
  const limit = 500; // Reduced from 1000 for better performance

  let unlisten: () => void;

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString();
  }

  async function fetchLogFilters() {
    try {
      filters = await invoke<LogFilters>("get_log_filters");
    } catch (error) {
      console.error("Failed to fetch log filters:", error);
    }
  }

  async function fetchInterfaceNames() {
    try {
      interfaceNames = await invoke<InterfaceNames>("get_interface_names");
    } catch (error) {
      console.error("Failed to fetch interface names:", error);
    }
  }

  async function fetchLogs() {
    isLoading = true;
    try {
      rawLogs = await invoke<FirewallLog[]>("get_firewall_logs");
      console.log(`Fetched ${rawLogs.length} logs from backend`);
      
      // Process logs using web worker if available
      if (logWorker && !isWorkerProcessing) {
        isWorkerProcessing = true;
        try {
          logWorker.postMessage({
            type: 'processlogs',
            logs: rawLogs,
            currentLogs: [],
            filters: {
              action: selectedAction,
              interface: selectedInterface,
              direction: selectedDirection
            },
            limit
          });
          console.log("Sent logs to worker for processing");
        } catch (workerError) {
          console.error("Error sending to worker:", workerError);
          // Fallback if worker fails
          logs = rawLogs;
          isWorkerProcessing = false;
          isLoading = false;
        }
      } else {
        // Fallback if worker isn't ready
        console.log("Using main thread processing (worker busy or not available)");
        logs = rawLogs;
        isLoading = false;
      }
    } catch (error) {
      console.error("Failed to fetch logs:", error);
      isLoading = false;
    } finally {
      if (!logWorker) {
        isLoading = false;
      }
    }
  }

  async function togglePlay() {
    console.log(`Toggle play from ${isPlaying} to ${!isPlaying}`);
    
    // Set the state first to reflect the change immediately in UI
    const wasPlaying = isPlaying;
    isPlaying = !wasPlaying;
    autoScroll = isPlaying; // Link auto-scroll state to play/pause
    
    try {
      if (isPlaying) {
        console.log("Starting polling...");
        await startPolling();
        console.log("Polling started");
        
        // Scroll to bottom when enabling auto-scroll
        setTimeout(() => {
          if (logContainer) {
            logContainer.scrollTop = logContainer.scrollHeight;
          }
        }, 0);
      } else {
        console.log("Stopping polling...");
        await stopPolling();
        console.log("Polling stopped");
      }
    } catch (error) {
      console.error("Error during play/pause toggle:", error);
      // Revert the state if an error occurred
      isPlaying = wasPlaying;
      autoScroll = wasPlaying;
      
      // Retry the operation once after a short delay
      setTimeout(async () => {
        try {
          console.log("Retrying play/pause operation...");
          if (!wasPlaying) {  // We were trying to start polling
            await startPolling();
          } else {  // We were trying to stop polling
            await stopPolling();
          }
        } catch (retryError) {
          console.error("Retry failed:", retryError);
        }
      }, 500);
    }
  }

  async function startPolling() {
    try {
      // Make sure we clean up any existing listener first
      if (unlisten) {
        try {
          unlisten();
          unlisten = undefined;
        } catch (e) {
          console.error("Error cleaning up previous listener:", e);
        }
      }
      
      console.log("Invoking start_log_polling Tauri command");
      await invoke("start_log_polling");
      console.log("Command invoked successfully");

      console.log("Setting up event listener for logs");
      unlisten = await listen("firewall-logs-updated", (event) => {
        console.log("Received firewall-logs-updated event");
        rawLogs = event.payload as FirewallLog[];
        console.log(`Event contained ${rawLogs.length} logs`);
        
        // Process logs using web worker
        if (logWorker && !isWorkerProcessing) {
          isWorkerProcessing = true;
          try {
            logWorker.postMessage({
              type: 'processlogs',
              logs: rawLogs,
              currentLogs: [],  // We use the logs directly from the backend
              filters: {
                action: selectedAction,
                interface: selectedInterface,
                direction: selectedDirection
              },
              limit
            });
            console.log("Sent logs to worker from event handler");
          } catch (workerError) {
            console.error("Error sending to worker from event:", workerError);
            // Fallback if worker fails
            logs = rawLogs;
            isWorkerProcessing = false;
            isLoading = false;
          }
        } else {
          // Fallback if worker isn't ready
          console.log("Using main thread processing for event (worker busy or not available)");
          logs = rawLogs;
          isLoading = false;
          
          // Auto-scroll to bottom on new logs if enabled
          if (autoScroll) {
            setTimeout(() => {
              if (logContainer) {
                logContainer.scrollTop = logContainer.scrollHeight;
              }
            }, 0);
          }
        }
      });
      
      console.log("Event listener set up complete");
    } catch (error) {
      console.error("Failed to start log polling:", error);
      isLoading = false;
      isPlaying = false; // Update state if failed
    }
  }

  async function stopPolling() {
    try {
      console.log("Invoking stop_log_polling Tauri command");
      // Make sure event listener is removed first to prevent any new events from coming in
      if (unlisten) {
        console.log("Removing event listener");
        await unlisten();
        unlisten = undefined;
      }
      
      // Then stop the backend polling
      await invoke("stop_log_polling");
      console.log("Log polling stopped successfully");
    } catch (error) {
      console.error("Failed to stop log polling:", error);
      // Don't revert isPlaying state here - that's handled in togglePlay
      throw error; // Re-throw to let togglePlay handle the error
    }
  }

  function showLogDetails(log: FirewallLog) {
    selectedLog = log;
    showModal = true;
  }

  function toggleFilters() {
    showFilters = !showFilters;
  }

  async function applyFilters() {
    isLoading = true;
    try {
      await invoke("update_log_filters", {
        action: selectedAction,
        interface: selectedInterface,
        direction: selectedDirection,
        limit,
      });

      // Get fresh logs with the new filters
      rawLogs = await invoke<FirewallLog[]>("get_firewall_logs");
      
      // Use web worker to process the filtered logs
      if (logWorker && !isWorkerProcessing) {
        isWorkerProcessing = true;
        logWorker.postMessage({
          type: 'processlogs',
          logs: rawLogs,
          currentLogs: [],
          filters: {
            action: selectedAction,
            interface: selectedInterface,
            direction: selectedDirection
          },
          limit
        });
      } else {
        // Fallback if worker isn't ready
        logs = rawLogs;
        isLoading = false;
      }
      
      showFilters = false;
    } catch (error) {
      console.error("Failed to apply filters:", error);
      isLoading = false;
    }
  }

  onMount(async () => {
    // Initialize the web worker
    logWorker = new Worker(new URL('./logWorker.ts', import.meta.url), { type: 'module' });
    
    // Set up web worker message handling
    logWorker.onmessage = (event) => {
      if (event.data.type === 'processedlogs') {
        console.log(`Worker processed ${event.data.logs.length} logs in ${event.data.processingTime.toFixed(2)}ms`);
        logs = event.data.logs;
        isWorkerProcessing = false;
        isLoading = false;
        
        // Auto-scroll to bottom if enabled
        if (autoScroll && logContainer) {
          setTimeout(() => {
            logContainer.scrollTop = logContainer.scrollHeight;
          }, 0);
        }
      } else if (event.data.type === 'error') {
        console.error("Worker reported error:", event.data.error);
        // If worker fails, use raw logs directly
        logs = rawLogs;
        isWorkerProcessing = false;
        isLoading = false;
      }
    };
    
    // Handle worker errors
    logWorker.onerror = (error) => {
      console.error("Web worker error:", error);
      // If worker fails, use raw logs directly
      logs = rawLogs;
      isWorkerProcessing = false;
      isLoading = false;
    };
    
    await Promise.all([fetchLogFilters(), fetchInterfaceNames()]);
    await fetchLogs();

    if (isPlaying) {
      await startPolling();
    }
  });

  // Add a scroll handler to detect if we should auto-scroll
  function handleScroll(e: Event) {
    const target = e.target as HTMLElement;
    if (!target) return;
    
    // Check if we're near the bottom (within 100px)
    // If so, enable auto-scroll, otherwise disable it
    const atBottom = target.scrollHeight - target.scrollTop - target.clientHeight < 100;
    autoScroll = atBottom;
    
    // If the user manually scrolled away from the bottom while playing,
    // we keep polling but disable auto-scrolling
    if (!atBottom && isPlaying) {
      // We keep isPlaying true (to continue polling) but disable auto-scroll
    } else if (atBottom && !isPlaying) {
      // Only update isPlaying when auto-scroll is re-enabled and polling was stopped
      isPlaying = true;
      startPolling();
    }
  }
  
  onDestroy(async () => {
    console.log("Component being destroyed, cleaning up resources");
    
    try {
      // First make sure polling is stopped
      if (isPlaying) {
        console.log("Stopping polling during component cleanup");
        isPlaying = false;
        await stopPolling();
      }
      
      // Terminate the web worker
      if (logWorker) {
        console.log("Terminating log worker");
        logWorker.terminate();
        logWorker = null;
      }
      
      console.log("Cleanup completed successfully");
    } catch (error) {
      console.error("Error during component cleanup:", error);
    }
  });
</script>

<AppLayout>
  <div class="p-4">
    <h1 class="text-2xl font-bold mb-4">Live Firewall Logs</h1>

    <div class="mb-4 flex flex-col sm:flex-row justify-between gap-2">
      <button 
        class="btn btn-primary"
        on:click|preventDefault|stopPropagation={togglePlay}
        on:keydown={(e) => e.key === 'Enter' && togglePlay()}
        aria-label={isPlaying ? "Pause log updates" : "Start log updates"}
        data-testid="play-pause-button"
      >
        <svg class="w-6 h-6 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={isPlaying ? mdiPause : mdiPlay} />
        </svg>
        {isPlaying ? "Pause" : "Play"}
      </button>
      <button class="btn btn-secondary" on:click={toggleFilters}>
        <svg class="w-6 h-6 mr-2" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiFilter} />
        </svg>
        Filters
      </button>
    </div>

    {#if showFilters && filters && interfaceNames}
      <div class="mb-4 p-4 bg-base-200 rounded-lg">
        <h2 class="text-lg font-semibold mb-2">Filters</h2>
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <select
            bind:value={selectedAction}
            class="select select-bordered w-full"
          >
            <option value="">Select Action</option>
            {#each filters.action as action}
              <option value={action}>{action}</option>
            {/each}
          </select>
          <select
            bind:value={selectedInterface}
            class="select select-bordered w-full"
          >
            <option value="">Select Interface</option>
            {#each Object.entries(interfaceNames) as [key, value]}
              <option value={key}>{value}</option>
            {/each}
          </select>
          <select
            bind:value={selectedDirection}
            class="select select-bordered w-full"
          >
            <option value="">Select Direction</option>
            {#each filters.dir as dir}
              <option value={dir}>{dir}</option>
            {/each}
          </select>
        </div>
        <button
          class="btn btn-primary mt-4 w-full sm:w-auto"
          on:click={applyFilters}>Apply Filters</button
        >
      </div>
    {/if}

    {#if isLoading && logs.length === 0}
      <div class="flex justify-center items-center py-12">
        <div class="text-center">
          <span class="loading loading-spinner loading-lg"></span>
          <p class="mt-4 text-base-content">Loading firewall logs...</p>
        </div>
      </div>
    {:else if logs.length === 0}
      <div class="flex justify-center items-center py-12">
        <div class="text-center p-6 bg-base-200 rounded-lg max-w-md">
          <svg class="w-12 h-12 mx-auto text-base-content/50" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiTextBoxSearch} />
          </svg>
          <h3 class="mt-4 text-lg font-medium">No logs found</h3>
          <p class="mt-2 text-base-content/70">
            {selectedAction || selectedInterface || selectedDirection
              ? "Try changing or clearing your filters"
              : "No firewall logs are available for the current criteria"}
          </p>
        </div>
      </div>
    {:else}
      <!-- Mobile view with virtual scrolling -->
      <div class="lg:hidden">
        <div 
          class="h-[calc(100vh-20rem)] overflow-y-auto space-y-2 pb-4 px-1"
          bind:this={logContainer}
          on:scroll={handleScroll}
        >
          {#each logs as log}
            <div class="w-full mb-2">
              <div
                class="card bg-base-100 shadow-sm {log.action === 'pass'
                  ? 'border-l-4 border-success'
                  : log.action === 'block'
                    ? 'border-l-4 border-error'
                    : ''}"
              >
                <div class="card-body p-2">
                  <div class="flex justify-between items-center mb-1">
                    <span class="text-xs font-semibold"
                      >{log.__timestamp__
                        ? formatTimestamp(log.__timestamp__)
                        : "N/A"}</span
                    >
                    <span
                      class="badge badge-sm {log.action === 'pass'
                        ? 'badge-success'
                        : log.action === 'block'
                          ? 'badge-error'
                          : ''}"
                    >
                      {log.action || "N/A"}
                    </span>
                  </div>
    
                  <!-- Interface and Protocol in one row -->
                  <div class="grid grid-cols-2 gap-x-2 text-xs mb-1">
                    <div class="overflow-hidden">
                      <span class="font-semibold">Interface:</span>
                      <span
                        class="truncate inline-block max-w-[100px]"
                        title={interfaceNames?.[log.interface || ""] ||
                          log.interface ||
                          "N/A"}
                      >
                        {interfaceNames?.[log.interface || ""] ||
                          log.interface ||
                          "N/A"}
                      </span>
                    </div>
                    <div>
                      <span class="font-semibold">Protocol:</span>
                      {log.protoname || "N/A"}
                    </div>
                  </div>
    
                  <!-- Source and destination -->
                  <div class="text-xs space-y-1">
                    <div class="overflow-hidden text-ellipsis">
                      <span class="font-semibold">Source:</span>
                      <span class="font-mono break-all">
                        {log.src
                          ? `${log.src}${log.srcport ? `:${log.srcport}` : ""}`
                          : "N/A"}
                      </span>
                    </div>
                    <div class="overflow-hidden text-ellipsis">
                      <span class="font-semibold">Destination:</span>
                      <span class="font-mono break-all">
                        {log.dst
                          ? `${log.dst}${log.dstport ? `:${log.dstport}` : ""}`
                          : "N/A"}
                      </span>
                    </div>
                  </div>
    
                  <!-- Label with text wrapping -->
                  <div class="text-xs mt-1 break-words">
                    <span class="font-semibold">Label:</span>
                    {log.label || "N/A"}
                  </div>
    
                  <div class="card-actions justify-end mt-1">
                    <button
                      class="btn btn-xs btn-ghost"
                      on:click={() => showLogDetails(log)}
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiInformation} />
                      </svg>
                      Details
                    </button>
                  </div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Desktop view with normal scrolling -->
      <div class="hidden lg:block overflow-x-auto">
        <div 
          class="h-[calc(100vh-20rem)] overflow-y-auto w-full" 
          bind:this={logContainer}
          on:scroll={handleScroll}
        >
          <table class="table table-compact w-full">
            <thead class="sticky top-0 bg-base-100 z-10">
              <tr>
                <th class="py-2 text-xs">Time</th>
                <th class="py-2 text-xs">Interface</th>
                <th class="py-2 text-xs">Source</th>
                <th class="py-2 text-xs">Destination</th>
                <th class="py-2 text-xs">Protocol</th>
                <th class="py-2 text-xs">Action</th>
                <th class="py-2 text-xs">Label</th>
                <th class="py-2 text-xs">Details</th>
              </tr>
            </thead>
            <tbody>
              {#each logs as log}
                <tr
                  class="hover:bg-base-200 {log.action === 'pass'
                    ? 'bg-success/10'
                    : log.action === 'block'
                      ? 'bg-error/10'
                      : ''}"
                >
                  <td class="py-1 text-xs"
                    >{log.__timestamp__
                      ? formatTimestamp(log.__timestamp__)
                      : "N/A"}</td
                  >
                  <td class="py-1 text-xs"
                    >{interfaceNames?.[log.interface || ""] ||
                      log.interface ||
                      "N/A"}</td
                  >
                  <td class="py-1 text-xs"
                    >{log.src
                      ? `${log.src}${log.srcport ? `:${log.srcport}` : ""}`
                      : "N/A"}</td
                  >
                  <td class="py-1 text-xs"
                    >{log.dst
                      ? `${log.dst}${log.dstport ? `:${log.dstport}` : ""}`
                      : "N/A"}</td
                  >
                  <td class="py-1 text-xs">{log.protoname || "N/A"}</td>
                  <td class="py-1 text-xs">
                    <span
                      class="badge badge-sm {log.action === 'pass'
                        ? 'badge-success'
                        : log.action === 'block'
                          ? 'badge-error'
                          : ''}"
                    >
                      {log.action || "N/A"}
                    </span>
                  </td>
                  <td class="py-1 text-xs">{log.label || "N/A"}</td>
                  <td class="py-1 text-xs">
                    <button
                      class="btn btn-xs btn-ghost p-1"
                      on:click={() => showLogDetails(log)}
                    >
                      <svg class="w-3 h-3" viewBox="0 0 24 24">
                        <path fill="currentColor" d={mdiInformation} />
                      </svg>
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

    {#if showModal && selectedLog}
      <div class="modal modal-open">
        <div class="modal-box relative max-w-3xl">
          <button
            class="btn btn-sm btn-circle absolute right-2 top-2"
            on:click={() => (showModal = false)}
          >
            ✕
          </button>
          <h3 class="font-bold text-lg pr-8 mb-4">Log Details</h3>

          <div
            class="grid grid-cols-1 md:grid-cols-2 gap-6 max-h-[70vh] overflow-y-auto"
          >
            <!-- Connection Information -->
            <div>
              <h4
                class="font-medium text-base mb-2 pb-1 border-b border-base-300"
              >
                Connection
              </h4>
              <div class="space-y-2">
                <div class="grid grid-cols-3 gap-1">
                  <span class="font-semibold text-sm">Action:</span>
                  <span class="col-span-2">
                    <span
                      class="badge {selectedLog.action === 'pass'
                        ? 'badge-success'
                        : selectedLog.action === 'block'
                          ? 'badge-error'
                          : ''}"
                    >
                      {selectedLog.action || "N/A"}
                    </span>
                  </span>
                </div>

                <div class="grid grid-cols-3 gap-1">
                  <span class="font-semibold text-sm">Source:</span>
                  <span class="col-span-2 break-all font-mono text-sm"
                    >{selectedLog.src || "N/A"}</span
                  >
                </div>

                {#if selectedLog.srcport}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Source Port:</span>
                    <span class="col-span-2 font-mono text-sm"
                      >{selectedLog.srcport}</span
                    >
                  </div>
                {/if}

                <div class="grid grid-cols-3 gap-1">
                  <span class="font-semibold text-sm">Destination:</span>
                  <span class="col-span-2 break-all font-mono text-sm"
                    >{selectedLog.dst || "N/A"}</span
                  >
                </div>

                {#if selectedLog.dstport}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Dest Port:</span>
                    <span class="col-span-2 font-mono text-sm"
                      >{selectedLog.dstport}</span
                    >
                  </div>
                {/if}

                <div class="grid grid-cols-3 gap-1">
                  <span class="font-semibold text-sm">Protocol:</span>
                  <span class="col-span-2">
                    {#if selectedLog.protoname}
                      <span class="badge badge-neutral"
                        >{selectedLog.protoname}</span
                      >
                    {:else}
                      N/A
                    {/if}
                  </span>
                </div>

                {#if selectedLog.dir}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Direction:</span>
                    <span class="col-span-2">{selectedLog.dir}</span>
                  </div>
                {/if}

                {#if selectedLog.tcpflags}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">TCP Flags:</span>
                    <span class="col-span-2 font-mono text-sm"
                      >{selectedLog.tcpflags}</span
                    >
                  </div>
                {/if}
              </div>
            </div>

            <!-- Firewall Information -->
            <div>
              <h4
                class="font-medium text-base mb-2 pb-1 border-b border-base-300"
              >
                Firewall Details
              </h4>
              <div class="space-y-2">
                {#if selectedLog.interface}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Interface:</span>
                    <span class="col-span-2"
                      >{interfaceNames?.[selectedLog.interface] ||
                        selectedLog.interface}</span
                    >
                  </div>
                {/if}

                {#if selectedLog.rulenr}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Rule:</span>
                    <span class="col-span-2 font-mono text-sm"
                      >{selectedLog.rulenr}</span
                    >
                  </div>
                {/if}

                {#if selectedLog.label}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Label:</span>
                    <span class="col-span-2 break-words"
                      >{selectedLog.label}</span
                    >
                  </div>
                {/if}

                {#if selectedLog.reason}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Reason:</span>
                    <span class="col-span-2">{selectedLog.reason}</span>
                  </div>
                {/if}

                {#if selectedLog.__timestamp__}
                  <div class="grid grid-cols-3 gap-1">
                    <span class="font-semibold text-sm">Time:</span>
                    <span class="col-span-2"
                      >{formatTimestamp(selectedLog.__timestamp__)}</span
                    >
                  </div>
                {/if}
              </div>
            </div>

            <!-- Additional Information (if needed) -->
            {#if selectedLog.ipversion || selectedLog.ttl || selectedLog.tos || selectedLog.id || selectedLog.length || selectedLog.datalen}
              <div class="col-span-1 md:col-span-2 mt-2">
                <h4
                  class="font-medium text-base mb-2 pb-1 border-b border-base-300"
                >
                  Packet Details
                </h4>
                <div
                  class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-x-4 gap-y-2"
                >
                  {#if selectedLog.ipversion}
                    <div>
                      <span class="font-semibold text-sm">IP Version:</span>
                      <span class="ml-1">{selectedLog.ipversion}</span>
                    </div>
                  {/if}

                  {#if selectedLog.ttl}
                    <div>
                      <span class="font-semibold text-sm">TTL:</span>
                      <span class="ml-1">{selectedLog.ttl}</span>
                    </div>
                  {/if}

                  {#if selectedLog.tos}
                    <div>
                      <span class="font-semibold text-sm">TOS:</span>
                      <span class="ml-1">{selectedLog.tos}</span>
                    </div>
                  {/if}

                  {#if selectedLog.id}
                    <div>
                      <span class="font-semibold text-sm">ID:</span>
                      <span class="ml-1">{selectedLog.id}</span>
                    </div>
                  {/if}

                  {#if selectedLog.length}
                    <div>
                      <span class="font-semibold text-sm">Length:</span>
                      <span class="ml-1">{selectedLog.length}</span>
                    </div>
                  {/if}

                  {#if selectedLog.datalen}
                    <div>
                      <span class="font-semibold text-sm">Data Length:</span>
                      <span class="ml-1">{selectedLog.datalen}</span>
                    </div>
                  {/if}

                  {#if selectedLog.seq}
                    <div>
                      <span class="font-semibold text-sm">Sequence:</span>
                      <span class="ml-1 font-mono">{selectedLog.seq}</span>
                    </div>
                  {/if}

                  {#if selectedLog.ack}
                    <div>
                      <span class="font-semibold text-sm">ACK:</span>
                      <span class="ml-1 font-mono">{selectedLog.ack}</span>
                    </div>
                  {/if}
                </div>
              </div>
            {/if}
          </div>

          <!-- Toggle for Raw JSON View -->
          <div class="mt-4 pt-2 border-t border-base-300">
            <button
              class="btn btn-sm"
              on:click={() => (showRawJson = !showRawJson)}
            >
              {showRawJson ? "Hide" : "Show"} Raw JSON
            </button>
            {#if showRawJson}
              <div class="mt-2 bg-base-200 p-2 rounded overflow-x-auto">
                <pre
                  class="text-xs font-mono whitespace-pre-wrap break-words">{JSON.stringify(
                    selectedLog,
                    null,
                    2,
                  )}</pre>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</AppLayout>