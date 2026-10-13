import {
  Button,
  Card,
  Code,
  IconButton,
  TextField as RadixTextField,
  Select,
  Table,
  Text,
  Tooltip,
} from '@radix-ui/themes';
import { Plus, Search, X } from 'lucide-react';
import { type FormEvent, type ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import { useT } from '../../i18n';
import { DateTime, FormatBadge, Page } from '../../ui';
import {
  blankCriterion,
  CRITERIA,
  type Criterion,
  criteriaPath,
  definition,
  OPERATORS,
  type Operator,
  readCriteria,
  writeCriteria,
} from './criteria';
import { lastModified, openPath, SEARCH, type SearchComponent } from './model';
import styles from './Search.module.css';
import { activate, LoadMore } from './SearchResults';
import { BATCH, type PagedSearch, usePagedSearch } from './usePagedSearch';

function CriterionRow({
  criterion,
  removable,
  onChange,
  onRemove,
}: {
  criterion: Criterion;
  removable: boolean;
  onChange(next: Criterion): void;
  onRemove(): void;
}) {
  const t = useT();
  const example = definition(criterion.key)?.example ?? '';
  const remove = (
    <IconButton
      type="button"
      size="2"
      variant="ghost"
      color="gray"
      style={{ boxSizing: 'content-box', width: 16, height: 16, padding: 6, margin: -6 }}
      aria-label={t('search.criteria.remove')}
      disabled={!removable}
      onClick={onRemove}
    >
      <X size={16} aria-hidden="true" />
    </IconButton>
  );
  return (
    <div className={styles.criterion}>
      <Select.Root size="2" value={criterion.key} onValueChange={(key) => onChange({ ...criterion, key })}>
        <Select.Trigger className={styles.criterionKey} aria-label={t('search.criteria.field')} />
        <Select.Content position="popper">
          {CRITERIA.map((candidate) => (
            <Select.Item key={candidate.key} value={candidate.key}>
              {t(candidate.label)}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
      <Select.Root
        size="2"
        value={criterion.operator}
        onValueChange={(operator) => onChange({ ...criterion, operator: operator as Operator })}
      >
        <Select.Trigger className={styles.criterionOperator} aria-label={t('search.criteria.operator')} />
        <Select.Content position="popper">
          {OPERATORS.map((operator) => (
            <Select.Item key={operator.value} value={operator.value}>
              {t(operator.label)}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
      <RadixTextField.Root
        size="2"
        className={styles.criterionValue}
        value={criterion.value}
        aria-label={t('search.criteria.value')}
        placeholder={t('search.criteria.example', { example })}
        autoComplete="off"
        onChange={(event) => onChange({ ...criterion, value: event.target.value })}
      />
      {removable ? (
        remove
      ) : (
        <Tooltip content={t('search.criteria.onlyOne')}>
          <span className={styles.criterionRemoveWrap}>{remove}</span>
        </Tooltip>
      )}
    </div>
  );
}

function Notice({
  icon,
  title,
  text,
  small,
}: {
  icon: number;
  title: string;
  text: string;
  small?: boolean;
}) {
  return (
    <div className={styles.notice}>
      <div className={styles.noticeBody}>
        <Search size={icon} color="var(--gray-8)" aria-hidden="true" />
        <Text size="3" className={styles.gray}>
          {title}
        </Text>
        <Text size={small ? '1' : '2'} align="center" className={styles.gray}>
          {text}
        </Text>
      </div>
    </div>
  );
}

function CriteriaResults({ results }: { results: PagedSearch<SearchComponent> }) {
  const t = useT();
  const navigate = useNavigate();
  if (results.error && !results.items?.length) {
    return (
      <div role="alert">
        <Notice icon={32} title={t('search.results.errorTitle')} text={results.error.message} small />
      </div>
    );
  }
  if (results.loading || results.items === undefined) {
    return (
      <div className={styles.loading} role="status">
        <Text size="2" className={styles.gray}>
          {t('search.results.loading')}
        </Text>
      </div>
    );
  }
  if (results.items.length === 0) {
    return (
      <Notice icon={32} title={t('search.criteria.emptyTitle')} text={t('search.criteria.emptyText')} small />
    );
  }
  return (
    <div className={styles.resultStack}>
      <div className={styles.status}>
        <Text size="2" className={styles.gray} role="status">
          {t('search.results.found', { count: results.items.length })}
        </Text>
      </div>
      <div className={styles.criteriaTableWrap}>
        <Table.Root variant="surface" size="2">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeaderCell>{t('search.results.format')}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{t('search.criteria.nameColumn')}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{t('search.criteria.groupColumn')}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{t('search.results.version')}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{t('search.results.repository')}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{t('search.results.updated')}</Table.ColumnHeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {results.items.map((component) => {
              const when = lastModified(component);
              const open = () => navigate(openPath(component));
              return (
                <Table.Row
                  key={component.id}
                  className={styles.row}
                  role="button"
                  tabIndex={0}
                  aria-label={t('search.criteria.open', { name: component.name })}
                  onClick={open}
                  onKeyDown={activate(open)}
                >
                  <Table.Cell>
                    <FormatBadge format={component.format} />
                  </Table.Cell>
                  <Table.Cell>
                    <Text weight="medium">{component.name}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="2" className={styles.gray}>
                      {component.group || '—'}
                    </Text>
                  </Table.Cell>
                  <Table.Cell>
                    {component.version ? <Code size="2">{component.version}</Code> : '—'}
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="2" className={styles.gray}>
                      {component.repository}
                    </Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="2" className={styles.gray}>
                      {when ? <DateTime value={when} dateOnly /> : '—'}
                    </Text>
                  </Table.Cell>
                </Table.Row>
              );
            })}
          </Table.Body>
        </Table.Root>
      </div>
      <LoadMore results={results} />
    </div>
  );
}

export default function CriteriaPage() {
  const t = useT();
  const [params, setParams] = useSearchParams();
  const urlText = params.toString();
  const fromUrl = useMemo(() => readCriteria(new URLSearchParams(urlText)), [urlText]);
  const [rows, setRows] = useState<Criterion[]>(() => (fromUrl.length > 0 ? fromUrl : [blankCriterion(1)]));
  const [everything, setEverything] = useState(false);
  const written = useRef<string | null>(null);

  useEffect(() => {
    if (written.current === urlText) {
      written.current = null;
      return;
    }
    written.current = null;
    setEverything(false);
    setRows(fromUrl.length > 0 ? fromUrl : [blankCriterion(1)]);
  }, [fromUrl, urlText]);

  const nextId = () => Math.max(0, ...rows.map((row) => row.id)) + 1;
  const filled = rows.some((row) => row.value.trim() !== '');

  const submit = (event: FormEvent) => {
    event.preventDefault();
    const next = writeCriteria(rows);
    written.current = next.toString();
    setEverything(next.toString() === '');
    setParams(next);
  };

  const clear = () => {
    setRows([blankCriterion(nextId())]);
    setEverything(false);
    written.current = '';
    setParams(new URLSearchParams());
  };

  const path = criteriaPath(params) ?? (everything ? SEARCH : null);
  const results = usePagedSearch<SearchComponent>(path, BATCH);
  let body: ReactNode;
  if (path === null) {
    body = <Notice icon={48} title={t('search.criteria.startTitle')} text={t('search.criteria.startText')} />;
  } else {
    body = <CriteriaResults results={results} />;
  }

  return (
    <Page
      size="large"
      gutter="large"
      className={styles.criteriaPage}
      title={t('search.criteria.title')}
      description={t('search.criteria.description')}
    >
      <div className={styles.stack}>
        <Card size="3">
          <form className={styles.criteria} onSubmit={submit} aria-labelledby="criteria-heading" noValidate>
            <div className={styles.criteriaHead}>
              <Text as="div" size="3" weight="bold" id="criteria-heading">
                {t('search.criteria.heading')}
              </Text>
              <Text as="div" size="2" className={styles.gray}>
                {t('search.criteria.every')}
              </Text>
            </div>
            <div className={styles.criteriaList}>
              {rows.map((row) => (
                <CriterionRow
                  key={row.id}
                  criterion={row}
                  removable={rows.length > 1}
                  onChange={(next) =>
                    setRows(rows.map((candidate) => (candidate.id === row.id ? next : candidate)))
                  }
                  onRemove={() => setRows(rows.filter((candidate) => candidate.id !== row.id))}
                />
              ))}
            </div>
            <div className={styles.criteriaAdd}>
              <Button
                type="button"
                variant="ghost"
                onClick={() => setRows([...rows, blankCriterion(nextId())])}
              >
                <Plus size={16} aria-hidden="true" />
                {t('search.criteria.add')}
              </Button>
            </div>
            <div className={styles.criteriaActions}>
              {filled && (
                <Button type="button" variant="ghost" onClick={clear}>
                  <X size={14} aria-hidden="true" />
                  {t('search.criteria.clear')}
                </Button>
              )}
              <Button type="submit">
                <Search size={16} aria-hidden="true" />
                {t('search.criteria.search')}
              </Button>
            </div>
          </form>
        </Card>
        {body}
      </div>
    </Page>
  );
}
