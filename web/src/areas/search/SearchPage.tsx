import { Button, IconButton, TextField as RadixTextField, Text } from '@radix-ui/themes';
import { PackageSearch, Search, SearchX, SlidersHorizontal, X } from 'lucide-react';
import { type FormEvent, useEffect, useId, useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router';
import { useT } from '../../i18n';
import { EmptyState, FieldShell, Page, SelectField } from '../../ui';
import { ComponentTable } from './ComponentTable';
import type { SearchComponent } from './model';
import { useFormatOptions, useRepositoryNames } from './options';
import {
  fromSortChoice,
  isSearchable,
  readSearchState,
  type SearchState,
  SORT_CHOICES,
  searchPath,
  sortChoice,
  writeSearchState,
} from './query';
import styles from './Search.module.css';
import { usePagedSearch } from './usePagedSearch';

const ALL = '*';
const DEBOUNCE_MS = 300;

export default function SearchPage() {
  const t = useT();
  const queryId = useId();
  const [params, setParams] = useSearchParams();
  const state = readSearchState(params);
  const [text, setText] = useState(state.q);
  const pushed = useRef(state.q);
  const formats = useFormatOptions();
  const repositories = useRepositoryNames(state.format, state.repository);

  const update = (patch: Partial<SearchState>, replace = false) => {
    setParams((previous) => writeSearchState({ ...readSearchState(previous), ...patch }), { replace });
  };

  useEffect(() => {
    if (state.q !== pushed.current) {
      pushed.current = state.q;
      setText(state.q);
    }
  }, [state.q]);

  useEffect(() => {
    if (text === pushed.current) {
      return;
    }
    const timer = window.setTimeout(() => {
      pushed.current = text;
      setParams((previous) => writeSearchState({ ...readSearchState(previous), q: text }), { replace: true });
    }, DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [text, setParams]);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    pushed.current = text;
    update({ q: text }, true);
  };

  const searchable = isSearchable(state);
  const results = usePagedSearch<SearchComponent>(searchable ? searchPath(state) : null);
  const count = results.items?.length ?? 0;

  return (
    <Page
      size="large"
      gutter="large"
      title={t('search.page.title')}
      description={t('search.page.description')}
      actions={
        <Button asChild variant="soft" color="gray">
          <Link to="/search/custom">
            <SlidersHorizontal size={16} aria-hidden="true" />
            {t('search.page.criteria')}
          </Link>
        </Button>
      }
    >
      <form role="search" className={styles.filters} onSubmit={submit} aria-label={t('search.page.filters')}>
        <div className={styles.query}>
          <FieldShell id={queryId} label={t('search.page.query')}>
            <RadixTextField.Root
              id={queryId}
              type="search"
              size="3"
              value={text}
              placeholder={t('search.page.placeholder')}
              autoComplete="off"
              onChange={(event) => setText(event.target.value)}
            >
              <RadixTextField.Slot>
                <Search size={16} aria-hidden="true" />
              </RadixTextField.Slot>
              {text !== '' && (
                <RadixTextField.Slot side="right">
                  <IconButton
                    type="button"
                    size="1"
                    variant="ghost"
                    color="gray"
                    aria-label={t('search.page.clear')}
                    onClick={() => {
                      setText('');
                      pushed.current = '';
                      update({ q: '' }, true);
                    }}
                  >
                    <X size={14} aria-hidden="true" />
                  </IconButton>
                </RadixTextField.Slot>
              )}
            </RadixTextField.Root>
          </FieldShell>
        </div>
        <SelectField
          label={t('search.page.format')}
          size="3"
          value={state.format || ALL}
          options={[{ value: ALL, label: t('search.page.allFormats') }, ...formats]}
          onChange={(value) => {
            const format = value === ALL ? '' : value;
            update({ format, repository: format === '' ? state.repository : '' });
          }}
        />
        <SelectField
          label={t('search.page.repository')}
          size="3"
          value={state.repository || ALL}
          options={[
            { value: ALL, label: t('search.page.allRepositories') },
            ...repositories.map((name) => ({ value: name, label: name })),
          ]}
          onChange={(value) => update({ repository: value === ALL ? '' : value })}
        />
        <SelectField
          label={t('search.page.sort')}
          size="3"
          value={sortChoice(state)}
          options={SORT_CHOICES.map((choice) => ({ value: choice.value, label: t(choice.label) }))}
          onChange={(value) => update(fromSortChoice(value))}
        />
      </form>
      {searchable && results.items !== undefined && count > 0 && (
        <div className={styles.status}>
          <Text size="2" color="gray" role="status">
            {results.hasMore
              ? t('search.results.shownMore', { count })
              : t('search.results.found', { count })}
          </Text>
        </div>
      )}
      {searchable ? (
        <ComponentTable
          label={t('search.results.label')}
          results={results}
          empty={
            <EmptyState
              icon={SearchX}
              title={t('search.results.emptyTitle')}
              description={t('search.results.emptyText')}
            />
          }
        />
      ) : (
        <EmptyState
          icon={PackageSearch}
          title={t('search.page.startTitle')}
          description={t('search.page.startText')}
        />
      )}
    </Page>
  );
}
