import { ApiError, CSRF_COOKIE, withQuery } from '../../api';
import { errorFromBody } from '../../api/errors';
import { readCookie } from '../../i18n';
import { COMPONENTS } from './model';

export interface Progress {
  loaded: number;
  total: number;
}

function parse(text: string, type: string): unknown {
  if (type.includes('json') && text !== '') {
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }
  return text;
}

export function uploadComponent(
  repository: string,
  data: FormData,
  onProgress: (progress: Progress) => void,
  signal?: AbortSignal,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const request = new XMLHttpRequest();
    request.open('POST', withQuery(COMPONENTS, { repository }));
    request.withCredentials = true;
    request.setRequestHeader('X-Astrum-UI', 'true');
    request.setRequestHeader('Accept', 'application/json');
    const csrf = readCookie(CSRF_COOKIE);
    if (csrf) {
      request.setRequestHeader('X-Astrum-CSRF', csrf);
    }
    request.upload.onprogress = (event) => {
      onProgress({ loaded: event.loaded, total: event.lengthComputable ? event.total : 0 });
    };
    request.onload = () => {
      if (request.status >= 200 && request.status < 300) {
        resolve();
        return;
      }
      const type = request.getResponseHeader('content-type') ?? '';
      reject(errorFromBody(request.status, request.statusText, parse(request.responseText, type)));
    };
    request.onerror = () => reject(new ApiError(0, 'network error'));
    request.onabort = () => reject(new DOMException('The upload was cancelled', 'AbortError'));
    if (signal) {
      if (signal.aborted) {
        request.abort();
        return;
      }
      signal.addEventListener('abort', () => request.abort(), { once: true });
    }
    request.send(data);
  });
}

export async function sha1Of(file: File, limit = 256 * 1024 * 1024): Promise<string | null> {
  if (file.size > limit || typeof crypto === 'undefined' || !crypto.subtle) {
    return null;
  }
  try {
    const digest = await crypto.subtle.digest('SHA-1', await file.arrayBuffer());
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, '0')).join('');
  } catch {
    return null;
  }
}
