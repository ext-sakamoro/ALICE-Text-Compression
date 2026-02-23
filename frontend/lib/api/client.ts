const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, {
    ...options,
    headers: { 'Content-Type': 'application/json', ...options?.headers },
  });
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

export const api = {
  health: () => request<{ status: string; version: string; uptime_secs: number }>('/health'),
  compress: (body: { text: string; algorithm?: string; level?: number }) =>
    request('/api/v1/text/compress', { method: 'POST', body: JSON.stringify(body) }),
  decompress: (body: { data: string; algorithm?: string }) =>
    request('/api/v1/text/decompress', { method: 'POST', body: JSON.stringify(body) }),
  analyze: (body: { text: string }) =>
    request('/api/v1/text/analyze', { method: 'POST', body: JSON.stringify(body) }),
  batch: (body: { texts: string[]; algorithm?: string }) =>
    request('/api/v1/text/batch', { method: 'POST', body: JSON.stringify(body) }),
  algorithms: () => request('/api/v1/text/algorithms'),
  stats: () => request('/api/v1/text/stats'),
};
