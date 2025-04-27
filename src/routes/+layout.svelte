<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import "../app.css";
  import Toast from '$lib/components/Toast.svelte';
  import { preventIOSInputScroll } from "$lib/utils/iosFocusFix";
  import { authStore } from "$lib/stores/authStore";
  import { registerLogoutCleanup } from "$lib/utils/dashboardCleanup";

  // Holds the unsubscribe function from the auth store
  let authUnsubscribe;

  onMount(() => {
    const cleanup = preventIOSInputScroll();

    // Register the logout cleanup function to clean up dashboard resources on logout
    authUnsubscribe = registerLogoutCleanup(authStore);

    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window).MSStream;
    
    if (isIOS) {
      document.body.style.overflow = 'hidden';
      document.body.style.position = 'fixed';
      document.body.style.width = '100%';
      document.body.style.height = '100%';

      const appContainer = document.getElementById('app-container');
      if (appContainer) {
        appContainer.style.height = '100%';
        appContainer.style.overflowY = 'auto';
        appContainer.style.position = 'absolute';
        appContainer.style.width = '100%';
        appContainer.style.webkitOverflowScrolling = 'touch';
      }
    }
    
    return () => {
      if (cleanup) cleanup();
    };
  });
  
  onDestroy(() => {
    // Unsubscribe from the auth store when the layout is destroyed
    if (authUnsubscribe) {
      authUnsubscribe();
    }
  });
</script>

<div id="app-container">
  <Toast />
  <slot />
</div>