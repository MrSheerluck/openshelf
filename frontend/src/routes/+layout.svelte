<script lang="ts">
  import * as auth from "$lib/auth.svelte.ts";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  let { children } = $props();
  let currentPath = $derived(String(page.url.pathname));

  $effect(() => {
    auth.checkAuth();
  });

  $effect(() => {
    if (
      !auth.isLoading() &&
      !auth.isAuthenticated() &&
      currentPath !== "/login"
    ) {
      goto("/login");
    }
  });
</script>

{#if currentPath === "/login"}
  {@render children()}
{:else if auth.isLoading()}
  <div class="loading">Loading...</div>
{:else if auth.isAuthenticated()}
  {@render children()}
{/if}

<style>
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    font-family: system-ui, sans-serif;
    color: #888;
  }
</style>
