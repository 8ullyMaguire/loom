<script>
  import { onMount } from 'svelte';
  import '../app.css';
  import Recommendations from '$lib/Recommendations.svelte';
  let { children } = $props();
  onMount(() => {
    if ('serviceWorker' in navigator) navigator.serviceWorker.register('/sw.js').catch(() => {});
  });
</script>

<header class="card">
  <a href="/"><strong>Loom</strong></a>
  <nav>
    <a href="/search">Search</a>
    <a href="/tags">Tags</a>
  </nav>
</header>
<main>
  {@render children()}
  <Recommendations />
</main>
<footer class="muted">A directory of Bluesky feeds, made by readers.</footer>
