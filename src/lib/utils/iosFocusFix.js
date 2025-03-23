/**
 * Prevents iOS Safari from scrolling when input fields are focused
 * Uses the "opacity trick" discovered in the wild
 */
export function preventIOSInputScroll() {
    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window).MSStream;
    
    if (!isIOS) return;
    const handleFocus = (e) => {
      const target = e.target;
      if (['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)) {
        const originalOpacity = target.style.opacity;
        target.style.opacity = '0';

        setTimeout(() => {
          target.style.opacity = originalOpacity || '1';
        }, 0);
      }
    };

    document.addEventListener('focus', handleFocus, true);

    return () => {
      document.removeEventListener('focus', handleFocus, true);
    };
  }