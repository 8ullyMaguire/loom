<script>
  import { page } from '$app/stores';
  import { getJSON } from '$lib/api.js';
  let bundle = null;
  let error = '';
  $effect(() => {
    const id = $page.params.id;
    getJSON(`/api/v1/bundles/${id}`).then((b) => { bundle = b; }).catch((e) => { error = String(e); });
  });
</script>
<svelte:head><title>Bundle — Loom</title></svelte:head>
<h1>{bundle ? bundle.name : 'Bundle'}</h1>
{#if error}<p role="alert">{error}</p>{/if}
{#if bundle}
  <p class="muted">{bundle.description}</p>
  <ol>
    {#each bundle.feeds as uri}
      <li><a href={uri}>{uri}</a></li>
    {/each}
  </ol>
{/if}
