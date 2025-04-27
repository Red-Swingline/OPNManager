export function preventIOSInputScroll() {
  const isIOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window).MSStream;
  
  if (!isIOS) return;
  
  // Track focused inputs to restore visibility if needed
  let inputElements = [];
  
  // Handle focus events
  const handleFocus = (e) => {
    const target = e.target;
    if (['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName)) {
      // Instead of opacity trick, use a small scroll adjustment
      setTimeout(() => {
        window.scrollBy(0, 1);
        window.scrollBy(0, -1);
      }, 0);
      
      // Add element to tracked list
      if (!inputElements.includes(target)) {
        inputElements.push(target);
      }
    }
  };
  
  // Handle app visibility changes
  const handleVisibilityChange = () => {
    if (document.visibilityState === 'visible') {
      // When app becomes visible again, ensure all inputs are visible
      inputElements.forEach(element => {
        if (element && element.style) {
          element.style.opacity = '1';
        }
      });
    }
  };

  document.addEventListener('focus', handleFocus, true);
  document.addEventListener('visibilitychange', handleVisibilityChange);

  return () => {
    document.removeEventListener('focus', handleFocus, true);
    document.removeEventListener('visibilitychange', handleVisibilityChange);
  };
}