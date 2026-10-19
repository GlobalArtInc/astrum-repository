import { Button, DropdownMenu } from '@radix-ui/themes';
import { keepPreviousData, useQuery } from '@tanstack/react-query';
import {
  ArrowLeft,
  ArrowRight,
  Ban,
  CircleAlert,
  CircleCheck,
  CircleX,
  FileCode,
  Plus,
  Search,
  Trash2,
} from 'lucide-react';
import { api } from '../../api';
import { useT } from '../../i18n';
import { Tooltip } from '../../ui';
import { useDebounced } from '../security/shared/useDebounced';
import {
  describePattern,
  type Matcher,
  type MatcherKind,
  type Mode,
  matcher,
  normalizePath,
  TEST,
  type TestResult,
  toPattern,
} from './model';
import styles from './Routing.module.css';

const KINDS: Array<{ kind: MatcherKind; icon: typeof FileCode; title: string; hint: string }> = [
  {
    kind: 'regex',
    icon: FileCode,
    title: 'routing.matcher.kinds.regex',
    hint: 'routing.matcher.kinds.regexText',
  },
  {
    kind: 'prefix',
    icon: ArrowRight,
    title: 'routing.matcher.kinds.prefix',
    hint: 'routing.matcher.kinds.prefixText',
  },
  {
    kind: 'suffix',
    icon: ArrowLeft,
    title: 'routing.matcher.kinds.suffix',
    hint: 'routing.matcher.kinds.suffixText',
  },
  {
    kind: 'contains',
    icon: Search,
    title: 'routing.matcher.kinds.contains',
    hint: 'routing.matcher.kinds.containsText',
  },
];

const PLACEHOLDERS: Record<MatcherKind, string> = {
  regex: 'routing.matcher.placeholders.regex',
  prefix: 'routing.matcher.placeholders.prefix',
  suffix: 'routing.matcher.placeholders.suffix',
  contains: 'routing.matcher.placeholders.contains',
};

const DESCRIBE: Record<MatcherKind, string> = {
  regex: 'routing.matcher.describe.regex',
  prefix: 'routing.matcher.describe.prefix',
  suffix: 'routing.matcher.describe.suffix',
  contains: 'routing.matcher.describe.contains',
};

export const MODES: Record<Mode, string> = {
  ALLOW: 'routing.modes.ALLOW',
  BLOCK: 'routing.modes.BLOCK',
};

export interface MatcherState {
  matches: boolean | null;
  error: string | null;
}

export function useMatcherTest(mode: Mode, matchers: readonly Matcher[], path: string) {
  const filled = matchers.filter((entry) => entry.text.trim() !== '');
  const body = useDebounced(
    JSON.stringify({ mode, matchers: filled.map(toPattern), path: normalizePath(path) || '/' }),
    300,
  );
  const sent = JSON.parse(body) as { matchers: string[] };
  const query = useQuery<TestResult>({
    queryKey: ['routing-test', body],
    queryFn: () => api.post<TestResult>(TEST, JSON.parse(body)),
    enabled: sent.matchers.length > 0,
    placeholderData: keepPreviousData,
    staleTime: Number.POSITIVE_INFINITY,
    retry: false,
  });
  const states = new Map<number, MatcherState>();
  const current = filled.map(toPattern);
  const fresh = JSON.stringify(sent.matchers) === JSON.stringify(current);
  if (fresh && query.data && query.data.matchers.length === filled.length) {
    filled.forEach((entry, index) => {
      const result = query.data.matchers[index];
      if (result) {
        states.set(entry.id, { matches: result.error ? null : result.matches, error: result.error });
      }
    });
  }
  const anyMatch = [...states.values()].some((state) => state.matches === true);
  return {
    states,
    allowed: fresh && query.data ? query.data.allowed : null,
    anyMatch,
    ready: fresh && query.data !== undefined && filled.length > 0,
  };
}

function Badge({ state }: { state: MatcherState | undefined }) {
  const t = useT();
  if (!state) {
    return null;
  }
  if (state.error) {
    return (
      <span className={`${styles.badge} ${styles.badgeInvalid}`}>
        <CircleX size={12} aria-hidden="true" />
        {t('routing.matcher.invalid')}
      </span>
    );
  }
  return state.matches ? (
    <span className={`${styles.badge} ${styles.badgeMatch}`}>
      <CircleCheck size={12} aria-hidden="true" />
      {t('routing.matcher.matches')}
    </span>
  ) : (
    <span className={`${styles.badge} ${styles.badgeMiss}`}>
      <CircleX size={12} aria-hidden="true" />
      {t('routing.matcher.noMatch')}
    </span>
  );
}

export function MatcherList({
  mode,
  matchers,
  onChange,
  path,
  onPathChange,
  problem,
  disabled,
}: {
  mode: Mode;
  matchers: Matcher[];
  onChange(next: Matcher[]): void;
  path: string;
  onPathChange(path: string): void;
  problem?: string;
  disabled?: boolean;
}) {
  const t = useT();
  const test = useMatcherTest(mode, matchers, path);
  const tested = path.trim() !== '';
  const hint = mode === 'BLOCK' ? t('routing.form.blockHint') : t('routing.form.allowHint');
  const invalid = matchers.filter((entry) => test.states.get(entry.id)?.error);
  const update = (id: number, text: string) =>
    onChange(matchers.map((entry) => (entry.id === id ? { ...entry, text } : entry)));
  return (
    <div className={styles.matchers}>
      <div className={styles.matchersHead}>
        <div className={styles.matchersLabel}>
          <span className={styles.matchersTitle}>
            {t('routing.form.matchers')}
            <span className={styles.required} aria-hidden="true">
              *
            </span>
          </span>
          <span className={styles.matchersHint}>{hint}</span>
        </div>
        {!disabled && (
          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              <Button type="button" variant="ghost" color="gray" className={styles.addButton}>
                <Plus size={16} aria-hidden="true" />
                {t('routing.matcher.add')}
              </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
              {KINDS.map(({ kind, icon: Icon, title, hint }) => (
                <DropdownMenu.Item
                  key={kind}
                  className={styles.menuItem}
                  onSelect={() => onChange([...matchers, matcher(kind)])}
                >
                  <span className={styles.menuIcon}>
                    <Icon size={14} aria-hidden="true" />
                  </span>
                  <span className={styles.menuText}>
                    <span className={styles.menuTitle}>{t(title)}</span>
                    <span className={styles.menuHint}>{t(hint)}</span>
                  </span>
                </DropdownMenu.Item>
              ))}
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        )}
      </div>
      {matchers.map((entry, index) => {
        const state = test.states.get(entry.id);
        const shown = state?.error ? state : tested ? state : undefined;
        const number = index + 1;
        return (
          <div
            key={entry.id}
            className={styles.matcherRow}
            data-state={shown?.error ? 'invalid' : shown?.matches ? 'match' : undefined}
          >
            <span className={styles.matcherNumber} aria-hidden="true">
              {number}
            </span>
            <input
              type="text"
              className={styles.matcherInput}
              value={entry.text}
              placeholder={t(PLACEHOLDERS[entry.kind])}
              aria-label={t('routing.matcher.label', { number })}
              aria-invalid={shown?.error ? true : undefined}
              disabled={disabled}
              spellCheck={false}
              onChange={(event) => update(entry.id, event.target.value)}
            />
            <span className={styles.badgeSlot}>
              <Badge state={shown} />
            </span>
            {!disabled && (
              <Tooltip content={t('routing.matcher.remove', { number })}>
                <button
                  type="button"
                  className={styles.remove}
                  aria-label={t('routing.matcher.remove', { number })}
                  onClick={() => onChange(matchers.filter((other) => other.id !== entry.id))}
                >
                  <Trash2 size={16} aria-hidden="true" />
                </button>
              </Tooltip>
            )}
          </div>
        );
      })}
      {(problem || invalid.length > 0) && (
        <div className={styles.problem} role="alert">
          <CircleAlert size={14} aria-hidden="true" />
          <span>
            {invalid.length > 0
              ? invalid.map((entry) => t('routing.matcher.invalidPattern', { pattern: entry.text })).join(' ')
              : problem}
          </span>
        </div>
      )}
      <section className={styles.box} aria-labelledby="routing-test-title">
        <h3 id="routing-test-title" className={styles.boxTitle}>
          {t('routing.matcher.testTitle')}
        </h3>
        <p className={styles.boxText}>{t('routing.matcher.testText')}</p>
        <div className={styles.testField}>
          <Search size={16} aria-hidden="true" className={styles.testIcon} />
          <input
            type="text"
            className={styles.testInput}
            value={path}
            placeholder={t('routing.matcher.testPlaceholder')}
            aria-label={t('routing.matcher.testTitle')}
            spellCheck={false}
            onChange={(event) => onPathChange(event.target.value)}
          />
        </div>
        {tested && test.ready && test.allowed !== null && (
          <div className={styles.result} data-allowed={test.allowed} role="status">
            {test.allowed ? (
              <CircleCheck size={20} aria-hidden="true" />
            ) : (
              <Ban size={20} aria-hidden="true" />
            )}
            <div>
              <div className={styles.resultTitle}>
                {test.allowed ? t('routing.matcher.allowed') : t('routing.matcher.blocked')}
              </div>
              <div className={styles.resultText}>
                {test.anyMatch ? t('routing.matcher.matched') : t('routing.matcher.notMatched')}
              </div>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}

export function RuleSummary({ mode, matchers }: { mode: Mode; matchers: readonly Matcher[] }) {
  const t = useT();
  const filled = matchers.filter((entry) => entry.text.trim() !== '');
  if (filled.length === 0) {
    return null;
  }
  return (
    <section className={styles.summary} aria-labelledby="routing-summary-title">
      <h3 id="routing-summary-title" className={styles.summaryTitle}>
        {t('routing.matcher.summary')}
      </h3>
      <p className={styles.summaryLine}>
        <strong>{t('routing.matcher.summaryMode')}</strong> {t(MODES[mode])}
      </p>
      <p className={styles.summaryLine}>
        <strong>{t('routing.matcher.summaryWhere')}</strong>
      </p>
      <ul className={styles.summaryList}>
        {filled.map((entry, index) => {
          const described = describePattern(toPattern(entry));
          return (
            <li key={entry.id}>
              {index > 0 && <span className={styles.or}>{t('routing.matcher.or')}</span>}
              {t(DESCRIBE[described.kind], { text: described.text })}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
