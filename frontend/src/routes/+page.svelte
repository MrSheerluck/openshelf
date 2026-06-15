<script>
  let health = $state("Checking backend...");

  async function checkHealth() {
    const res = await fetch("http://localhost:3001/health");
    const data = await res.json();
    health = JSON.stringify(data, null, 2);
  }

  $effect(() => {
    checkHealth();
  });
</script>

<main>
  <h1>Hello from OpenShelf</h1>
  <pre>{health}</pre>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    font-family: system-ui, sans-serif;
  }
  h1 { font-size: 2.5rem; }
  pre {
    background: #f4f4f4;
    padding: 1rem 2rem;
    border-radius: 8px;
    font-size: 0.9rem;
  }
</style>
