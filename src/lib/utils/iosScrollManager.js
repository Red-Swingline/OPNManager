/**
 * Manages iOS-specific scrolling behaviors to fix common issues
 */
export function setupIOSScrolling() {
    const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window).MSStream;
    
    if (!isIOS) return;

    let appContainer = document.getElementById('app-container');

    if (!appContainer) {
      console.warn('App container not found, iOS scroll fixes may not work properly');
      return;
    }

    document.body.style.overflow = 'hidden';
    document.body.style.position = 'fixed';
    document.body.style.width = '100%';
    document.body.style.height = '100%';
    document.body.style.top = '0';
    document.body.style.left = '0';
    appContainer.style.height = '100%';
    appContainer.style.overflowY = 'auto';
    appContainer.style.position = 'absolute';
    appContainer.style.width = '100%';
    appContainer.style.webkitOverflowScrolling = 'touch';
    appContainer.style.overscrollBehavior = 'none';

    const ensureBottomReachable = () => {
      const pageContents = document.querySelectorAll('.page-content');
      pageContents.forEach(pageContent => {
        pageContent.style.paddingBottom = '60px';
      });
    };

    ensureBottomReachable();

    window.addEventListener('resize', ensureBottomReachable);

    const smoothScrollTo = (element) => {
      if (!element) return;
      
      const elementTop = element.getBoundingClientRect().top;
      const offsetPosition = elementTop + appContainer.scrollTop;
      
      appContainer.scrollTo({
        top: offsetPosition,
        behavior: 'smooth'
      });
    };

    return {
      scrollTo: smoothScrollTo,
      refreshLayout: ensureBottomReachable,
      cleanup: () => {
        window.removeEventListener('resize', ensureBottomReachable);
      }
    };
  }