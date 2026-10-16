export function getSocketUrl(): string {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = import.meta.env.VITE_DALE_HOST || "127.0.0.1:8000";
  return `${protocol}//${host}/ws`;
}

export async function playAlbum(id: string, offset: number = 0): Promise<unknown> {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play/${encodedId}?offset=${offset}`, { method: "POST" });
  return await response.json();
}

export async function playDisc(
  id: string,
  discNumber: number,
  offset: number = 0
): Promise<unknown> {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/play-disc/${encodedId}?disc=${discNumber}&offset=${offset}`, {
    method: "POST"
  });
  return await response.json();
}

export async function queueAlbum(id: string): Promise<unknown> {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/queue/${encodedId}`, { method: "POST" });
  return await response.json();
}

export async function jumpToQueueIndex(index: string | number): Promise<unknown> {
  const response = await fetch(`/api/jump/${index}`, { method: "POST" });
  return await response.json();
}

export async function executeAction(
  name: string,
  id: string,
  extraParams: Record<string, string> = {}
): Promise<unknown> {
  const params = new URLSearchParams({ id, ...extraParams });
  const response = await fetch(`/api/actions/${name}?${params.toString()}`, { method: "POST" });
  return await response.json();
}

export async function updateAlbum(id: string): Promise<unknown> {
  const encodedId = encodeURIComponent(id);
  const response = await fetch(`/api/update-album/${encodedId}`, { method: "POST" });
  return await response.json();
}
