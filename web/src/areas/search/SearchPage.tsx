import { IconButton, TextField as RadixTextField, Select, Text } from '@radix-ui/themes';
import { PackageSearch, Search, SearchX, X } from 'lucide-react';
import { type FormEvent, useEffect, useId, useRef, useState } from 'react';
import { useSearchParams } from 'react-router';
import { useT } from '../../i18n';
import { Page } from '../../ui';
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
import { ComponentResults, SearchMessage } from './SearchResults';
import { BATCH, usePagedSearch } from './usePagedSearch';

const ALL = '*';
const DEBOUNCE_MS = 300;

function Choice({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: Array<{ value: string; label: string }>;
  onChange(value: string): void;
}) {
  const id = useId();
  return (
    <div className={styles.choice}>
      <Text as="label" size="1" htmlFor={id} id={`${id}-label`} className={styles.gray}>
        {label}
      </Text>
      <Select.Root size="3" value={value} onValueChange={onChange}>
        <Select.Trigger id={id} aria-labelledby={`${id}-label`} />
        <Select.Content position="popper">
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

export default function SearchPage() {
  const t = useT();
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
  const results = usePagedSearch<SearchComponent>(searchable ? searchPath(state) : null, BATCH);

  return (
    <Page
      size="large"
      gutter="large"
      className={styles.page}
      title={t('search.page.title')}
      description={t('search.page.description')}
    >
      <div className={styles.stack}>
        <form role="search" className={styles.bar} onSubmit={submit}>
          <div className={styles.barRow}>
            <div className={styles.query}>
              <RadixTextField.Root
                type="text"
                size="3"
                value={text}
                placeholder={t('search.page.placeholder')}
                aria-label={t('search.page.query')}
                autoComplete="off"
                autoFocus
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
            </div>
            <Choice
              label={t('search.page.format')}
              value={state.format || ALL}
              options={[{ value: ALL, label: t('search.page.allFormats') }, ...formats]}
              onChange={(value) => {
                const format = value === ALL ? '' : value;
                update({ format, repository: format === '' ? state.repository : '' });
              }}
            />
            <Choice
              label={t('search.page.repository')}
              value={state.repository || ALL}
              options={[
                { value: ALL, label: t('search.page.allRepositories') },
                ...repositories.map((name) => ({ value: name, label: name })),
              ]}
              onChange={(value) => update({ repository: value === ALL ? '' : value })}
            />
            <Choice
              label={t('search.page.sort')}
              value={sortChoice(state)}
              options={SORT_CHOICES.map((choice) => ({ value: choice.value, label: t(choice.label) }))}
              onChange={(value) => update(fromSortChoice(value))}
            />
          </div>
        </form>
        <section aria-label={t('search.results.label')}>
          {searchable ? (
            <ComponentResults
              results={results}
              empty={
                <SearchMessage icon={SearchX} title={t('search.results.emptyTitle')}>
                  {t('search.results.emptyText')}
                </SearchMessage>
              }
            />
          ) : (
            <SearchMessage icon={PackageSearch} title={t('search.page.startTitle')}>
              {t('search.page.startText')}
            </SearchMessage>
          )}
        </section>
      </div>
    </Page>
  );
}
