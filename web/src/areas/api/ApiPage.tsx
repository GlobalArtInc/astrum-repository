import { IconButton, Select, Text, TextField, Tooltip } from '@radix-ui/themes';
import { CircleHelp, ExternalLink, PanelLeftClose, PanelLeftOpen } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useSearchParams } from 'react-router';
import { useApi } from '../../api';
import { useT } from '../../i18n';
import { ErrorState, Page } from '../../ui';
import styles from './Api.module.css';
import { EndpointPanel, MethodBadge } from './EndpointPanel';
import {
  ALL,
  DOCS,
  DOCUMENT,
  type Endpoint,
  endpointKey,
  endpoints,
  filterEndpoints,
  groupEndpoints,
  METHODS,
  type OpenApi,
  tagsOf,
} from './model';

export default function ApiPage() {
  const t = useT();
  const document = useApi<OpenApi>(DOCUMENT);
  const [params, setParams] = useSearchParams();
  const [query, setQuery] = useState('');
  const [method, setMethod] = useState<string>(ALL);
  const [tag, setTag] = useState<string>(ALL);
  const [collapsed, setCollapsed] = useState(false);
  const all = useMemo(() => (document.data ? endpoints(document.data) : []), [document.data]);
  const tags = useMemo(() => tagsOf(all), [all]);
  const groups = useMemo(
    () => groupEndpoints(filterEndpoints(all, { query, method, tag })),
    [all, query, method, tag],
  );
  const selectedKey =
    params.get('method') && params.get('path')
      ? endpointKey(params.get('method') ?? '', params.get('path') ?? '')
      : null;
  const selected = all.find((endpoint) => endpoint.key === selectedKey) ?? null;
  const select = (endpoint: Endpoint) =>
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        next.set('method', endpoint.method);
        next.set('path', endpoint.path);
        return next;
      },
      { replace: true },
    );
  const toggleLabel = collapsed ? t('api.list.expand') : t('api.list.collapse');
  return (
    <Page
      gutter="small"
      title={t('api.page.title')}
      description={t('api.page.description')}
      breadcrumbs={[{ label: t('api.crumbs.settings'), to: '/settings' }, { label: t('api.page.title') }]}
    >
      {document.error ? (
        <ErrorState error={document.error} onRetry={() => void document.refetch()} />
      ) : (
        <div className={styles.stack}>
          <div className={styles.layout} data-collapsed={collapsed || undefined}>
            {!collapsed && (
              <nav className={styles.list} aria-label={t('api.list.label')}>
                <TextField.Root
                  size="2"
                  value={query}
                  placeholder={t('api.list.search')}
                  aria-label={t('api.list.search')}
                  onChange={(event) => setQuery(event.target.value)}
                />
                <div className={styles.filters}>
                  <FilterSelect
                    label={t('api.list.method')}
                    value={method}
                    onChange={setMethod}
                    options={METHODS.map((name) => ({ value: name, label: name }))}
                  />
                  <FilterSelect
                    label={t('api.list.tag')}
                    value={tag}
                    onChange={setTag}
                    options={tags.map((name) => ({ value: name, label: name }))}
                  />
                </div>
                <div className={styles.scroller}>
                  {document.isPending && (
                    <Text size="2" color="gray">
                      {t('api.list.loading')}
                    </Text>
                  )}
                  {!document.isPending && groups.length === 0 && (
                    <Text size="2" color="gray">
                      {t('api.list.none')}
                    </Text>
                  )}
                  {groups.map((group) => (
                    <section key={group.tag ?? ''} className={styles.group}>
                      <Text as="div" size="1" weight="bold" className={styles.tag}>
                        {group.tag ?? t('api.list.untagged')}
                      </Text>
                      <ul className={styles.rows}>
                        {group.endpoints.map((endpoint) => (
                          <li key={endpoint.key}>
                            <button
                              type="button"
                              className={styles.row}
                              aria-current={endpoint.key === selectedKey || undefined}
                              title={endpoint.summary || endpoint.fullPath}
                              onClick={() => select(endpoint)}
                            >
                              <MethodBadge method={endpoint.method} />
                              <span className={styles.path}>{endpoint.fullPath}</span>
                            </button>
                          </li>
                        ))}
                      </ul>
                    </section>
                  ))}
                </div>
              </nav>
            )}
            <div className={styles.toggle}>
              <Tooltip content={toggleLabel}>
                <IconButton
                  size="1"
                  variant="soft"
                  aria-label={toggleLabel}
                  aria-expanded={!collapsed}
                  onClick={() => setCollapsed(!collapsed)}
                >
                  {collapsed ? <PanelLeftOpen size={14} /> : <PanelLeftClose size={14} />}
                </IconButton>
              </Tooltip>
            </div>
            <div className={styles.detail}>
              {selected && document.data ? (
                <EndpointPanel key={selected.key} endpoint={selected} document={document.data} />
              ) : (
                <Intro />
              )}
            </div>
          </div>
          <section className={styles.help} aria-labelledby="api-help-title">
            <Text as="div" size="2" weight="medium" id="api-help-title" className={styles.helpTitle}>
              <CircleHelp size={14} aria-hidden="true" />
              {t('api.about.title')}
            </Text>
            <Text as="p" size="2" className={styles.helpText}>
              {t('api.about.text')}
            </Text>
            <Text as="p" size="2" className={styles.helpText}>
              {t.rich('api.about.more', {
                link: (
                  <a href={DOCS} target="_blank" rel="noreferrer" className={styles.external}>
                    {t('api.about.docs')}
                    <ExternalLink size={12} aria-hidden="true" />
                  </a>
                ),
              })}
            </Text>
          </section>
        </div>
      )}
    </Page>
  );
}

function Intro() {
  const t = useT();
  return (
    <div className={styles.intro}>
      <Text as="div" size="3" weight="medium" className={styles.introTitle}>
        {t('api.intro.title')}
      </Text>
      <Text as="div" size="2" color="gray">
        {t('api.intro.text')}
      </Text>
      <ul className={styles.introList}>
        <li>
          <Text size="2">{t.rich('api.intro.try', { name: <strong>{t('api.tabs.try')}</strong> })}</Text>
        </li>
      </ul>
    </div>
  );
}

function FilterSelect({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: string;
  onChange(value: string): void;
  options: Array<{ value: string; label: string }>;
}) {
  const t = useT();
  return (
    <div className={styles.filter}>
      <Text as="div" size="1" weight="medium" aria-hidden="true">
        {label}
      </Text>
      <Select.Root size="2" value={value} onValueChange={onChange}>
        <Select.Trigger variant="surface" aria-label={label} className={styles.trigger} />
        <Select.Content position="popper">
          <Select.Item value={ALL}>{t('api.list.all')}</Select.Item>
          {options.map((option) => (
            <Select.Item key={option.value} value={option.value}>
              {option.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    </div>
  );
}
