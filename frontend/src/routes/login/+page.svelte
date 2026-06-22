<script lang="ts">
  import * as auth from "$lib/auth.svelte.ts";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let password = $state("");
  let error = $state("");
  let isFirstRun = $state(true);
  let loading = $state(false);

  onMount(async () => {
    isFirstRun = !(await auth.hasUsers());
  });

  $effect(() => {
    if (auth.isAuthenticated()) {
      goto("/");
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = "";
    loading = true;
    const ok = await auth.login(password);
    if (ok) {
      goto("/");
    } else {
      error = isFirstRun ? "Failed to create account" : "Invalid password";
    }
    loading = false;
  }
</script>

<svelte:head>
  <title>Login - OpenShelf</title>
</svelte:head>

<main>
  <h1>OpenShelf</h1>
  <form onsubmit={handleSubmit}>
    {#if isFirstRun}
      <p class="hint">Set your admin password to get started.</p>
    {/if}
    <input
      type="password"
      bind:value={password}
      placeholder="Password"
      class="input"
      disabled={loading}
    />
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <button type="submit" class="btn" disabled={loading || !password}>
      {loading ? "Please wait..." : isFirstRun ? "Create account" : "Sign in"}
    </button>
  </form>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    font-family: system-ui, sans-serif;
  }
  h1 {
    font-size: 2rem;
    margin-bottom: 2rem;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 280px;
  }
  .hint {
    font-size: 0.9rem;
    color: #666;
    margin: 0;
    text-align: center;
  }
  .input {
    padding: 0.6rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 1rem;
  }
  .btn {
    padding: 0.6rem;
    background: #111;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    cursor: pointer;
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .error {
    color: #dc2626;
    font-size: 0.85rem;
    margin: 0;
  }
</style>
