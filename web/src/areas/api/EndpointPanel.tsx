import { Badge, Button, Heading, Tabs, Text, TextArea, TextField } from '@radix-ui/themes';
import { Play } from 'lucide-react';
import { useId, useState } from 'react';
import { CSRF_COOKIE } from '../../api';
import { CSRF_HEADER, UI_HEADER } from '../../api/client';
import { readCookie, useT } from '../../i18n';
import { CodeBlock } from '../../ui';
import styles from './Api.module.css';
import {
  bodySchema,
  type Endpoint,
  initialValue,
  type OpenApi,
  requestUrl,
  schemaName,
  skeleton,
} from './model';

const SAFE = new Set(['GET', 'HEAD', 'OPTIONS']);
const REASONS: Record<number, string> = {
  200: 'OK',
  201: 'Created',
  202: 'Accepted',
  204: 'No Content',
  302: 'Found',
  304: 'Not Modified',
  400: 'Bad Request',
  401: 'Unauthorized',
  403: 'Forbidden',
  404: 'Not Found',
  405: 'Method Not Allowed',
  409: 'Conflict',
  415: 'Unsupported Media Type',
  422: 'Unprocessable Entity',
  429: 'Too Many Requests',
  500: 'Internal Server Error',
  503: 'Service Unavailable',
};

export function MethodBadge({ method, size = '1' }: { method: string; size?: '1' | '2' }) {
  return (
    <Badge size={size} variant="soft" radius="small" className={styles.method}>
      {method}
    </Badge>
  );
}

interface Result {
  status: number;
  statusText: string;
  contentType: string;
  body: string;
  millis: number;
}

export function EndpointPanel({ endpoint, document }: { endpoint: Endpoint; document: OpenApi }) {
  const t = useT();
  return (
    <article className={styles.panel} aria-labelledby="api-endpoint-title">
      <header className={styles.panelHead}>
        <MethodBadge method={endpoint.method} size="2" />
        <Heading as="h2" size="5" weight="medium" id="api-endpoint-title" className={styles.panelPath}>
          {endpoint.fullPath}
        </Heading>
      </header>
      {endpoint.summary && (
        <Text as="p" size="2" className={styles.summary}>
          {endpoint.summary}
        </Text>
      )}
      <Tabs.Root defaultValue="try">
        <Tabs.List>
          <Tabs.Trigger value="try">{t('api.tabs.try')}</Tabs.Trigger>
        </Tabs.List>
        <Tabs.Content value="try" className={styles.tab}>
          <TryIt endpoint={endpoint} document={document} />
        </Tabs.Content>
      </Tabs.Root>
    </article>
  );
}

function TryIt({ endpoint, document }: { endpoint: Endpoint; document: OpenApi }) {
  const t = useT();
  const id = useId();
  const schemas = document.components?.schemas ?? {};
  const parameters = (endpoint.operation.parameters ?? []).filter((parameter) => parameter.in !== 'cookie');
  const body = bodySchema(endpoint.operation);
  const [values, setValues] = useState<Record<string, string>>(() =>
    Object.fromEntries(parameters.map((parameter) => [parameter.name, initialValue(parameter)])),
  );
  const [text, setText] = useState(() =>
    body ? JSON.stringify(skeleton(body.schema, schemas), null, 2) : '',
  );
  const [problem, setProblem] = useState<string | null>(null);
  const [result, setResult] = useState<Result | null>(null);
  const [running, setRunning] = useState(false);

  const execute = async () => {
    setProblem(null);
    const { url, missing } = requestUrl(endpoint, values);
    if (missing.length > 0) {
      setProblem(t('api.try.missing', { names: missing.join(', ') }));
      return;
    }
    const headers = new Headers({ [UI_HEADER]: 'true', Accept: 'application/json, text/plain, */*' });
    for (const parameter of parameters.filter((candidate) => candidate.in === 'header')) {
      const value = values[parameter.name];
      if (value) {
        headers.set(parameter.name, value);
      }
    }
    if (!SAFE.has(endpoint.method)) {
      const csrf = readCookie(CSRF_COOKIE);
      if (csrf) {
        headers.set(CSRF_HEADER, csrf);
      }
    }
    let payload: string | undefined;
    if (body && text.trim() !== '') {
      try {
        payload = JSON.stringify(JSON.parse(text));
      } catch {
        setProblem(t('api.try.invalidJson'));
        return;
      }
      headers.set('Content-Type', body.type);
    }
    setRunning(true);
    const started = performance.now();
    try {
      const response = await fetch(url, {
        method: endpoint.method,
        headers,
        body: payload,
        credentials: 'same-origin',
      });
      const raw = endpoint.method === 'HEAD' ? '' : await response.text();
      const contentType = response.headers.get('content-type') ?? '';
      setResult({
        status: response.status,
        statusText: response.statusText || REASONS[response.status] || '',
        contentType,
        body: pretty(raw, contentType),
        millis: Math.round(performance.now() - started),
      });
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
    } finally {
      setRunning(false);
    }
  };

  const responses = Object.entries(endpoint.operation.responses ?? {});
  return (
    <div className={styles.try}>
      <section aria-labelledby={`${id}-parameters`}>
        <Text as="div" size="2" weight="bold" id={`${id}-parameters`} className={styles.sectionTitle}>
          {t('api.try.parameters')}
        </Text>
        {parameters.length === 0 ? (
          <Text as="p" size="2" color="gray">
            {t('api.try.noParameters')}
          </Text>
        ) : (
          <table className={styles.table}>
            <thead>
              <tr>
                <th>{t('api.try.name')}</th>
                <th>{t('api.try.value')}</th>
              </tr>
            </thead>
            <tbody>
              {parameters.map((parameter) => (
                <tr key={`${parameter.in}:${parameter.name}`}>
                  <td>
                    <div className={styles.paramName}>
                      {parameter.name}
                      {parameter.required && <span className={styles.required}>*</span>}
                    </div>
                    <Text as="div" size="1" color="gray">
                      {schemaName(parameter.schema) ?? 'string'} · {location(t, parameter.in)}
                    </Text>
                  </td>
                  <td>
                    <TextField.Root
                      size="2"
                      aria-label={parameter.name}
                      value={values[parameter.name] ?? ''}
                      onChange={(event) =>
                        setValues((current) => ({ ...current, [parameter.name]: event.target.value }))
                      }
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
      {body && (
        <section aria-labelledby={`${id}-body`}>
          <Text as="div" size="2" weight="bold" id={`${id}-body`} className={styles.sectionTitle}>
            {t('api.try.body')}{' '}
            <Text size="1" color="gray" weight="regular">
              {body.type}
              {schemaName(body.schema) ? ` · ${schemaName(body.schema)}` : ''}
            </Text>
          </Text>
          <TextArea
            aria-labelledby={`${id}-body`}
            value={text}
            rows={Math.min(16, Math.max(4, text.split('\n').length))}
            className={styles.bodyInput}
            onChange={(event) => setText(event.target.value)}
          />
        </section>
      )}
      <div className={styles.actions}>
        <Button size="2" onClick={() => void execute()} disabled={running}>
          <Play size={14} aria-hidden="true" />
          {t('api.try.execute')}
        </Button>
        {problem && (
          <Text size="2" color="red" role="alert">
            {problem}
          </Text>
        )}
      </div>
      {result && (
        <section aria-labelledby={`${id}-result`} className={styles.result}>
          <Text as="div" size="2" weight="bold" id={`${id}-result`} className={styles.sectionTitle}>
            {t('api.try.response')}
          </Text>
          <div className={styles.status}>
            <Badge
              size="2"
              variant="soft"
              color={result.status < 300 ? 'green' : result.status < 400 ? 'gray' : 'red'}
            >
              {result.status} {result.statusText}
            </Badge>
            <Text size="1" color="gray">
              {t('api.try.took', { millis: result.millis })}
              {result.contentType ? ` · ${result.contentType}` : ''}
            </Text>
          </div>
          {result.body !== '' ? (
            <CodeBlock code={result.body} copyLabel={t('api.try.copy')} />
          ) : (
            <Text as="p" size="2" color="gray">
              {t('api.try.emptyBody')}
            </Text>
          )}
        </section>
      )}
      {responses.length > 0 && (
        <section aria-labelledby={`${id}-responses`}>
          <Text as="div" size="2" weight="bold" id={`${id}-responses`} className={styles.sectionTitle}>
            {t('api.try.responses')}
          </Text>
          <table className={styles.table}>
            <thead>
              <tr>
                <th>{t('api.try.code')}</th>
                <th>{t('api.try.schema')}</th>
              </tr>
            </thead>
            <tbody>
              {responses.map(([code, response]) => {
                const content = response.content ?? {};
                const media = Object.keys(content)[0];
                const name = media ? schemaName(content[media]?.schema) : null;
                return (
                  <tr key={code}>
                    <td className={styles.code}>{code}</td>
                    <td>
                      {name ? (
                        <span className={styles.schema}>
                          {name}
                          <Text size="1" color="gray">
                            {' '}
                            {media}
                          </Text>
                        </span>
                      ) : (
                        <Text size="2" color="gray">
                          {t('api.try.noContent')}
                        </Text>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </section>
      )}
    </div>
  );
}

function location(t: ReturnType<typeof useT>, place: string): string {
  switch (place) {
    case 'path':
      return t('api.try.in.path');
    case 'header':
      return t('api.try.in.header');
    default:
      return t('api.try.in.query');
  }
}

function pretty(raw: string, contentType: string): string {
  if (contentType.includes('json') && raw !== '') {
    try {
      return JSON.stringify(JSON.parse(raw), null, 2);
    } catch {
      return raw;
    }
  }
  return raw;
}
