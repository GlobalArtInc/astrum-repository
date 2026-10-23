import { CSRF_COOKIE } from '../../api';
import { CSRF_HEADER, UI_HEADER } from '../../api/client';
import { readCookie } from '../../i18n';

export const READ = 'astrum:settings:read';
export const DOCUMENT = '/service/rest/swagger.json';

const SAFE_METHODS = new Set(['GET', 'HEAD', 'OPTIONS']);

export interface SwaggerRequest {
  url: string;
  method?: string;
  headers: Record<string, string>;
}

export function withSession(request: SwaggerRequest): SwaggerRequest {
  request.headers[UI_HEADER] = 'true';
  if (!SAFE_METHODS.has((request.method ?? 'GET').toUpperCase())) {
    const csrf = readCookie(CSRF_COOKIE);
    if (csrf) {
      request.headers[CSRF_HEADER] = csrf;
    }
  }
  return request;
}
