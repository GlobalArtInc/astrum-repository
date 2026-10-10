

import type { SwaggerRequest, SwaggerResponse } from './types';

export function getCsrfToken(): string {
  const match = document.cookie.match('(^|; )AS-ANTI-CSRF-TOKEN=([^;]*)');
  return match ? match[2] : '';
}

export function swaggerRequestInterceptor(request: SwaggerRequest): SwaggerRequest {
  const csrfToken = getCsrfToken();
  if (csrfToken) {
    request.headers['AS-ANTI-CSRF-TOKEN'] = csrfToken;
  }
  request.headers['X-Astrum-UI'] = 'true';
  return request;
}

export function swaggerResponseInterceptor(response: SwaggerResponse): SwaggerResponse {
  let data = response.data;

  if (typeof response.data === 'string') {
    try {
      data = JSON.parse(response.data);
    } catch (e) {
      console.error('Failed to parse Swagger response:', e);
    }
  }

  if (data && typeof data === 'object' && 'tags' in data) {
    const dataObj = data as { tags?: Array<{ name: string }> };
    if (Array.isArray(dataObj.tags)) {
      const sortedTags = [...dataObj.tags].sort((a, b) => a.name.localeCompare(b.name));
      response.body.tags = sortedTags;
      response.data = { ...dataObj, tags: sortedTags };

      try {
        const text = JSON.parse(response.text);
        response.text = JSON.stringify({ ...text, tags: sortedTags });
      } catch {
        // ignore
      }
    }
  }

  return response;
}
