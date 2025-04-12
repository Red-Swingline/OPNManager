<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    
    onMount(() => {
        goto('/routes/static');
    });
</script>

<!-- This is just a redirect page to /routes/static -->
<div class="flex justify-center items-center h-screen">
    <span class="loading loading-spinner loading-lg"></span>
    <p class="ml-4">Redirecting to Static Routes...</p>
</div>