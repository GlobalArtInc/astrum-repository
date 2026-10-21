import { Dialog, VisuallyHidden } from '@radix-ui/themes';
import { Search, X } from 'lucide-react';
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

export interface SearchTriggerProps {
  onOpen(): void;
}

export function SearchTrigger({ onOpen }: SearchTriggerProps) {
  const t = useT();
  return (
    <button
      type="button"
      className={styles.trigger}
      aria-haspopup="dialog"
      aria-label={t('shell.search.open')}
      onClick={onOpen}
    >
      <span className={styles.slot} aria-hidden="true">
        <Search size={16} />
      </span>
      <span className={styles.placeholder}>{t('shell.search.placeholder')}</span>
      <span className={styles.slot} aria-hidden="true">
        <kbd className={styles.kbd}>{isMac() ? '⌘ K' : 'Ctrl K'}</kbd>
      </span>
    </button>
  );
}

export interface SearchDialogProps {
  open: boolean;
  onOpenChange(open: boolean): void;
}

export function SearchDialog({ open, onOpenChange }: SearchDialogProps) {
  const t = useT();
  const navigate = useNavigate();
  const providers = useVisibleSearch();
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(-1);
  const [activeId, setActiveId] = useState<string | undefined>();
  const input = useRef<HTMLInputElement>(null);
  const results = useRef<HTMLDivElement>(null);
  const listId = useId();

  useEffect(() => {
    if (!open) {
      setQuery('');
      setActive(-1);
      setActiveId(undefined);
    }
  }, [open]);

  const resetActive = () => {
    markActive(results.current, -1);
    setActive(-1);
    setActiveId(undefined);
  };

  const close = () => onOpenChange(false);

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
      move(event.key === 'ArrowDown' ? 1 : -1);
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
          close();
          navigate(target);
          return;
        }
      }
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content
        align="start"
        maxWidth="720px"
        className={styles.dialog}
        aria-describedby={undefined}
        onOpenAutoFocus={(event) => {
          event.preventDefault();
          input.current?.focus();
        }}
      >
        <VisuallyHidden>
          <Dialog.Title>{t('shell.search.title')}</Dialog.Title>
        </VisuallyHidden>
        <SearchPanelContext.Provider value={{ close }}>
          <div className={styles.field}>
            <span className={styles.slot} aria-hidden="true">
              <Search size={18} />
            </span>
            <input
              ref={input}
              className={styles.input}
              type="text"
              role="combobox"
              autoComplete="off"
              spellCheck={false}
              aria-expanded="true"
              aria-controls={listId}
              aria-autocomplete="list"
              aria-activedescendant={activeId}
              aria-label={t('shell.search.label')}
              placeholder={t('shell.search.placeholder')}
              value={query}
              onChange={(event) => {
                setQuery(event.target.value);
                resetActive();
              }}
              onKeyDown={onKeyDown}
            />
            {query !== '' && (
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
            )}
            <Dialog.Close>
              <button type="button" className={styles.escape} aria-label={t('shell.search.close')}>
                Esc
              </button>
            </Dialog.Close>
          </div>
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
              <provider.Results key={provider.id} query={query.trim()} close={close} />
            ))}
          </div>
          <div className={styles.footer} aria-hidden="true">
            <span className={styles.key}>
              <kbd className={styles.kbd}>↑</kbd>
              <kbd className={styles.kbd}>↓</kbd>
              {t('shell.search.keysMove')}
            </span>
            <span className={styles.key}>
              <kbd className={styles.kbd}>Enter</kbd>
              {t('shell.search.keysOpen')}
            </span>
            <span className={styles.key}>
              <kbd className={styles.kbd}>Esc</kbd>
              {t('shell.search.keysClose')}
            </span>
          </div>
        </SearchPanelContext.Provider>
      </Dialog.Content>
    </Dialog.Root>
  );
}
