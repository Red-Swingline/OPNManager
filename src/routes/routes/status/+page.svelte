<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { toasts } from "$lib/stores/toastStore";
    import {
        mdiRefresh,
        mdiMapMarkerPath,
        mdiIpNetwork,
        mdiEthernet,
        mdiLanConnect,
    } from "@mdi/js";
    import AppLayout from "../../AppLayout.svelte";

    interface RouteTableEntry {
        proto: string;
        destination: string;
        gateway: string;
        flags: string;
        nhop: string;
        mtu: string;
        netif: string;
        expire: string;
        intf_description: string;
    }

    let routeTable: RouteTableEntry[] = [];
    let isLoading = true;
    let searchTerm = "";
    let filterProtocol = "all";
    let filterInterface = "all";
    let showLegend = false;

    // Lists for filters
    let interfaceOptions: string[] = [];
    let protocolOptions: string[] = ["all", "ipv4", "ipv6"];

    onMount(async () => {
        await loadRouteTable();
    });

    async function loadRouteTable() {
        try {
            isLoading = true;
            const tableData = await invoke<RouteTableEntry[]>("get_route_table");
            routeTable = tableData;
            
            // Extract unique interface options for filtering
            const interfaces = new Set<string>();
            tableData.forEach(entry => {
                if (entry.intf_description) {
                    interfaces.add(entry.intf_description);
                }
            });
            interfaceOptions = ["all", ...Array.from(interfaces)];
            
            isLoading = false;
        } catch (error) {
            console.error("Failed to load route table:", error);
            toasts.error(`Failed to load route table: ${error}`);
            isLoading = false;
        }
    }

    $: filteredRouteTable = routeTable
        .filter(route => {
            // Search term filter
            const searchLower = searchTerm.toLowerCase();
            const matchSearch = searchTerm === "" || 
                route.destination.toLowerCase().includes(searchLower) ||
                route.gateway.toLowerCase().includes(searchLower) ||
                route.netif.toLowerCase().includes(searchLower) ||
                route.intf_description.toLowerCase().includes(searchLower);
            
            // Protocol filter
            const matchProtocol = filterProtocol === "all" || 
                route.proto.toLowerCase() === filterProtocol;
            
            // Interface filter
            const matchInterface = filterInterface === "all" || 
                route.intf_description === filterInterface;
            
            return matchSearch && matchProtocol && matchInterface;
        });

    // Get class for flags
    function getFlagClass(flags: string) {
        if (flags.includes('H')) return 'text-warning'; // Host routes
        if (flags.includes('G')) return 'text-info';    // Gateway routes
        if (flags.includes('S')) return 'text-success'; // Static routes
        if (flags.includes('D')) return 'text-accent';  // Dynamic routes
        return 'text-base-content';
    }

    // Explain flags
    function getFlagDescription(flags: string) {
        const flagDescriptions: Record<string, string> = {
            'U': 'Up',
            'G': 'Gateway',
            'H': 'Host',
            'R': 'Reinstate',
            'D': 'Dynamic',
            'M': 'Modified',
            'A': 'Announced',
            'C': 'Clone',
            'L': 'Link',
            'W': 'WasCloned',
            'S': 'Static',
            'B': 'Blackhole',
            'P': 'Protospec'
        };
        
        return flags.split('').map(flag => flagDescriptions[flag] || flag).join(', ');
    }
</script>

<AppLayout>
    <div class="container mx-auto p-4 max-w-6xl">
        <!-- Header Controls Card -->
        <div class="card bg-base-100 shadow-xl mb-6">
            <div class="card-body p-4">
                <div class="flex justify-between items-center mb-4">
                    <h1 class="text-xl font-bold">Route Table Status</h1>
                    <button class="btn btn-sm btn-ghost" on:click={loadRouteTable}>
                        <svg class="w-5 h-5" viewBox="0 0 24 24">
                            <path fill="currentColor" d={mdiRefresh} />
                        </svg>
                    </button>
                </div>
                
                <!-- Filters -->
                <div class="space-y-4 md:space-y-0 md:grid md:grid-cols-3 md:gap-4">
                    <!-- Search Input (full width on mobile) -->
                    <div class="form-control">
                        <div class="relative flex items-center">
                            <div class="absolute left-3 top-1/2 transform -translate-y-1/2 pointer-events-none z-10">
                                <svg class="w-5 h-5" viewBox="0 0 24 24">
                                    <path fill="currentColor" d={mdiMapMarkerPath} />
                                </svg>
                            </div>
                            <input
                                type="text"
                                placeholder="Search routes..."
                                class="input input-bordered w-full pl-10"
                                bind:value={searchTerm}
                            />
                        </div>
                    </div>
                    
                    <!-- Protocol filter -->
                    <div class="form-control">
                        <div class="relative flex items-center">
                            <div class="absolute left-3 top-1/2 transform -translate-y-1/2 pointer-events-none z-10">
                                <svg class="w-5 h-5" viewBox="0 0 24 24">
                                    <path fill="currentColor" d={mdiIpNetwork} />
                                </svg>
                            </div>
                            <select class="select select-bordered w-full pl-10 text-sm md:text-base" bind:value={filterProtocol}>
                                {#each protocolOptions as protocol}
                                    <option value={protocol}>{protocol.toUpperCase()}</option>
                                {/each}
                            </select>
                        </div>
                    </div>
                    
                    <!-- Interface filter -->
                    <div class="form-control">
                        <div class="relative flex items-center">
                            <div class="absolute left-3 top-1/2 transform -translate-y-1/2 pointer-events-none z-10">
                                <svg class="w-5 h-5" viewBox="0 0 24 24">
                                    <path fill="currentColor" d={mdiEthernet} />
                                </svg>
                            </div>
                            <select class="select select-bordered w-full pl-10 text-sm md:text-base" bind:value={filterInterface}>
                                {#each interfaceOptions as interfaceName}
                                    <option value={interfaceName}>{interfaceName === 'all' ? 'All Interfaces' : interfaceName}</option>
                                {/each}
                            </select>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        
        <!-- Route Table -->
        {#if isLoading}
            <div class="flex justify-center items-center h-32">
                <span class="loading loading-spinner loading-lg"></span>
            </div>
        {:else if filteredRouteTable.length === 0}
            <div class="card bg-base-100 shadow-xl p-8 text-center">
                <p class="text-lg">No routes match your filters</p>
            </div>
        {:else}
            <!-- Desktop/Tablet View (hidden on small screens) -->
            <div class="hidden md:block overflow-x-auto">
                <table class="table table-zebra w-full">
                    <thead>
                        <tr>
                            <th>Protocol</th>
                            <th>Destination</th>
                            <th>Gateway</th>
                            <th>Flags</th>
                            <th>MTU</th>
                            <th>Interface</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each filteredRouteTable as route}
                            <tr class="hover">
                                <td class="font-mono">
                                    <span class="badge {route.proto === 'ipv4' ? 'badge-primary' : 'badge-secondary'}">
                                        {route.proto}
                                    </span>
                                </td>
                                <td class="font-mono">{route.destination}</td>
                                <td class="font-mono">{route.gateway}</td>
                                <td>
                                    <div class="tooltip" data-tip={getFlagDescription(route.flags)}>
                                        <span class="font-mono {getFlagClass(route.flags)}">{route.flags}</span>
                                    </div>
                                </td>
                                <td class="font-mono">{route.mtu}</td>
                                <td>
                                    <div class="flex items-center">
                                        <svg class="w-4 h-4 mr-1" viewBox="0 0 24 24">
                                            <path fill="currentColor" d={mdiLanConnect} />
                                        </svg>
                                        <span>{route.netif}</span>
                                        {#if route.intf_description}
                                            <span class="ml-1 opacity-70">({route.intf_description})</span>
                                        {/if}
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
            
            <!-- Mobile View (card-based for better readability) -->
            <div class="md:hidden space-y-4">
                {#each filteredRouteTable as route}
                    <div class="card bg-base-100 shadow-lg hover:shadow-xl transition-all duration-200">
                        <div class="card-body p-4">
                            <div class="flex justify-between items-start mb-2">
                                <span class="badge {route.proto === 'ipv4' ? 'badge-primary' : 'badge-secondary'} mr-2">
                                    {route.proto}
                                </span>
                                <div class="tooltip" data-tip={getFlagDescription(route.flags)}>
                                    <span class="badge badge-outline font-mono {getFlagClass(route.flags)}">{route.flags}</span>
                                </div>
                            </div>
                            
                            <div class="grid grid-cols-3 gap-1 text-xs mb-1">
                                <span class="font-semibold">Destination:</span>
                                <span class="font-mono col-span-2 break-all">{route.destination}</span>
                            </div>
                            
                            <div class="grid grid-cols-3 gap-1 text-xs mb-1">
                                <span class="font-semibold">Gateway:</span>
                                <span class="font-mono col-span-2 break-all">{route.gateway}</span>
                            </div>
                            
                            <div class="grid grid-cols-3 gap-1 text-xs">
                                <span class="font-semibold">Interface:</span>
                                <div class="col-span-2 flex items-center">
                                    <svg class="w-3 h-3 mr-1" viewBox="0 0 24 24">
                                        <path fill="currentColor" d={mdiLanConnect} />
                                    </svg>
                                    <span>{route.netif}</span>
                                    {#if route.intf_description}
                                        <span class="ml-1 opacity-70 text-xs">({route.intf_description})</span>
                                    {/if}
                                </div>
                            </div>
                            
                            <div class="grid grid-cols-3 gap-1 text-xs">
                                <span class="font-semibold">MTU:</span>
                                <span class="font-mono">{route.mtu}</span>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
            
            <div class="mt-4 text-sm opacity-70">
                <p>Showing {filteredRouteTable.length} of {routeTable.length} routes</p>
            </div>
        {/if}
        
        <!-- Flag Legend with toggle button on mobile -->
        <div class="card bg-base-100 shadow-xl mt-6">
            <div class="card-body p-4">
                <div class="flex justify-between items-center mb-2 md:mb-4">
                    <h3 class="font-bold">Route Flags Legend</h3>
                    <button 
                        class="btn btn-xs btn-ghost md:hidden" 
                        on:click={() => showLegend = !showLegend}
                    >
                        {showLegend ? 'Hide' : 'Show'}
                    </button>
                </div>
                
                <div class={showLegend ? 'block' : 'hidden md:block'}>
                    <div class="grid grid-cols-2 md:grid-cols-3 gap-x-4 gap-y-1 text-sm">
                        <div><span class="font-mono">U</span> - Up</div>
                        <div><span class="font-mono text-info">G</span> - Gateway</div>
                        <div><span class="font-mono text-warning">H</span> - Host</div>
                        <div><span class="font-mono">R</span> - Reinstate</div>
                        <div><span class="font-mono text-accent">D</span> - Dynamic</div>
                        <div><span class="font-mono">M</span> - Modified</div>
                        <div><span class="font-mono">A</span> - Announced</div>
                        <div><span class="font-mono">C</span> - Clone</div>
                        <div><span class="font-mono">L</span> - Link</div>
                        <div><span class="font-mono">W</span> - WasCloned</div>
                        <div><span class="font-mono text-success">S</span> - Static</div>
                        <div><span class="font-mono">B</span> - Blackhole</div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</AppLayout>