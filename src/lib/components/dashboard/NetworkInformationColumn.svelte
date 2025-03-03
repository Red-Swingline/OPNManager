<script lang="ts">
    import GatewaysCard from './GatewaysCard.svelte';
    import InterfaceTrafficCard from './InterfaceTrafficCard.svelte';
  
    export let gatewayStatus: any[];
    export let expandedGateway: string | null;
    export let toggleGatewayExpansion: (gatewayName: string) => void;
    export let sortedInterfaces: [string, any][];
    export let formatBytes: (bytes: string) => string;
  </script>
  
  <div class="space-y-6 order-last lg:order-first">
    <!-- Gateways Card -->
    <GatewaysCard 
      {gatewayStatus}
      {expandedGateway}
      {toggleGatewayExpansion}
    />
  
    <!-- Interface Traffic Card -->
    <InterfaceTrafficCard 
      {sortedInterfaces}
      {formatBytes}
    />
  </div>