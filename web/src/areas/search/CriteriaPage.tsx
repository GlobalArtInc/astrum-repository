import {
  Button,
  Flex,
  Heading,
  IconButton,
  TextField as RadixTextField,
  SegmentedControl,
  Select,
  Text,
} from '@radix-ui/themes';
import { Plus, Search, SearchX, X } from 'lucide-react';
import { type FormEvent, useEffect, useMemo, useState } from 'react';
import { useSearchParams } from 'react-router';
import { useSession } from '../../auth/state';
import { type Translate, useT } from '../../i18n';
import { EmptyState, FormError, Page } from '../../ui';
import { AssetTable, ComponentTable } from './ComponentTable';
import {
  availableCriteria,
  type Criterion,
  type CriterionProblem,
  checkCriterion,
  criteriaPath,
  defaultOperator,
  definition,
  type Mode,
  nextKey,
  type Operator,
  operatorsOf,
  readCriteria,
  writeCriteria,
} from './criteria';
import type { SearchAsset, SearchComponent } from './model';
import { useFormatOptions, useRepositoryNames } from './options';
import styles from './Search.module.css';
import { usePagedSearch } from './usePagedSearch';

const OPERATOR_LABELS: Record<Operator, string> = {
  is: 'search.operator.is',
  startsWith: 'search.operator.startsWith',
  contains: 'search.operator.contains',
};

let lastId = 1000;

function freshCriterion(key: string): Criterion {
  lastId += 1;
  return { id: lastId, key, operator: defaultOperator(key), value: '' };
}

const PROBLEMS: Record<CriterionProblem, string> = {
  empty: 'search.criteria.emptyValue',
  checksum: 'search.criteria.badChecksum',
};

function problemText(t: Translate, problem: CriterionProblem, criterion: Criterion): string {
  return t(PROBLEMS[problem], { length: definition(criterion.key)?.length ?? 0 });
}

function CriterionRow({
  criterion,
  keys,
  error,
  onChange,
  onRemove,
  removable,
}: {
  criterion: Criterion;
  keys: Array<{ value: string; label: string }>;
  error?: string;
  onChange(next: Criterion): void;
  onRemove(): void;
  removable: boolean;
}) {
  const t = useT();
  const found = definition(criterion.key);
  const formats = useFormatOptions();
  const repositories = useRepositoryNames('', found?.kind === 'repository' ? criterion.value : '');
  const operators = operatorsOf(criterion.key);
  const valueLabel = t('search.criteria.valueOf', { name: found ? t(found.label) : criterion.key });
  const errorId = `criterion-${criterion.id}-error`;
  const choices =
    found?.kind === 'format'
      ? formats
      : found?.kind === 'repository'
        ? repositories.map((name) => ({ value: name, label: name }))
        : found?.kind === 'choice'
          ? (found.options ?? []).map((option) => ({ value: option.value, label: t(option.label) }))
          : null;
  return (
    <li className={styles.criterion}>
      <div className={styles.criterionKey}>
        <Select.Root
          value={criterion.key}
          onValueChange={(key) => onChange({ ...criterion, key, operator: defaultOperator(key), value: '' })}
        >
          <Select.Trigger aria-label={t('search.criteria.field')} className={styles.full} />
          <Select.Content position="popper">
            {keys.map((key) => (
              <Select.Item key={key.value} value={key.value}>
                {key.label}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
      </div>
      <div className={styles.criterionOperator}>
        {operators.length > 1 ? (
          <Select.Root
            value={criterion.operator}
            onValueChange={(operator) => onChange({ ...criterion, operator: operator as Operator })}
          >
            <Select.Trigger aria-label={t('search.criteria.operator')} className={styles.full} />
            <Select.Content position="popper">
              {operators.map((operator) => (
                <Select.Item key={operator} value={operator}>
                  {t(OPERATOR_LABELS[operator])}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        ) : (
          <span className={styles.operator}>{t(OPERATOR_LABELS[operators[0] ?? 'is'])}</span>
        )}
      </div>
      <div className={styles.criterionValue}>
        {choices ? (
          <Select.Root
            value={criterion.value === '' ? undefined : criterion.value}
            onValueChange={(value) => onChange({ ...criterion, value })}
          >
            <Select.Trigger
              aria-label={valueLabel}
              placeholder={t('search.criteria.choose')}
              className={styles.full}
              aria-invalid={error ? true : undefined}
              aria-describedby={error ? errorId : undefined}
            />
            <Select.Content position="popper">
              {choices.map((choice) => (
                <Select.Item key={choice.value} value={choice.value}>
                  {choice.label}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        ) : (
          <RadixTextField.Root
            aria-label={valueLabel}
            value={criterion.value}
            placeholder={
              found?.example
                ? t('search.criteria.example', { example: found.example })
                : t('search.criteria.value')
            }
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? errorId : undefined}
            onChange={(event) => onChange({ ...criterion, value: event.target.value })}
          />
        )}
        {error && (
          <Text id={errorId} size="1" color="red" role="alert">
            {error}
          </Text>
        )}
      </div>
      <div className={styles.criterionRemove}>
        <IconButton
          type="button"
          variant="ghost"
          color="gray"
          mt="1"
          disabled={!removable}
          aria-label={t('search.criteria.remove', { name: found ? t(found.label) : criterion.key })}
          onClick={onRemove}
        >
          <X size={16} aria-hidden="true" />
        </IconButton>
      </div>
    </li>
  );
}

export default function CriteriaPage() {
  const t = useT();
  const { state } = useSession();
  const [params, setParams] = useSearchParams();
  const formatIds = useMemo(() => state.formats.map((format) => format.id), [state.formats]);
  const fromUrl = useMemo(() => readCriteria(params), [params]);
  const initial = () => (fromUrl.criteria.length > 0 ? fromUrl.criteria : [freshCriterion('q')]);
  const [rows, setRows] = useState<Criterion[]>(initial);
  const [mode, setMode] = useState<Mode>(fromUrl.mode);
  const [errors, setErrors] = useState<Record<number, string>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const urlText = params.toString();

  useEffect(() => {
    const next = readCriteria(new URLSearchParams(urlText));
    setRows(next.criteria.length > 0 ? next.criteria : [freshCriterion('q')]);
    setMode(next.mode);
    setErrors({});
    setFormError(null);
  }, [urlText]);

  const keys = useMemo(
    () =>
      availableCriteria(formatIds).map((candidate) => ({ value: candidate.key, label: t(candidate.label) })),
    [formatIds, t],
  );

  const add = () => {
    const key = nextKey(rows, formatIds);
    if (key) {
      setRows([...rows, freshCriterion(key)]);
    }
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();
    const filled = rows.filter((row) => row.value.trim() !== '');
    if (filled.length === 0) {
      setFormError(t('search.criteria.none'));
      return;
    }
    const found: Record<number, string> = {};
    for (const row of filled) {
      const problem = checkCriterion(row);
      if (problem) {
        found[row.id] = problemText(t, problem, row);
      }
    }
    const seen = new Set<string>();
    for (const row of filled) {
      if (seen.has(row.key)) {
        found[row.id] = t('search.criteria.duplicate');
      }
      seen.add(row.key);
    }
    setErrors(found);
    setFormError(null);
    if (Object.keys(found).length > 0) {
      return;
    }
    setParams(writeCriteria(filled, mode));
  };

  const path = criteriaPath(params);
  const assets = fromUrl.mode === 'assets';
  const components = usePagedSearch<SearchComponent>(path && !assets ? path : null);
  const files = usePagedSearch<SearchAsset>(path && assets ? path : null);
  const empty = (
    <EmptyState
      icon={SearchX}
      title={assets ? t('search.criteria.noAssets') : t('search.results.emptyTitle')}
      description={t('search.criteria.emptyText')}
    />
  );

  return (
    <Page
      size="large"
      gutter="large"
      title={t('search.criteria.title')}
      description={t('search.criteria.description')}
      breadcrumbs={[{ label: t('search.page.title'), to: '/search' }, { label: t('search.criteria.title') }]}
    >
      <form className={styles.criteria} onSubmit={submit} aria-labelledby="criteria-heading" noValidate>
        <Flex direction="column" gap="1">
          <Heading as="h2" size="4" id="criteria-heading">
            {t('search.criteria.heading')}
          </Heading>
          <Text size="2" color="gray">
            {t('search.criteria.every')}
          </Text>
        </Flex>
        <ul className={styles.criteriaList}>
          {rows.map((row) => (
            <CriterionRow
              key={row.id}
              criterion={row}
              keys={keys.filter(
                (key) => key.value === row.key || !rows.some((other) => other.key === key.value),
              )}
              error={errors[row.id]}
              removable={rows.length > 1}
              onChange={(next) => {
                setRows(rows.map((candidate) => (candidate.id === row.id ? next : candidate)));
                setErrors(({ [row.id]: _, ...rest }) => rest);
              }}
              onRemove={() => setRows(rows.filter((candidate) => candidate.id !== row.id))}
            />
          ))}
        </ul>
        <div className={styles.criteriaActions}>
          <Button type="button" variant="ghost" onClick={add} disabled={nextKey(rows, formatIds) === null}>
            <Plus size={16} aria-hidden="true" />
            {t('search.criteria.add')}
          </Button>
          <Flex gap="3" align="center" wrap="wrap">
            <SegmentedControl.Root
              value={mode}
              onValueChange={(value) => setMode(value === 'assets' ? 'assets' : 'components')}
              aria-label={t('search.criteria.mode')}
            >
              <SegmentedControl.Item value="components">
                {t('search.criteria.components')}
              </SegmentedControl.Item>
              <SegmentedControl.Item value="assets">{t('search.criteria.assets')}</SegmentedControl.Item>
            </SegmentedControl.Root>
            <Button type="submit">
              <Search size={16} aria-hidden="true" />
              {t('search.criteria.search')}
            </Button>
          </Flex>
        </div>
        <FormError message={formError} />
      </form>
      {path === null ? (
        <EmptyState
          icon={Search}
          title={t('search.criteria.startTitle')}
          description={t('search.criteria.startText')}
        />
      ) : assets ? (
        <AssetTable label={t('search.criteria.assetResults')} results={files} empty={empty} />
      ) : (
        <ComponentTable label={t('search.results.label')} results={components} empty={empty} />
      )}
    </Page>
  );
}
