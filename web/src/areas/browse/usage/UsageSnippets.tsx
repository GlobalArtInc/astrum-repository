import { Text } from '@radix-ui/themes';
import { type KeyboardEvent, useId, useMemo, useRef, useState } from 'react';
import { useT } from '../../../i18n';
import { CodeBlock, formatName } from '../../../ui';
import { type Snippet, type UsageContext, usageSnippets } from './snippets';
import styles from './Usage.module.css';

export interface UsageSnippetsProps {
  context: UsageContext;
}

export interface UsageTabsProps {
  snippets: Snippet[];
}

export function UsageTabs({ snippets }: UsageTabsProps) {
  const t = useT();
  const id = useId();
  const [chosen, setChosen] = useState<string | undefined>(undefined);
  const list = useRef<HTMLDivElement>(null);
  const current = snippets.find((snippet) => snippet.id === chosen) ?? snippets[0];
  if (!current) {
    return null;
  }
  const label = (snippet: Snippet) => (snippet.labelKey ? t(snippet.labelKey) : snippet.label);
  const move = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0;
    if (step === 0 && event.key !== 'Home' && event.key !== 'End') {
      return;
    }
    event.preventDefault();
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? snippets.length - 1
          : (index + step + snippets.length) % snippets.length;
    const target = snippets[next];
    if (target) {
      setChosen(target.id);
      list.current?.querySelectorAll<HTMLButtonElement>('[role=tab]')[next]?.focus();
    }
  };
  return (
    <div className={styles.root}>
      <div ref={list} role="tablist" aria-label={t('browse.usage.clients')} className={styles.tabs}>
        {snippets.map((snippet, index) => {
          const selected = snippet.id === current.id;
          return (
            <button
              key={snippet.id}
              type="button"
              role="tab"
              id={`${id}-tab-${snippet.id}`}
              aria-selected={selected}
              aria-controls={`${id}-panel`}
              tabIndex={selected ? 0 : -1}
              className={styles.tab}
              onClick={() => setChosen(snippet.id)}
              onKeyDown={(event) => move(event, index)}
            >
              {label(snippet)}
            </button>
          );
        })}
      </div>
      <div
        role="tabpanel"
        id={`${id}-panel`}
        aria-labelledby={`${id}-tab-${current.id}`}
        className={styles.panel}
      >
        {current.hint && (
          <Text as="p" size="2" color="gray" className={styles.hint}>
            {t(current.hint, current.hintVars)}
          </Text>
        )}
        <CodeBlock code={current.code} copyLabel={t('browse.usage.copy', { label: label(current) })} />
      </div>
    </div>
  );
}

export function UsageSnippets({ context }: UsageSnippetsProps) {
  const t = useT();
  const snippets = useMemo(() => usageSnippets(context), [context]);
  if (snippets.length === 0) {
    return (
      <Text as="p" size="2" color="gray">
        {t('browse.usage.none', { format: formatName(context.format) })}
      </Text>
    );
  }
  return <UsageTabs snippets={snippets} />;
}
