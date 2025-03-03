<script lang="ts">
    import { mdiHarddisk } from '@mdi/js';
  
    export let mainDisk: { 
      used_pct: number; 
      used: string; 
      available: string; 
      mountpoint: string; 
    };
  </script>
  
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h3 class="card-title text-lg flex items-center gap-2">
        <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiHarddisk} />
        </svg>
        Disk Usage
      </h3>
      <div class="divider my-2"></div>
      <div class="flex items-center justify-between">
        <div 
          class="radial-progress text-accent" 
          style="--value:{mainDisk.used_pct}; --size:4rem; --thickness: 8px;" 
          role="progressbar"
        >
          {mainDisk.used_pct}%
        </div>
        <div>
          <p class="text-sm">
            <span class="font-semibold">Used:</span> 
            {mainDisk.used}
          </p>
          <p class="text-sm">
            <span class="font-semibold">Available:</span> 
            {mainDisk.available}
          </p>
          <p class="text-sm">
            <span class="font-semibold">Mountpoint:</span> 
            {mainDisk.mountpoint}
          </p>
        </div>
      </div>
    </div>
  </div>