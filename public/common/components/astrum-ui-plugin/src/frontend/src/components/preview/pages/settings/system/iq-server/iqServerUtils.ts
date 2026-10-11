

import { IqServerConfiguration, IqValidationErrors, PASSWORD_PLACEHOLDER } from './types';
import IqServerStrings from '../../../../constants/pages/settings/system/IqServerStrings';

/** validateIqConfig doesn't read `properties`, so it accepts either the wire shape
 *  (IqServerConfiguration, string properties) or the form shape (IqServerFormData,
 *  array properties) — both are structurally assignable once `properties` is omitted. */
type IqConfigForValidation = Omit<IqServerConfiguration, 'properties'>;

const URL_HOSTNAME_REGEX = /^(([a-z0-9]|[a-z0-9][a-z0-9-]*[a-z0-9])\.)*([a-z0-9]|[a-z0-9][a-z0-9-]*[a-z0-9])$/i;
const URL_PATHNAME_REGEX = /^([\S]*\S)?$/i;

export function isValidUrl(url: string): boolean {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }
  const hostnameMatch = url.match(/^https?:\/\/([^:/?#]+)/i);
  const hostname = hostnameMatch?.[1];
  const { protocol, pathname, port } = parsed;
  return (
    (protocol === 'http:' || protocol === 'https:') &&
    Boolean(hostname) &&
    URL_HOSTNAME_REGEX.test(hostname!) &&
    port !== '0' &&
    URL_PATHNAME_REGEX.test(decodeURIComponent(pathname))
  );
}

/**
 * Validates IQ Server configuration and returns validation errors
 */
export function validateIqConfig(config: IqConfigForValidation, pristineConfig: IqConfigForValidation): IqValidationErrors {
  const errors: IqValidationErrors = {};

  if (!config.url?.trim()) {
    errors.url = IqServerStrings.UTILS.IQ_SERVER_URL_REQUIRED;
  } else if (!isValidUrl(config.url)) {
    errors.url = IqServerStrings.UTILS.PLEASE_ENTER_VALID_URL;
  }

  if (!config.authenticationType) {
    errors.authenticationType = IqServerStrings.UTILS.AUTHENTICATION_METHOD_REQUIRED;
  }

  if (config.authenticationType === 'USER') {
    if (!config.username?.trim()) {
      errors.username = IqServerStrings.UTILS.USERNAME_REQUIRED;
    }

    // Password is required for new configs or when URL changes
    const urlChanged = pristineConfig.url && pristineConfig.url !== config.url;
    const isPlaceholder = config.password === PASSWORD_PLACEHOLDER;

    if (!config.password?.trim() && !isPlaceholder) {
      errors.password = IqServerStrings.UTILS.PASSWORD_REQUIRED;
    } else if (urlChanged && isPlaceholder) {
      errors.password = IqServerStrings.UTILS.PASSWORD_REQUIRED_WHEN_CHANGING_URL;
    }
  }

  if (config.timeoutSeconds !== null && (config.timeoutSeconds < 1 || config.timeoutSeconds > 3600)) {
    errors.timeoutSeconds = IqServerStrings.UTILS.TIMEOUT_MUST_BETWEEN_1_3600;
  }

  return errors;
}

/**
 * Parses the verification reason into application names or a status message.
 * Backend returns either comma-separated app names or messages like "No applications configured yet."
 */
export function parseApplicationReason(reason: string): { isList: boolean; items: string[] } {
  const trimmed = reason.trim();
  const appsPrefix = 'Applications: ';
  if (trimmed.includes(appsPrefix)) {
    const after = trimmed.split(appsPrefix)[1]?.trim() ?? '';
    const items = after.split(',').map((s) => s.trim()).filter(Boolean);
    return { isList: items.length > 0, items };
  }
  if (
    trimmed.toLowerCase().startsWith('no applications') ||
    (trimmed.startsWith('Connection successful') && !trimmed.includes(','))
  ) {
    return { isList: false, items: [trimmed] };
  }
  const items = trimmed.split(',').map((s) => s.trim()).filter(Boolean);
  return { isList: items.length > 0, items };
}

/** "Connected v1.2.3" -> " (v1.2.3)"; no match -> "". Mirrors the inline regex used today. */
export function parseVersion(reason?: string): string {
  const m = reason?.match(/v?(\d+\.\d+(?:\.\d+)?)/);
  return m ? ` (v${m[1]})` : '';
}


/**
 * Best-effort error-message extraction from an Axios error or REST client
 * error. Handles single strings, JSON-encoded strings, JSR-303 validation
 * arrays, and `{message}` objects. Falls back to the provided default.
 */
export function formatErrorMessage(err: any, fallback: string): string {
  const raw = err?.response?.data ?? err?.message ?? err;
  let parsed: any = raw;
  if (typeof raw === 'string') {
    const trimmed = raw.trim();
    if (trimmed.startsWith('[') || trimmed.startsWith('{')) {
      try {
        parsed = JSON.parse(trimmed);
      } catch {
        /* fall through */
      }
    }
  }
  let text: string;
  if (Array.isArray(parsed)) {
    text = parsed.map((e: any) => e?.message || JSON.stringify(e)).join(' ');
  } else if (typeof parsed === 'string') {
    text = parsed;
  } else if (parsed?.message) {
    text = parsed.message;
  } else {
    return fallback;
  }
  return humanizeIqError(text) || fallback;
}

// Peel Java toString / HttpClient noise from verify errors and map to plain user copy.
function humanizeIqError(raw: string): string {
  let text = raw.trim();

  // Peel ValidationErrorXO{id='...', message='...'} → the inner message.
  const xo = text.match(/ValidationErrorXO\{[^}]*message='([^']*)'/);
  if (xo) text = xo[1];

  // Strip the "[hostname/ip, hostname/ip]" resolver dump that
  // org.apache.http adds after the target: "Connect to localhost:8073 [localhost/127.0.0.1, ...] failed: ..."
  text = text.replace(/\s*\[[^\]]*(?:\/[^\]]*)+\]/g, '');

  // Pattern-match common root causes and rewrite as user-facing copy.
  const connectMatch = text.match(/Connect to ([^\s]+) failed: (.+)$/i);
  if (connectMatch) {
    const [, target, cause] = connectMatch;
    const causeLc = cause.toLowerCase();
    if (causeLc.includes('connection refused')) {
      return IqServerStrings.UTILS.CANNOT_REACH_CONNECTION_REFUSED_CHECK(target);
    }
    if (causeLc.includes('timed out') || causeLc.includes('timeout')) {
      return IqServerStrings.UTILS.CANNOT_REACH_CONNECTION_TIMED_OUT(target);
    }
    return IqServerStrings.UTILS.CANNOT_REACH(target, cause);
  }
  if (/UnknownHostException|unknown host|nodename nor servname/i.test(text)) {
    const host = text.match(/([\w.-]+)(?::\d+)?/)?.[1];
    return host
      ? IqServerStrings.UTILS.HOST_NOT_FOUND_CHECK_URL(host)
      : IqServerStrings.UTILS.HOST_NOT_FOUND_CHECK_URL_2;
  }
  if (/SSLHandshake|PKIX|certificate/i.test(text)) {
    return IqServerStrings.UTILS.SSL_CERTIFICATE_COULD_NOT_VERIFIED;
  }
  if (/401|unauthorized|invalid credentials/i.test(text)) {
    return IqServerStrings.UTILS.AUTHENTICATION_FAILED_CHECK_USERNAME_PASSWORD;
  }
  if (/403|forbidden/i.test(text)) {
    return IqServerStrings.UTILS.IQ_SERVER_USER_DOES_NOT;
  }
  return text;
}

/**
 * Convert an UPPER_SNAKE_CASE policy stage value (e.g. "STAGE_RELEASE") to a
 * human-readable label ("Stage Release") for display.
 */
export function humanizeStage(stage: string | null | undefined): string {
  if (!stage) return '';
  return stage
    .split('_')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join(' ');
}

