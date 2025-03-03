<script lang="ts">
    import { mdiEthernet } from '@mdi/js';
  
    export let sortedInterfaces: [string, any][];
    export let formatBytes: (bytes: string) => string;
  </script>
  
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h3 class="card-title text-lg flex items-center gap-2">
        <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24">
          <path fill="currentColor" d={mdiEthernet} />
        </svg>
        Interface Traffic
      </h3>
  
      <div class="divider my-2"></div>
  
      <div class="overflow-x-auto">
        <table class="table table-zebra w-full">
          <thead>
            <tr>
              <th>Interface</th>
              <th>Received</th>
              <th>Transmitted</th>
            </tr>
          </thead>
          <tbody>
            {#each sortedInterfaces as [key, interfaceData]}
              <tr>
                <td class="font-medium">
                  <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full bg-accent"></div>
                    {interfaceData.name}
                  </div>
                  <div class="text-xs opacity-50">
                    {interfaceData.device}
                  </div>
                </td>
                <td>
                  <span class="font-mono"
                    >{formatBytes(
                      interfaceData["bytes received"]
                    )}</span
                  >
                </td>
                <td>
                  <span class="font-mono"
                    >{formatBytes(
                      interfaceData["bytes transmitted"]
                    )}</span
                  >
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>