export const API = import.meta.env.VITE_LOOM_API || '';

export async function getJSON(path) {
  const response = await fetch(`${API}${path}`, { headers: { accept: 'application/json' } });
  if (!response.ok) throw new Error(`${response.status} ${path}`);
  return response.json();
}

export async function postJSON(path, body) {
  const response = await fetch(`${API}${path}`, {
    method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body)
  });
  if (!response.ok) throw new Error(`${response.status} ${path}`);
  return response.json();
}
