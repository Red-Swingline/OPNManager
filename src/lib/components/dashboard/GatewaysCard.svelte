<script lang="ts">
    import { 
      mdiRouter, 
      mdiChevronUp, 
      mdiChevronDown 
    } from '@mdi/js';
  
    export let gatewayStatus: any[];
    export let expandedGateway: string | null;
    export let toggleGatewayExpansion: (gatewayName: string) => void;
  </script>
  
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h3 class="card-title text-lg flex items-center gap-2">
        <svg class="w-5 h-5 text-primary" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiRouter} />
        </svg>
        Gateways
      </h3>
  
      <div class="divider my-2"></div>
  
      <div class="space-y-3">
        {#each gatewayStatus as gateway}
          <div class="border rounded-lg p-3">
            <div
              class="flex justify-between items-center cursor-pointer"
              on:click={() => toggleGatewayExpansion(gateway.name)}
            >
              <div>
                <div class="font-medium flex items-center">
                  <span
                    class={`inline-block w-2 h-2 rounded-full mr-2 ${gateway.status_translated === "Online" ? "bg-success" : "bg-error"}`}
                  ></span>
                  {gateway.name}
                </div>
                <div class="text-sm opacity-70">
                  {gateway.address}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <span
                  class="badge badge-sm {gateway.status_translated === 'Online'
                    ? 'badge-success'
                    : 'badge-error'}"
                >
                  {gateway.status_translated}
                </span>
                <svg class="w-5 h-5" viewBox="0 0 24 24">
                  <path
                    fill="currentColor"
                    d={expandedGateway === gateway.name
                      ? mdiChevronUp
                      : mdiChevronDown}
                  />
                </svg>
              </div>
            </div>
            {#if expandedGateway === gateway.name}
              <div
                class="mt-3 text-sm grid grid-cols-3 gap-2 pt-2 border-t border-base-300"
              >
                <div>
                  <span class="font-medium">RTT:</span>
                  <span class="ml-1"
                    >{gateway.delay === "~"
                      ? "-"
                      : gateway.delay}</span
                  >
                </div>
                <div>
                  <span class="font-medium">RTTd:</span>
                  <span class="ml-1"
                    >{gateway.stddev === "~"
                      ? "-"
                      : gateway.stddev}</span
                  >
                </div>
                <div>
                  <span class="font-medium">Loss:</span>
                  <span class="ml-1"
                    >{gateway.loss === "~" ? "-" : gateway.loss}</span
                  >
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>