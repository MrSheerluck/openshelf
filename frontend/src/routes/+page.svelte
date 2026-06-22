<script lang="ts">
  import * as auth from "$lib/auth.svelte.ts";
</script>

<svelte:head>
  <title>Library - OpenShelf</title>
</svelte:head>

<div class="app">
  <aside class="sidebar">
    <h1>OpenShelf</h1>
    <nav>
      <a href="/" class="active">Library</a>
    </nav>
    <div class="spacer"></div>
    <button onclick={() => auth.logout()}>Sign out</button>
  </aside>
  <main class="content">
    <div class="empty-state">
      <h2>Upload your first book</h2>
      <p>Drag and drop an EPUB file or click to browse.</p>
      <input type="file" accept=".epub" disabled />
    </div>
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    font-family: system-ui, sans-serif;
  }
  .sidebar {
    width: 240px;
    background: #f9fafb;
    border-right: 1px solid #e5e7eb;
    display: flex;
    flex-direction: column;
    padding: 1.5rem 1rem;
  }
  .sidebar h1 {
    font-size: 1.25rem;
    margin: 0 0 2rem 0;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  nav a {
    text-decoration: none;
    color: #111;
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    font-size: 0.9rem;
  }
  nav a.active {
    background: #e5e7eb;
    font-weight: 500;
  }
  .spacer {
    flex: 1;
  }
  .sidebar button {
    background: none;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    padding: 0.4rem;
    cursor: pointer;
    font-size: 0.85rem;
    color: #555;
  }
  .content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .empty-state {
    text-align: center;
    color: #888;
  }
  .empty-state h2 {
    color: #333;
    margin-bottom: 0.5rem;
  }
  .empty-state p {
    margin-bottom: 1.5rem;
  }
</style>
