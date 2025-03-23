// src/lib/utils/iosScrollManager.js

/**
 * Manages iOS-specific scrolling behaviors to fix common issues
 */
export function setupIOSScrolling() {
    // Only needed for iOS devices
    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window).MSStream;
    
    if (!isIOS) return;
    
    // Elements to set up
    let appContainer = document.getElementById('app-container');
    
    // Create container if it doesn't exist
    if (!appContainer) {
      console.warn('App container not found, iOS scroll fixes may not work properly');
      return;
    }
    
    // Prevent document body from scrolling
    document.body.style.overflow = 'hidden';
    document.body.style.position = 'fixed';
    document.body.style.width = '100%';
    document.body.style.height = '100%';
    document.body.style.top = '0';
    document.body.style.left = '0';
    
    // Make the app container scrollable
    appContainer.style.height = '100%';
    appContainer.style.overflowY = 'auto';
    appContainer.style.position = 'absolute';
    appContainer.style.width = '100%';
    appContainer.style.webkitOverflowScrolling = 'touch';
    appContainer.style.overscrollBehavior = 'none';
    
    // Ensure content at the bottom is reachable
    const ensureBottomReachable = () => {
      // Find all page content containers
      const pageContents = document.querySelectorAll('.page-content');
      pageContents.forEach(pageContent => {
        // Add padding to ensure bottom content is reachable
        pageContent.style.paddingBottom = '60px';
      });
    };
    
    // Run initially
    ensureBottomReachable();
    
    // Also run when the window resizes
    window.addEventListener('resize', ensureBottomReachable);
    
    // Scroll to specific elements without bouncing
    const smoothScrollTo = (element) => {
      if (!element) return;
      
      const elementTop = element.getBoundingClientRect().top;
      const offsetPosition = elementTop + appContainer.scrollTop;
      
      appContainer.scrollTo({
        top: offsetPosition,
        behavior: 'smooth'
      });
    };
    
    // Expose utility functions
    return {
      scrollTo: smoothScrollTo,
      refreshLayout: ensureBottomReachable,
      cleanup: () => {
        window.removeEventListener('resize', ensureBottomReachable);
      }
    };
  }