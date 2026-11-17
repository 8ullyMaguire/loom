<script>
  import { getJSON } from '$lib/api.js';
  let q = '';
  let items = [];
  let error = '';
  let searched = false;
  async function run() {
    try {
      const page = await getJSON(`/api/v1/search?q=${encodeURIComponent(q)}`);
      items = page.items; error = ''; searched = true;
    } catch (e) { error = String(e); }
  }
</script>
<svelte:head><title>Search feeds — Loom</title></svelte:head>
<h1>Search feeds</h1>
<form onsubmit={(e) => { e.preventDefault(); run(); }}>
  <input bind:value={q} placeholder="language, topic, name" aria-label="Search feeds" />
  <button type="submit">Search</button>
</form>
{#if error}<p role="alert">{error}</p>{/if}
{#if searched && items.length === 0}<p class="muted">No feeds matched.</p>{/if}
{#each items as item (item.uri)}
  <article class="card"><h2>{item.name}</h2><p>{item.description}</p>
    <a href={`/b/${encodeURIComponent(item.uri)}`}>Open in a bundle</a></article>
{/each}
