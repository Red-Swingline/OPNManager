<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import Toast from '$lib/components/Toast.svelte';
  import { preventIOSInputScroll } from "$lib/utils/iosFocusFix";

  // Add an event listener to handle iOS-specific scroll behaviors
  onMount(() => {
    // Initialize iOS focus fix
    const cleanup = preventIOSInputScroll();
    
    // Handle iOS-specific behaviors
    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window).MSStream;
    
    if (isIOS) {
      // Prevent body scrolling on iOS
      document.body.style.overflow = 'hidden';
      document.body.style.position = 'fixed';
      document.body.style.width = '100%';
      document.body.style.height = '100%';
      
      // Ensure the app container has the right height
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
</script>

<div id="app-container">
  <Toast />
  <slot />
</div>