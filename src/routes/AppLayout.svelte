<script lang="ts">
  import {
    mdiMenu,
    mdiHome,
    mdiCog,
    mdiLogout,
    mdiRouter,
    mdiShieldSearch,
    mdiWallFire,
    mdiPowerStandby,
    mdiUpdate,
    mdiThemeLightDark,
    mdiTextBoxSearch,
    mdiMapMarkerPath,
    mdiDnsOutline,
    mdiChevronDown,
    mdiChevronUp,
  } from "@mdi/js";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/authStore";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toastStore";
  import { onMount } from "svelte";

  // Add iOS-specific scroll handling
  import { setupIOSScrolling } from "$lib/utils/iosScrollManager";

  export let title = "OPNManager";
  let isSidebarOpen = false;
  let isRebootDialogOpen = false;
  let theme = "light";
  let expandedCategories = { unbound: false };

  // Add scroll manager reference
  let scrollManager;

  const menuItems = [
    { path: "/", icon: mdiHome, label: "Home" },
    { path: "/devices", icon: mdiRouter, label: "Devices" },
    { path: "/alias", icon: mdiShieldSearch, label: "Alias" },
    { path: "/routes", icon: mdiMapMarkerPath, label: "Routes" },
    { path: "/rules", icon: mdiWallFire, label: "Firewall Rules" },
    { path: "/logs", icon: mdiTextBoxSearch, label: "Firewall Logs" },
    {
      category: "unbound",
      icon: mdiDnsOutline,
      label: "Unbound",
      items: [
        { path: "/unbound", icon: mdiDnsOutline, label: "DNS Blocklist" },
      ],
    },
    { path: "/updates", icon: mdiUpdate, label: "Updates" },
    { path: "/settings", icon: mdiCog, label: "Settings" },
  ];

  function toggleSidebar() {
    isSidebarOpen = !isSidebarOpen;
  }

  function handleLogout() {
    goto("/logout");
    isSidebarOpen = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      toggleSidebar();
    }
  }

  function handleOverlayKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " " || event.key === "Escape") {
      toggleSidebar();
    }
  }

  function handleNavigation(path: string) {
    if ($page.url.pathname !== path) {
      goto(path);
    }
    isSidebarOpen = false;
  }

  function toggleCategory(category: string) {
    expandedCategories[category] = !expandedCategories[category];
  }

  function openRebootDialog() {
    isRebootDialogOpen = true;
  }

  function closeRebootDialog() {
    isRebootDialogOpen = false;
  }

  async function handleReboot() {
    try {
      const response = await invoke("reboot_firewall");
      if (response && response.status === "ok") {
        toasts.success("Firewall reboot initiated successfully");
        closeRebootDialog();
      } else {
        toasts.error(`Failed to reboot firewall: Unexpected response`);
      }
    } catch (error) {
      console.error("Reboot error:", error);
      toasts.error(`Failed to reboot firewall: ${error}`);
    }
  }

  function toggleTheme() {
    theme = theme === "light" ? "dark" : "light";
    document.documentElement.setAttribute("data-theme", theme);

    localStorage.setItem("theme", theme);

    if (/iPad|iPhone|iPod/.test(navigator.userAgent) && !window.MSStream) {
      const header = document.querySelector("header.fixed-header");
      if (header) {
        header.classList.add("theme-updating");
        setTimeout(() => {
          header.classList.remove("theme-updating");
        }, 50);
      }
    }
  }

  function isInCategory(category) {
    return (
      category.items &&
      category.items.some((item) => $page.url.pathname === item.path)
    );
  }

  onMount(() => {
    const savedTheme = localStorage.getItem("theme") || "light";
    theme = savedTheme;
    document.documentElement.setAttribute("data-theme", theme);

    const isIOS =
      /iPad|iPhone|iPod/.test(navigator.userAgent) && !window.MSStream;
    if (isIOS) {
      scrollManager = setupIOSScrolling();
    }

    return () => {
      if (scrollManager && scrollManager.cleanup) {
        scrollManager.cleanup();
      }
    };
  });
</script>

<div class="flex h-screen bg-base-200" id="app-layout-container">
  <!-- Sidebar -->
  <aside class="hidden lg:flex flex-col w-64 bg-base-100">
    <div class="flex items-center justify-center h-16 bg-primary">
      <span class="text-xl font-bold text-primary-content">{title}</span>
    </div>
    <nav class="flex-1 overflow-y-auto">
      <ul class="p-2 space-y-2">
        {#each menuItems as item}
          {#if item.category}
            <li>
              <button
                on:click={() => toggleCategory(item.category)}
                class="flex items-center justify-between w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
                class:bg-base-300={isInCategory(item) ||
                  expandedCategories[item.category]}
                aria-expanded={expandedCategories[item.category]}
              >
                <div class="flex items-center space-x-3">
                  <svg class="w-6 h-6" viewBox="0 0 24 24">
                    <path fill="currentColor" d={item.icon} />
                  </svg>
                  <span>{item.label}</span>
                </div>
                <svg class="w-5 h-5" viewBox="0 0 24 24">
                  <path
                    fill="currentColor"
                    d={expandedCategories[item.category]
                      ? mdiChevronUp
                      : mdiChevronDown}
                  />
                </svg>
              </button>
              {#if expandedCategories[item.category]}
                <ul class="pl-8 mt-1 space-y-1">
                  {#each item.items as subItem}
                    <li>
                      <button
                        on:click={() => handleNavigation(subItem.path)}
                        class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
                        class:bg-base-300={$page.url.pathname === subItem.path}
                      >
                        <svg class="w-5 h-5" viewBox="0 0 24 24">
                          <path fill="currentColor" d={subItem.icon} />
                        </svg>
                        <span>{subItem.label}</span>
                      </button>
                    </li>
                  {/each}
                </ul>
              {/if}
            </li>
          {:else}
            <li>
              <button
                on:click={() => handleNavigation(item.path)}
                class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
                class:bg-base-300={$page.url.pathname === item.path}
              >
                <svg class="w-6 h-6" viewBox="0 0 24 24">
                  <path fill="currentColor" d={item.icon} />
                </svg>
                <span>{item.label}</span>
              </button>
            </li>
          {/if}
        {/each}
        <li class="mt-auto">
          <button
            on:click={openRebootDialog}
            class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200 text-error"
          >
            <svg class="w-6 h-6" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiPowerStandby} />
            </svg>
            <span>Reboot Firewall</span>
          </button>
        </li>
        <li>
          <button
            on:click={handleLogout}
            class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
          >
            <svg class="w-6 h-6" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiLogout} />
            </svg>
            <span>Logout</span>
          </button>
        </li>
      </ul>
    </nav>
  </aside>

  <!-- Main content -->
  <div class="flex-1 flex flex-col overflow-hidden">
    <!-- Top navbar - using fixed-header class -->
    <header class="bg-base-100 border-b border-base-300 fixed-header">
      <div class="flex items-center justify-between p-4">
        <div class="flex items-center space-x-4">
          <button
            type="button"
            class="lg:hidden p-1 rounded-md hover:bg-base-200 transition-colors duration-200"
            on:click={toggleSidebar}
            on:keydown={handleKeydown}
            aria-label="Toggle sidebar"
          >
            <svg class="w-6 h-6" viewBox="0 0 24 24">
              <path fill="currentColor" d={mdiMenu} />
            </svg>
          </button>
          <h1 class="text-xl font-semibold">{title}</h1>
        </div>
        <!-- Theme toggle button -->
        <button
          type="button"
          class="p-1 rounded-md hover:bg-base-200 transition-colors duration-200 theme-toggle-btn"
          on:click={toggleTheme}
          aria-label="Toggle theme"
        >
          <svg class="w-6 h-6" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiThemeLightDark} />
          </svg>
        </button>
      </div>
    </header>

    <!-- Page content - using page-content class -->
    <main class="flex-1 overflow-y-auto bg-base-200 p-6 page-content">
      <slot></slot>
    </main>
  </div>

  <!-- Mobile sidebar -->
  {#if isSidebarOpen}
    <div
      class="fixed inset-0 z-50 lg:hidden"
      on:click={toggleSidebar}
      on:keydown={handleOverlayKeydown}
      tabindex="0"
      role="button"
      aria-label="Close sidebar"
    >
      <div class="absolute inset-0 bg-base-300 opacity-75"></div>
    </div>

    <aside class="fixed inset-y-0 left-0 z-50 w-64 bg-base-100 lg:hidden">
      <div class="flex items-center justify-center h-16 bg-primary">
        <span class="text-xl font-bold text-primary-content">{title}</span>
      </div>
      <nav class="mt-5 overflow-y-auto" style="max-height: calc(100% - 64px)">
        <ul class="p-2 space-y-2">
          {#each menuItems as item}
            {#if item.category}
              <li>
                <button
                  on:click={() => toggleCategory(item.category)}
                  class="flex items-center justify-between w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
                  class:bg-base-300={isInCategory(item) ||
                    expandedCategories[item.category]}
                  aria-expanded={expandedCategories[item.category]}
                >
                  <div class="flex items-center space-x-3">
                    <svg class="w-6 h-6" viewBox="0 0 24 24">
                      <path fill="currentColor" d={item.icon} />
                    </svg>
                    <span>{item.label}</span>
                  </div>
                  <svg class="w-5 h-5" viewBox="0 0 24 24">
                    <path
                      fill="currentColor"
                      d={expandedCategories[item.category]
                        ? mdiChevronUp
                        : mdiChevronDown}
                    />
                  </svg>
                </button>
                {#if expandedCategories[item.category]}
                  <ul class="pl-8 mt-1 space-y-1">
                    {#each item.items as subItem}
                      <li>
                        <button
                          on:click={() => handleNavigation(subItem.path)}
                          class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
                          class:bg-base-300={$page.url.pathname ===
                            subItem.path}
                        >
                          <svg class="w-5 h-5" viewBox="0 0 24 24">
                            <path fill="currentColor" d={subItem.icon} />
                          </svg>
                          <span>{subItem.label}</span>
                        </button>
                      </li>
                    {/each}
                  </ul>
                {/if}
              </li>
            {:else}
              <li>
                <button
                  on:click={() => handleNavigation(item.path)}
                  class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
                  class:bg-base-300={$page.url.pathname === item.path}
                >
                  <svg class="w-6 h-6" viewBox="0 0 24 24">
                    <path fill="currentColor" d={item.icon} />
                  </svg>
                  <span>{item.label}</span>
                </button>
              </li>
            {/if}
          {/each}
          <li class="mt-auto">
            <button
              on:click={openRebootDialog}
              class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200 text-error"
            >
              <svg class="w-6 h-6" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiPowerStandby} />
              </svg>
              <span>Reboot Firewall</span>
            </button>
          </li>
          <li>
            <button
              on:click={handleLogout}
              class="flex items-center w-full p-2 space-x-3 rounded-md hover:bg-base-200 transition-colors duration-200"
            >
              <svg class="w-6 h-6" viewBox="0 0 24 24">
                <path fill="currentColor" d={mdiLogout} />
              </svg>
              <span>Logout</span>
            </button>
          </li>
        </ul>
      </nav>
    </aside>
  {/if}

  <!-- Reboot Confirmation Dialog -->
  {#if isRebootDialogOpen}
    <div
      class="fixed inset-0 z-50 overflow-y-auto"
      aria-labelledby="modal-title"
      role="dialog"
      aria-modal="true"
    >
      <div
        class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0"
      >
        <div
          class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
          aria-hidden="true"
        ></div>
        <span
          class="hidden sm:inline-block sm:align-middle sm:h-screen"
          aria-hidden="true">&#8203;</span
        >
        <div
          class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full dark:bg-gray-800"
        >
          <div class="bg-white dark:bg-gray-800 px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
            <div class="sm:flex sm:items-start">
              <div
                class="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10 dark:bg-red-900"
              >
                <svg
                  class="h-6 w-6 text-red-600 dark:text-red-400"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  aria-hidden="true"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
              </div>
              <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
                <h3
                  class="text-lg leading-6 font-medium text-gray-900 dark:text-white"
                  id="modal-title"
                >
                  Reboot Firewall
                </h3>
                <div class="mt-2">
                  <p class="text-sm text-gray-500 dark:text-gray-400">
                    Are you sure you want to reboot the firewall? This action
                    cannot be undone.
                  </p>
                </div>
              </div>
            </div>
          </div>
          <div
            class="bg-gray-50 dark:bg-gray-700 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse"
          >
            <button
              type="button"
              class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm"
              on:click={handleReboot}
            >
              Reboot
            </button>
            <button
              type="button"
              class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm dark:bg-gray-800 dark:text-gray-200 dark:border-gray-600 dark:hover:bg-gray-700"
              on:click={closeRebootDialog}
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* iOS scrolling fixes */
  @supports (-webkit-touch-callout: none) {
    /* Add extra padding at the bottom of page content to help with "reachability" */
    .page-content {
      padding-bottom: 80px !important;
    }

    /* Ensure the fixed header doesn't disappear during scroll */
    .fixed-header {
      position: sticky;
      top: 0;
      z-index: 10;
      backdrop-filter: blur(8px);
    }

    /* Prevent "bounce" effect on the main layout container */
    #app-layout-container {
      height: 100%;
      overflow: hidden;
    }

    /* Adjusted scrolling for the main content area */
    .page-content {
      -webkit-overflow-scrolling: touch;
      overscroll-behavior: none;
    }

    /* Fix the height in iOS */
    .min-h-screen {
      min-height: -webkit-fill-available;
    }
  }

  /* Original styles */
  .btn-circle {
    @apply rounded-full w-14 h-14 p-0 grid place-items-center;
  }

  .btn-lg {
    @apply w-16 h-16;
  }
</style>
