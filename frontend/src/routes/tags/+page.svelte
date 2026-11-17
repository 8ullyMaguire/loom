<script>
  import { postJSON, getJSON } from '$lib/api.js';
  let feed = '';
  let tag = '';
  let confidence = 80;
  let result = null;
  let error = '';
  async function submit() {
    try {
      await postJSON('/api/v1/tags', { feed, tag, confidence: Number(confidence) });
      result = await getJSON(`/api/v1/tags?feed=${encodeURIComponent(feed)}&tag=${encodeURIComponent(tag)}`);
      error = '';
    } catch (e) { error = String(e); }
  }
</script>
<svelte:head><title>Tag a feed — Loom</title></svelte:head>
<h1>Tag a feed</h1>
<p class="muted">Your confidence is one opinion. A number is only shown once several readers agree.</p>
<form onsubmit={(e) => { e.preventDefault(); submit(); }}>
  <input bind:value={feed} placeholder="at://…" aria-label="Feed AT URI" />
  <input bind:value={tag} placeholder="tag" aria-label="Tag" />
  <input bind:value={confidence} type="number" min="0" max="100" aria-label="Confidence" />
  <button type="submit">Add tag</button>
</form>
{#if error}<p role="alert">{error}</p>{/if}
{#if result}<p class="card">{result.status}{result.n !== undefined ? ` · ${result.n} votes` : ''}</p>{/if}
