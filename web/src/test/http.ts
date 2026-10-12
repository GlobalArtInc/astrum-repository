export function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

export type Handler = (request: {
  method: string;
  path: string;
  body: unknown;
  headers: Headers;
}) => Response;

export function fakeServer(handlers: Record<string, Handler>) {
  const calls: Array<{ method: string; path: string; body: unknown }> = [];
  const fetch = async (input: RequestInfo | URL, init: RequestInit = {}) => {
    const method = (init.method ?? 'GET').toUpperCase();
    const path = String(input);
    const body = typeof init.body === 'string' ? JSON.parse(init.body) : init.body;
    calls.push({ method, path, body });
    const handler = handlers[`${method} ${path}`];
    if (!handler) {
      return jsonResponse({ id: '*', message: `no handler for ${method} ${path}` }, 404);
    }
    return handler({ method, path, body, headers: new Headers(init.headers) });
  };
  return { fetch, calls };
}
