// src/lib/utils/iosFocusFix.js

/**
 * Prevents iOS Safari from scrolling when input fields are focused
 * Uses the "opacity trick" discovered in the wild
 */
export function preventIOSInputScroll() {
    // Only needed for iOS devices
    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window).MSStream;
    
    if (!isIOS) return;
    
    // Add a focus event listener to the document
    const handleFocus = (e) => {
      const target = e.target;
      
      // Only apply to input elements, textareas, and select elements
      if (['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)) {
        // Store the current opacity value
        const originalOpacity = target.style.opacity;
        
        // Briefly set opacity to 0
        target.style.opacity = '0';
        
        // Then restore it in the next tick
        setTimeout(() => {
          target.style.opacity = originalOpacity || '1';
        }, 0);
      }
    };
    
    // Add event listener
    document.addEventListener('focus', handleFocus, true);
    
    // Return cleanup function
    return () => {
      document.removeEventListener('focus', handleFocus, true);
    };
  }