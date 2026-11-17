<script>
  import { onMount } from 'svelte';
  import { getJSON } from './api.js';
  let items = [];
  let failed = false;
  onMount(async () => {
    try {
      const page = await getJSON('/api/v1/recommendations?surface=web');
      items = page.items.slice(0, 6);
    } catch { failed = true; }
  });
</script>

<section class="card" aria-label="Suggested feeds">
  <h2>Worth a look</h2>
  {#if failed}<p class="muted">Suggestions are unavailable right now.</p>{/if}
  <ul>
    {#each items as item (item.uri)}
      <li><a href={`/b/${encodeURIComponent(item.uri)}`}>{item.name}</a> — {item.why}</li>
    {/each}
  </ul>
</section>
