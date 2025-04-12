<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { toasts } from "$lib/stores/toastStore";
    import {
        mdiCheck,
        mdiClose,
        mdiDelete,
        mdiPlus,
        mdiRefresh,
        mdiSelectAll,
        mdiSelectOff,
    } from "@mdi/js";
    import AppLayout from "../AppLayout.svelte";

    interface Route {
        uuid: string;
        disabled: string;
        network: string;
        gateway: string;
        descr: string;
    }

    interface GatewayOption {
        value: string;
        selected: number;
    }

    let routes: (Route & { selected?: boolean })[] = [];
    let isLoading = true;
    let showAddModal = false;
    let showConfirmDelete = false;
    let showConfirmToggle = false;
    let selectedRoute: Route | null = null;
    let gatewayOptions: Record<string, GatewayOption> = {};
    let newRoute = {
        network: "",
        gateway: "",
        description: "",
        disabled: false,
    };
    let selectedAction = "";
    let selectAll = false;
    let isActionLoading = false;

    $: selectedRoutes = routes.filter((route) => route.selected);
    $: hasSelectedRoutes = selectedRoutes.length > 0;

    onMount(async () => {
        await loadRoutes();
        await loadGatewayOptions();
    });

    async function loadRoutes() {
        try {
            const response = await invoke<{ rows: Route[] }>("get_routes");
            routes = response.rows.map((route) => ({
                ...route,
                selected: false,
            }));
            isLoading = false;
        } catch (error) {
            console.error("Failed to load routes:", error);
            toasts.error(`Failed to load routes: ${error}`);
            isLoading = false;
        }
    }

    async function loadGatewayOptions() {
        try {
            const response = await invoke<{
                route: { gateway: Record<string, GatewayOption> };
            }>("get_route_info");
            gatewayOptions = response.route.gateway;
        } catch (error) {
            console.error("Failed to load gateway options:", error);
            toasts.error(`Failed to load gateway options: ${error}`);
        }
    }

    function openAddModal() {
        showAddModal = true;
        newRoute = {
            network: "",
            gateway: "",
            description: "",
            disabled: false,
        };
    }

    function closeAddModal() {
        showAddModal = false;
    }

    async function handleAddRoute() {
        try {
            if (!newRoute.gateway) {
                toasts.error("Please select a gateway");
                return;
            }

            const gatewayName = newRoute.gateway.split(" - ")[0];

            const result = await invoke("add_route", {
                network: newRoute.network,
                gateway: gatewayName,
                description: newRoute.description,
                disabled: newRoute.disabled,
            });

            console.log("Add route result:", result);

            // No need to apply changes anymore, it's done in the backend
            await loadRoutes();
            closeAddModal();
            toasts.success("Route added and applied successfully");
        } catch (error) {
            console.error("Failed to add route:", error);
            toasts.error(`Failed to add route: ${error}`);
        }
    }

    function openDeleteConfirmation(route?: Route) {
        if (route) {
            selectedRoute = route;
        }
        showConfirmDelete = true;
    }

    function closeDeleteConfirmation() {
        showConfirmDelete = false;
        selectedRoute = null;
    }

    function openToggleConfirmation(route?: Route) {
        if (route) {
            selectedRoute = route;
        }
        showConfirmToggle = true;
    }

    function closeToggleConfirmation() {
        showConfirmToggle = false;
        selectedRoute = null;
    }

    async function handleDeleteRoute() {
        try {
            isActionLoading = true;
            if (selectedRoute) {
                await invoke("delete_route", { uuid: selectedRoute.uuid });
            } else if (hasSelectedRoutes) {
                for (const route of selectedRoutes) {
                    await invoke("delete_route", { uuid: route.uuid });
                }
            }

            // No need to apply changes anymore, it's done in the backend
            await loadRoutes();
            closeDeleteConfirmation();
            toasts.success(
                selectedRoute
                    ? "Route deleted and applied successfully"
                    : "Routes deleted and applied successfully",
            );
        } catch (error) {
            console.error("Failed to delete route(s):", error);
            toasts.error(`Failed to delete route(s): ${error}`);
        } finally {
            isActionLoading = false;
        }
    }

    async function handleToggleRoute() {
        try {
            isActionLoading = true;
            if (selectedRoute) {
                await invoke("toggle_route", { uuid: selectedRoute.uuid });
            } else if (hasSelectedRoutes) {
                for (const route of selectedRoutes) {
                    await invoke("toggle_route", { uuid: route.uuid });
                }
            }

            // No need to apply changes anymore, it's done in the backend
            await loadRoutes();
            closeToggleConfirmation();
            toasts.success("Route(s) toggled and applied successfully");
        } catch (error) {
            console.error("Failed to toggle route(s):", error);
            toasts.error(`Failed to toggle route(s): ${error}`);
        } finally {
            isActionLoading = false;
        }
    }

    function toggleSelectAll() {
        selectAll = !selectAll;
        routes = routes.map((route) => ({
            ...route,
            selected: selectAll,
        }));
    }
    function handleRouteSelection(route: Route & { selected?: boolean }) {
        route.selected = !route.selected;
        routes = routes; // Trigger reactivity
        // Update selectAll based on whether all routes are selected
        selectAll = routes.every((r) => r.selected);
    }
    function handleBulkAction() {
        if (!hasSelectedRoutes) return;

        if (selectedAction === "delete") {
            openDeleteConfirmation();
        } else if (selectedAction === "toggle") {
            openToggleConfirmation();
        }

        selectedAction = "";
    }
</script>

<AppLayout>
    <div class="container mx-auto p-4 max-w-5xl">
        <!-- Header Controls Card -->
        <div class="card bg-base-100 shadow-xl mb-6">
            <div class="card-body p-4">
                <div class="flex flex-col gap-2">
                    <div class="flex justify-between items-center">
                        <h1 class="text-xl font-bold">Network Routes</h1>
                        <div class="flex gap-2">
                            <label class="cursor-pointer flex items-center gap-2">
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-sm"
                                    checked={selectAll}
                                    on:change={toggleSelectAll}
                                />
                            </label>
                            <button class="btn btn-sm btn-ghost" on:click={loadRoutes}>
                                <svg class="w-5 h-5" viewBox="0 0 24 24">
                                    <path fill="currentColor" d={mdiRefresh} />
                                </svg>
                            </button>
                            <button class="btn btn-sm btn-primary" on:click={openAddModal}>
                                <svg class="w-5 h-5" viewBox="0 0 24 24">
                                    <path fill="currentColor" d={mdiPlus} />
                                </svg>
                            </button>
                        </div>
                    </div>
                    {#if hasSelectedRoutes}
                        <select
                            class="select select-bordered select-sm w-full"
                            bind:value={selectedAction}
                            on:change={handleBulkAction}
                        >
                            <option value="">Bulk Actions ({selectedRoutes.length} selected)</option>
                            <option value="toggle">Toggle Selected</option>
                            <option value="delete">Delete Selected</option>
                        </select>
                    {/if}
                </div>
            </div>
        </div>
    
        <!-- Routes Listing -->
        {#if isLoading}
            <div class="flex justify-center items-center h-32">
                <span class="loading loading-spinner loading-lg"></span>
            </div>
        {:else}
            {#each routes as route (route.uuid)}
                <div 
                    class="card bg-base-100 shadow-lg hover:shadow-xl transition-all duration-200 mb-4 overflow-hidden"
                    class:opacity-60={route.disabled === '1'}
                >
                    <div class="card-body p-4 relative">
                        <!-- Status Indicator -->
                        <div 
                            class="absolute left-0 top-0 bottom-0 w-1.5 {route.disabled === '1' ? 'bg-error' : 'bg-success'}"
                        ></div>
    
                        <div class="flex items-center justify-between gap-4 pl-2">
                            <!-- Left Side: Checkbox and Info -->
                            <label class="cursor-pointer flex items-center gap-4 flex-1">
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-sm"
                                    checked={route.selected}
                                    on:change={() => handleRouteSelection(route)}
                                />
                                <div class="flex-1 min-w-0"> <!-- min-w-0 helps with text truncation -->
                                    <div class="font-mono text-base font-medium">{route.network}</div>
                                    <div class="font-mono text-sm opacity-75 mt-1">{route.gateway}</div>
                                    {#if route.descr}
                                        <div class="text-sm opacity-60 mt-1 truncate">
                                            {route.descr}
                                        </div>
                                    {/if}
                                </div>
                            </label>
    
                            <!-- Right Side: Action Buttons -->
                            <div class="flex items-center gap-3">
                                <button
                                    class="btn btn-sm {route.disabled === '1' ? 'btn-error' : 'btn-success'} gap-2"
                                    on:click={() => openToggleConfirmation(route)}
                                >
                                    <svg class="w-4 h-4" viewBox="0 0 24 24">
                                        <path
                                            fill="currentColor"
                                            d={route.disabled === "1" ? mdiClose : mdiCheck}
                                        />
                                    </svg>
                                    {route.disabled === "1" ? "Off" : "On"}
                                </button>
                                
                                <button
                                    class="btn btn-sm btn-ghost text-error hover:bg-error hover:text-white transition-colors"
                                    on:click={() => openDeleteConfirmation(route)}
                                >
                                    <svg class="w-4 h-4" viewBox="0 0 24 24">
                                        <path fill="currentColor" d={mdiDelete} />
                                    </svg>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            {/each}
        {/if}
    </div>

    <!-- Add Route Modal -->
    {#if showAddModal}
        <div class="modal modal-open">
            <div class="modal-box max-w-sm">
                <h3 class="font-bold text-lg mb-4">Add New Route</h3>
                <form
                    on:submit|preventDefault={handleAddRoute}
                    class="space-y-4"
                >
                    <div class="form-control">
                        <label class="label" for="network">
                            <span class="label-text">Network</span>
                        </label>
                        <input
                            type="text"
                            id="network"
                            class="input input-bordered w-full"
                            bind:value={newRoute.network}
                            placeholder="e.g., 192.168.1.0/24"
                            required
                        />
                    </div>

                    <div class="form-control">
                        <label class="label" for="gateway">
                            <span class="label-text">Gateway</span>
                        </label>
                        <select
                            id="gateway"
                            class="select select-bordered w-full"
                            bind:value={newRoute.gateway}
                            required
                        >
                            <option value="">Select a gateway</option>
                            {#each Object.entries(gatewayOptions) as [key, option]}
                                <option value={option.value}
                                    >{option.value}</option
                                >
                            {/each}
                        </select>
                    </div>

                    <div class="form-control">
                        <label class="label" for="description">
                            <span class="label-text">Description</span>
                        </label>
                        <input
                            type="text"
                            id="description"
                            class="input input-bordered w-full"
                            bind:value={newRoute.description}
                            placeholder="Route description"
                        />
                    </div>

                    <div class="form-control">
                        <label class="label cursor-pointer justify-start gap-4">
                            <span class="label-text">Disabled</span>
                            <input
                                type="checkbox"
                                class="toggle"
                                bind:checked={newRoute.disabled}
                            />
                        </label>
                    </div>

                    <div class="modal-action">
                        <button
                            type="button"
                            class="btn"
                            on:click={closeAddModal}
                            disabled={isActionLoading}
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary {isActionLoading
                                ? 'loading'
                                : ''}"
                            disabled={isActionLoading}
                        >
                            Save & Apply
                        </button>
                    </div>
                </form>
            </div>
        </div>
    {/if}

    <!-- Delete Confirmation Modal -->
    {#if showConfirmDelete}
        <div class="modal modal-open">
            <div class="modal-box">
                <h3 class="font-bold text-lg">Confirm Delete</h3>
                <p class="py-4">
                    {#if selectedRoute}
                        Are you sure you want to delete the route to {selectedRoute.network}?
                    {:else}
                        Are you sure you want to delete {selectedRoutes.length} selected
                        routes?
                    {/if}
                    This action cannot be undone.
                </p>
                <div class="modal-action">
                    <button
                        class="btn"
                        on:click={closeDeleteConfirmation}
                        disabled={isActionLoading}
                    >
                        Cancel
                    </button>
                    <button
                        class="btn btn-error {isActionLoading ? 'loading' : ''}"
                        on:click={handleDeleteRoute}
                        disabled={isActionLoading}
                    >
                        Delete & Apply
                    </button>
                </div>
            </div>
        </div>
    {/if}

    <!-- Toggle Confirmation Modal -->
    {#if showConfirmToggle}
        <div class="modal modal-open">
            <div class="modal-box">
                <h3 class="font-bold text-lg">Confirm Toggle</h3>
                <p class="py-4">
                    {#if selectedRoute}
                        Are you sure you want to {selectedRoute.disabled === "1"
                            ? "enable"
                            : "disable"} the route to {selectedRoute.network}?
                    {:else}
                        Are you sure you want to toggle {selectedRoutes.length} selected
                        routes?
                    {/if}
                </p>
                <div class="modal-action">
                    <button
                        class="btn"
                        on:click={closeToggleConfirmation}
                        disabled={isActionLoading}
                    >
                        Cancel
                    </button>
                    <button
                        class="btn btn-primary {isActionLoading
                            ? 'loading'
                            : ''}"
                        on:click={handleToggleRoute}
                        disabled={isActionLoading}
                    >
                        Save & Apply
                    </button>
                </div>
            </div>
        </div>
    {/if}
</AppLayout>
