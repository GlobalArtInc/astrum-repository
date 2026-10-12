import { ArrowLeft, Search, X } from 'lucide-react';
import { type KeyboardEvent, useEffect, useId, useRef, useState } from 'react';
import { useNavigate } from 'react-router';
import { useVisibleSearch } from '../../app/registry';
import { useT } from '../../i18n';
import { ITEM_SELECTOR, SearchPanelContext } from './context';
import styles from './SearchPanel.module.css';

export function isSearchShortcut(event: {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
}) {
  return event.key.toLowerCase() === 'k' && (event.ctrlKey || event.metaKey) && !event.altKey;
}

export function useSearchShortcut(toggle: () => void): void {
  useEffect(() => {
    const listener = (event: globalThis.KeyboardEvent) => {
      if (isSearchShortcut(event)) {
        event.preventDefault();
        toggle();
      }
    };
    window.addEventListener('keydown', listener);
    return () => window.removeEventListener('keydown', listener);
  }, [toggle]);
}

export function isMac(): boolean {
  return /mac|iphone|ipad/i.test(navigator.platform || navigator.userAgent);
}

function items(container: HTMLElement | null): HTMLElement[] {
  return Array.from(container?.querySelectorAll<HTMLElement>(ITEM_SELECTOR) ?? []);
}

function markActive(container: HTMLElement | null, index: number): string | undefined {
  const all = items(container);
  all.forEach((item, position) => {
    item.dataset.active = String(position === index);
    item.setAttribute('aria-selected', String(position === index));
  });
  const chosen = all[index];
  chosen?.scrollIntoView({ block: 'nearest' });
  return chosen?.id;
}

export interface SearchPanelProps {
  open: boolean;
  onOpenChange(open: boolean): void;
  layer?: boolean;
}

export function SearchPanel({ open, onOpenChange, layer = false }: SearchPanelProps) {
  const t = useT();
  const navigate = useNavigate();
  const providers = useVisibleSearch();
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(-1);
  const [activeId, setActiveId] = useState<string | undefined>();
  const container = useRef<HTMLDivElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const results = useRef<HTMLDivElement>(null);
  const listId = useId();

  const resetActive = () => {
    markActive(results.current, -1);
    setActive(-1);
    setActiveId(undefined);
  };

  const finish = () => {
    setQuery('');
    resetActive();
    input.current?.blur();
    onOpenChange(false);
  };

  useEffect(() => {
    if (open && document.activeElement !== input.current) {
      input.current?.focus();
    }
    if (!open) {
      setActive(-1);
      setActiveId(undefined);
    }
  }, [open]);

  useEffect(() => {
    if (!open || layer) {
      return;
    }
    const outside = (event: PointerEvent) => {
      if (container.current && !container.current.contains(event.target as Node)) {
        onOpenChange(false);
      }
    };
    document.addEventListener('pointerdown', outside);
    return () => document.removeEventListener('pointerdown', outside);
  }, [open, layer, onOpenChange]);

  const move = (step: number) => {
    const count = items(results.current).length;
    if (count === 0) {
      return;
    }
    const next = active < 0 ? (step > 0 ? 0 : count - 1) : (active + step + count) % count;
    setActive(next);
    setActiveId(markActive(results.current, next));
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (!open) {
        onOpenChange(true);
        return;
      }
      move(event.key === 'ArrowDown' ? 1 : -1);
      return;
    }
    if (event.key === 'Escape') {
      event.preventDefault();
      if (layer || !open) {
        finish();
      } else {
        onOpenChange(false);
      }
      return;
    }
    if (event.key === 'Enter') {
      event.preventDefault();
      const chosen = active >= 0 ? items(results.current)[active] : undefined;
      if (chosen) {
        chosen.click();
        return;
      }
      const text = query.trim();
      for (const provider of providers) {
        const target = text ? provider.submit?.(text) : null;
        if (target) {
          finish();
          navigate(target);
          return;
        }
      }
    }
  };

  const shown = open || layer;
  const field = (
    <div className={styles.field} data-focused={shown || undefined}>
      <span className={styles.slot} aria-hidden="true">
        <Search size={16} />
      </span>
      <input
        ref={input}
        className={styles.input}
        type="text"
        role="combobox"
        autoComplete="off"
        spellCheck={false}
        aria-expanded={shown}
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={shown ? activeId : undefined}
        aria-label={t('shell.search.label')}
        placeholder={t('shell.search.placeholder')}
        value={query}
        onFocus={() => {
          if (!open) {
            onOpenChange(true);
          }
        }}
        onChange={(event) => {
          setQuery(event.target.value);
          resetActive();
          if (!open) {
            onOpenChange(true);
          }
        }}
        onKeyDown={onKeyDown}
      />
      {query !== '' ? (
        <button
          type="button"
          className={styles.clear}
          aria-label={t('shell.search.clear')}
          onClick={() => {
            setQuery('');
            resetActive();
            input.current?.focus();
          }}
        >
          <X size={14} aria-hidden="true" />
        </button>
      ) : (
        !layer && (
          <span className={styles.slot} aria-hidden="true">
            <kbd className={styles.kbd}>{isMac() ? '⌘ K' : 'Ctrl K'}</kbd>
          </span>
        )
      )}
    </div>
  );

  const dropdown = shown && (
    <SearchPanelContext.Provider value={{ close: finish }}>
      <div className={styles.dropdown}>
        <div
          ref={results}
          id={listId}
          role="listbox"
          aria-label={t('shell.search.results')}
          className={styles.results}
        >
          {providers.length === 0 || query.trim() === '' ? (
            <div className={styles.hint}>{t('shell.search.hint')}</div>
          ) : null}
          {providers.map((provider) => (
            <provider.Results key={provider.id} query={query.trim()} close={finish} />
          ))}
        </div>
      </div>
    </SearchPanelContext.Provider>
  );

  if (layer) {
    return (
      <div className={styles.layer} role="dialog" aria-modal="true" aria-label={t('shell.search.title')}>
        <div className={styles.bar}>
          <button type="button" className={styles.back} aria-label={t('shell.search.close')} onClick={finish}>
            <ArrowLeft size={20} aria-hidden="true" />
          </button>
          <div ref={container} className={styles.container}>
            {field}
            {dropdown}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={container}
      role="search"
      className={styles.container}
      onBlur={(event) => {
        if (open && !event.currentTarget.contains(event.relatedTarget as Node | null)) {
          onOpenChange(false);
        }
      }}
    >
      {field}
      {dropdown}
    </div>
  );
}
