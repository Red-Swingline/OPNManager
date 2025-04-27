// logWorker.ts - Web worker for processing firewall logs off the main thread
interface FirewallLog {
  rulenr?: string;
  interface?: string;
  src?: string;
  dst?: string;
  srcport?: string;
  dstport?: string;
  protoname?: string;
  action?: string;
  __timestamp__?: string;
  label?: string;
  digest?: string;
  dir?: string;
}

interface LogFilters {
  action: string;
  interface: string;
  direction: string;
}

// Handle incoming messages from main thread
self.onmessage = (event) => {
  if (event.data.type === 'processlogs') {
    try {
      console.log("Web worker received processlogs message");
      
      // Start performance measurement
      const startTime = performance.now();
      
      // Extract data from the message
      const { logs, currentLogs, filters, limit } = event.data;
      console.log(`Processing ${logs.length} logs in worker with filters:`, filters);
      
      // Process logs in the worker thread
      const processedLogs = processLogs(logs, currentLogs, filters, limit);
      
      // End performance measurement
      const endTime = performance.now();
      const processingTime = endTime - startTime;
      console.log(`Log processing took ${processingTime.toFixed(2)}ms in worker, returning ${processedLogs.length} logs`);
      
      // Send processed logs back to main thread
      self.postMessage({ 
        type: 'processedlogs', 
        logs: processedLogs,
        processingTime: processingTime
      });
    } catch (error) {
      console.error("Error in web worker:", error);
      // Report error back to main thread
      self.postMessage({ 
        type: 'error', 
        error: error.toString()
      });
    }
  }
};

// Add error handler for uncaught exceptions
self.addEventListener('error', (event) => {
  console.error('Worker error:', event.message, event.error);
});

/**
 * Process logs in a separate thread to avoid blocking the main UI
 * 
 * @param newLogs - New logs received from backend
 * @param currentLogs - Current logs already in view (unused in current implementation)
 * @param filters - User-selected filters
 * @param limit - Maximum number of logs to return
 * @returns Filtered, sorted, and limited logs
 */
function processLogs(newLogs: FirewallLog[], currentLogs: FirewallLog[], filters: LogFilters, limit: number): FirewallLog[] {
  // We're using just the new logs directly from the backend
  let allLogs = [...newLogs];
  
  // Apply filters if any are set - optimize by returning early if no filters
  if (filters.action || filters.interface || filters.direction) {
    allLogs = allLogs.filter(log => {
      return (!filters.action || log.action === filters.action) &&
             (!filters.interface || log.interface === filters.interface) &&
             (!filters.direction || (
               // Check both label and dir field for direction
               (log.dir && log.dir === filters.direction) || 
               (log.label && log.label.includes(filters.direction))
             ));
    });
  }

  // Sort logs by timestamp (newest first)
  if (allLogs.length > 1) {
    allLogs.sort((a, b) => {
      const dateA = a.__timestamp__ ? new Date(a.__timestamp__).getTime() : 0;
      const dateB = b.__timestamp__ ? new Date(b.__timestamp__).getTime() : 0;
      return dateB - dateA;
    });
  }

  // Limit the number of logs (avoid unnecessary slice if already within limit)
  return allLogs.length > limit ? allLogs.slice(0, limit) : allLogs;
}