<script lang="ts">
    import MemoryUsageCard from './MemoryUsageCard.svelte';
    import DiskUsageCard from './DiskUsageCard.svelte';
    import ServicesCard from './ServicesCard.svelte';
  
    export let systemResources: any;
    export let mainDisk: any;
    export let services: any[];
    export let expandedServices: boolean;
    export let toggleServicesExpansion: () => void;
    export let restartService: (serviceId: string) => Promise<void>;
  </script>
  
  <div class="space-y-6 order-first lg:order-last">
    <!-- Memory Usage Card -->
    {#if systemResources}
      <MemoryUsageCard {systemResources} />
    {/if}
  
    <!-- Disk Usage Card -->
    {#if mainDisk}
      <DiskUsageCard {mainDisk} />
    {/if}
  
    <!-- Services Card -->
    <ServicesCard 
      {services}
      {expandedServices}
      {toggleServicesExpansion}
      {restartService}
    />
  </div>