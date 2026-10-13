import {
  Button,
  Dialog,
  Flex,
  IconButton,
  TextField as RadixTextField,
  Select,
  Spinner,
} from '@radix-ui/themes';
import { Ban, Check, ChevronDown, ChevronRight, Database, Info, ListTree, Search, X } from 'lucide-react';
import { type FormEvent, useState } from 'react';
import { ApiError, api, path, useApi, withQuery } from '../../api';
import { useT } from '../../i18n';
import { describeError, Page } from '../../ui';
import {
  countNodes,
  type Filter,
  LIST_PAGE,
  normalizePath,
  PREVIEW,
  type PreviewNode,
  type RoutingRuleDetails,
  UI_RULES,
} from './model';
import styles from './Routing.module.css';
import { ModeLabel } from './RoutingRulesPage';

const FILTERS: Array<{ value: Filter; label: string }> = [
  { value: 'all', label: 'routing.preview.filters.all' },
  { value: 'groups', label: 'routing.preview.filters.groups' },
  { value: 'proxies', label: 'routing.preview.filters.proxies' },
];

const EXAMPLE = '/com/example/artifact-1.0.jar';

function RuleDialog({ name, path: tested, onClose }: { name: string; path: string; onClose(): void }) {
  const t = useT();
  const rule = useApi<RoutingRuleDetails>(`${UI_RULES}${path`/${name}`}`);
  return (
    <Dialog.Root open onOpenChange={(open) => !open && onClose()}>
      <Dialog.Content maxWidth="500px" aria-describedby={undefined}>
        <Flex justify="between" align="start" gap="3">
          <Dialog.Title>{name}</Dialog.Title>
          <Dialog.Close>
            <IconButton variant="soft" color="gray" aria-label={t('routing.actions.close')}>
              <X size={16} aria-hidden="true" />
            </IconButton>
          </Dialog.Close>
        </Flex>
        {rule.isPending ? (
          <Flex justify="center" py="6">
            <Spinner size="3" />
          </Flex>
        ) : rule.error ? (
          <p role="alert">
            {rule.error instanceof ApiError && rule.error.status === 404
              ? t('routing.preview.ruleMissing')
              : describeError(t, rule.error)}
          </p>
        ) : (
          rule.data && (
            <>
              <div className={styles.modalSection}>
                <span className={styles.modalLabel}>{t('routing.list.mode')}</span>
                <ModeLabel mode={rule.data.mode} />
              </div>
              <div className={styles.modalSection}>
                <span className={styles.modalLabel}>{t('routing.preview.matchedPath')}</span>
                <span className={styles.modalValue}>{tested}</span>
              </div>
              <div className={styles.modalPatterns}>
                <span className={styles.modalLabel}>{t('routing.preview.patterns')}</span>
                <div className={styles.modalHint}>{t('routing.preview.patternsText')}</div>
                <ul>
                  {rule.data.matchers.map((pattern) => (
                    <li key={pattern}>
                      <code className={styles.patternCode}>{pattern}</code>
                    </li>
                  ))}
                </ul>
              </div>
              <div className={styles.modalSection}>
                <span className={styles.modalLabel}>{t('routing.preview.ruleName')}</span>
                <span className={styles.modalValue}>{rule.data.name}</span>
              </div>
            </>
          )
        )}
        <Flex justify="end" mt="4">
          <Dialog.Close>
            <Button variant="soft" color="gray">
              {t('routing.actions.close')}
            </Button>
          </Dialog.Close>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function Nodes({
  nodes,
  depth,
  closed,
  toggle,
  openRule,
}: {
  nodes: readonly PreviewNode[];
  depth: number;
  closed: ReadonlySet<string>;
  toggle(key: string): void;
  openRule(name: string): void;
}) {
  const t = useT();
  return nodes.map((node, index) => {
    const name = node.repository ?? '';
    const key = `${depth}:${index}:${name}`;
    const group = (node.children?.length ?? 0) > 0 || node.type === 'group';
    const open = !closed.has(key);
    const Icon = node.type === 'group' ? ListTree : Database;
    return (
      <div key={key} role="listitem" aria-label={name}>
        <div
          className={styles.node}
          data-group={node.type === 'group'}
          style={{ paddingLeft: 16 + depth * 24 }}
        >
          {group ? (
            <button
              type="button"
              className={styles.toggle}
              aria-label={
                open ? t('routing.preview.collapse', { name }) : t('routing.preview.expand', { name })
              }
              onClick={() => toggle(key)}
            >
              {open ? (
                <ChevronDown size={16} aria-hidden="true" />
              ) : (
                <ChevronRight size={16} aria-hidden="true" />
              )}
            </button>
          ) : (
            <span className={styles.toggleSpace} />
          )}
          <Icon size={16} aria-hidden="true" className={styles.nodeIcon} />
          <span className={styles.nodeName}>{name}</span>
          {node.format && <span className={styles.format}>{node.format}</span>}
          {node.rule && (
            <button
              type="button"
              className={styles.ruleLink}
              aria-label={t('routing.preview.viewRule', { name: node.rule })}
              onClick={() => node.rule && openRule(node.rule)}
            >
              {node.rule}
            </button>
          )}
          <span className={styles.verdict} data-allowed={node.allowed !== false}>
            {node.allowed === false ? (
              <>
                <Ban size={14} aria-hidden="true" />
                {t('routing.preview.blocked')}
              </>
            ) : (
              <>
                <Check size={14} aria-hidden="true" />
                {t('routing.preview.allowed')}
              </>
            )}
          </span>
        </div>
        {group && open && node.children && (
          <div role="list">
            <Nodes
              nodes={node.children}
              depth={depth + 1}
              closed={closed}
              toggle={toggle}
              openRule={openRule}
            />
          </div>
        )}
      </div>
    );
  });
}

export default function PreviewPage() {
  const t = useT();
  const [input, setInput] = useState('');
  const [filter, setFilter] = useState<Filter>('all');
  const [result, setResult] = useState<{ path: string; root: PreviewNode } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [closed, setClosed] = useState<ReadonlySet<string>>(new Set());
  const [rule, setRule] = useState<string | null>(null);
  const run = async (event?: FormEvent) => {
    event?.preventDefault();
    const tested = normalizePath(input);
    if (tested === '') {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const root = await api.get<PreviewNode>(withQuery(PREVIEW, { path: tested, filter }));
      setResult({ path: tested, root });
      setClosed(new Set());
    } catch (failure) {
      setResult(null);
      setError(describeError(t, failure));
    } finally {
      setBusy(false);
    }
  };
  const nodes = result?.root.children ?? [];
  const counts = countNodes(nodes);
  return (
    <Page
      className={styles.page}
      title={t('routing.preview.title')}
      description={t('routing.preview.intro')}
      breadcrumbs={[
        { label: t('routing.crumbs.settings'), to: '/settings' },
        { label: t('routing.list.title'), to: LIST_PAGE },
        { label: t('routing.crumbs.preview') },
      ]}
    >
      <div className={styles.stack}>
        <form className={styles.previewBar} onSubmit={(event) => void run(event)}>
          <div className={styles.previewPath}>
            <label htmlFor="routing-preview-path" className={styles.previewLabel}>
              {t('routing.preview.path')}
            </label>
            <RadixTextField.Root
              id="routing-preview-path"
              className={styles.previewInput}
              value={input}
              placeholder={t('routing.preview.pathPlaceholder')}
              spellCheck={false}
              onChange={(event) => setInput(event.target.value)}
            >
              <RadixTextField.Slot>
                <Search size={16} aria-hidden="true" />
              </RadixTextField.Slot>
            </RadixTextField.Root>
          </div>
          <div className={styles.previewFilter}>
            <span id="routing-preview-filter" className={styles.previewLabel}>
              {t('routing.preview.filter')}
            </span>
            <Select.Root value={filter} onValueChange={(value) => setFilter(value as Filter)}>
              <Select.Trigger aria-labelledby="routing-preview-filter" />
              <Select.Content position="popper">
                {FILTERS.map((option) => (
                  <Select.Item key={option.value} value={option.value}>
                    {t(option.label)}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
          </div>
          <Button
            type="submit"
            size="3"
            className={styles.testButton}
            loading={busy}
            disabled={input.trim() === ''}
          >
            {t('routing.preview.test')}
          </Button>
        </form>
        {error && (
          <div className={styles.problem} role="alert">
            {error}
          </div>
        )}
        {result === null ? (
          <section className={styles.how} aria-labelledby="routing-how">
            <h2 id="routing-how" className={styles.helpTitle}>
              <Info size={16} aria-hidden="true" />
              {t('routing.preview.howTitle')}
            </h2>
            <p className={styles.helpText}>
              {t.rich('routing.preview.howText', {
                example: (
                  <code key="example" className={styles.exampleCode}>
                    {EXAMPLE}
                  </code>
                ),
              })}
            </p>
          </section>
        ) : (
          <>
            <div className={styles.counts} role="status">
              <div className={styles.count}>
                <span className={styles.countLabel}>{t('routing.preview.total')}</span>
                <span className={styles.countValue}>{counts.total}</span>
              </div>
              <div className={styles.count}>
                <span className={styles.countLabel}>{t('routing.preview.allowed')}</span>
                <span className={styles.countValue} data-tone="green">
                  {counts.allowed}
                </span>
              </div>
              <div className={styles.count}>
                <span className={styles.countLabel}>{t('routing.preview.blocked')}</span>
                <span className={styles.countValue} data-tone="red">
                  {counts.blocked}
                </span>
              </div>
            </div>
            {nodes.length === 0 ? (
              <div className={styles.empty}>{t('routing.preview.empty')}</div>
            ) : (
              <div className={styles.tree} role="list" aria-label={t('routing.preview.title')}>
                <Nodes
                  nodes={nodes}
                  depth={0}
                  closed={closed}
                  toggle={(key) =>
                    setClosed((current) => {
                      const next = new Set(current);
                      if (next.has(key)) {
                        next.delete(key);
                      } else {
                        next.add(key);
                      }
                      return next;
                    })
                  }
                  openRule={setRule}
                />
              </div>
            )}
          </>
        )}
      </div>
      {rule && result && <RuleDialog name={rule} path={result.path} onClose={() => setRule(null)} />}
    </Page>
  );
}
