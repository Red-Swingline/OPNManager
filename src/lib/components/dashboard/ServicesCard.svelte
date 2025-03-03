<script lang="ts">
    import { 
      mdiServerNetwork, 
      mdiChevronUp, 
      mdiChevronDown,
      mdiRestart 
    } from '@mdi/js';
  
    export let services: any[];
    export let expandedServices: boolean;
    export let toggleServicesExpansion: () => void;
    export let restartService: (serviceId: string) => Promise<void>;
  </script>
  
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <div class="flex justify-between items-center">
        <h3 class="card-title text-lg flex items-center gap-2">
          <svg class="w-5 h-5 text-secondary" viewBox="0 0 24 24">
            <path fill="currentColor" d={mdiServerNetwork} />
          </svg>
          Services
          <span class="badge badge-sm ml-2">
            {services.length}
          </span>
        </h3>
        <button 
          class="btn btn-ghost btn-sm" 
          on:click={toggleServicesExpansion}
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24">
            <path 
              fill="currentColor" 
              d={expandedServices ? mdiChevronUp : mdiChevronDown} 
            />
          </svg>
        </button>
      </div>
  
      {#if expandedServices}
        <div class="divider my-2"></div>
        <div class="space-y-3">
          {#each services as service}
            <div 
              class="border rounded-lg p-3 
              {service.running ? 'bg-success/10 border-success/30' : 'bg-error/10 border-error/30'}"
            >
              <div class="flex justify-between items-center">
                <div>
                  <div class="font-medium">{service.name}</div>
                  <div class="text-sm opacity-70">{service.description}</div>
                </div>
                <div class="flex items-center space-x-2">
                  <span 
                    class="badge {service.running ? 'badge-success' : 'badge-error'}"
                  >
                    {service.running ? 'Running' : 'Stopped'}
                  </span>
                  <button 
                    class="btn btn-ghost btn-sm" 
                    on:click={() => restartService(service.id)}
                    title="Restart Service"
                  >
                    <svg class="w-5 h-5" viewBox="0 0 24 24">
                      <path fill="currentColor" d={mdiRestart} />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>