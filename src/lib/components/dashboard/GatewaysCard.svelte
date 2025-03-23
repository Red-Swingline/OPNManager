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
            class="flex items-center w-full cursor-pointer"
            on:click={() => toggleGatewayExpansion(gateway.name)}
          >
            <!-- Left side with gateway info (will take available space and truncate) -->
            <div class="flex-1 min-w-0 mr-2">
              <div class="font-medium flex items-center">
                <span
                  class="inline-block w-2 h-2 rounded-full mr-2 flex-shrink-0 {gateway.status_translated === 'Online' ? 'bg-success' : 'bg-error'}"
                ></span>
                <!-- Important: truncate class on the text itself -->
                <span class="truncate" title={gateway.name}>{gateway.name}</span>
              </div>
              <div class="text-sm opacity-70 truncate" title={gateway.address}>
                {gateway.address}
              </div>
            </div>
            
            <!-- Right side with status badge and toggle (won't shrink) -->
            <div class="flex items-center gap-2 flex-shrink-0">
              <span
                class="badge badge-sm whitespace-nowrap {gateway.status_translated === 'Online' ? 'badge-success' : 'badge-error'}"
              >
                {gateway.status_translated}
              </span>
              <svg class="w-5 h-5" viewBox="0 0 24 24">
                <path
                  fill="currentColor"
                  d={expandedGateway === gateway.name ? mdiChevronUp : mdiChevronDown}
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
                <span class="ml-1">{gateway.delay === "~" ? "-" : gateway.delay}</span>
              </div>
              <div>
                <span class="font-medium">RTTd:</span>
                <span class="ml-1">{gateway.stddev === "~" ? "-" : gateway.stddev}</span>
              </div>
              <div>
                <span class="font-medium">Loss:</span>
                <span class="ml-1">{gateway.loss === "~" ? "-" : gateway.loss}</span>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  /* Make sure truncation works */
  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }
</style>