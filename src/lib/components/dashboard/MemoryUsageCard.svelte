<script lang="ts">
    import { mdiServer } from '@mdi/js';
  
    export let systemResources: { 
      memory: { 
        used: number; 
        total: string; 
        used_frmt: string; 
        total_frmt: string; 
      } 
    };
  </script>
  
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h3 class="card-title text-lg flex items-center gap-2">
        <svg class="w-5 h-5 text-indigo-500" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiServer} />
        </svg>
        Memory Usage
      </h3>
      <div class="divider my-2"></div>
      <div class="flex items-center justify-between">
        <div 
          class="radial-progress text-indigo-500" 
          style="--value:{Math.round((systemResources.memory.used / parseInt(systemResources.memory.total)) * 100)}; --size:4rem; --thickness: 8px;" 
          role="progressbar"
        >
          {Math.round((systemResources.memory.used / parseInt(systemResources.memory.total)) * 100)}%
        </div>
        <div>
          <p class="text-sm">
            <span class="font-semibold">Used:</span> 
            {systemResources.memory.used_frmt}
          </p>
          <p class="text-sm">
            <span class="font-semibold">Total:</span> 
            {systemResources.memory.total_frmt}
          </p>
        </div>
      </div>
    </div>
  </div>