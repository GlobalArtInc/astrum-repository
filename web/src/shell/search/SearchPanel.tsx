import { IconButton, Theme } from '@radix-ui/themes';
import { Search, X } from 'lucide-react';
import { Dialog } from 'radix-ui';
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

function moveActive(container: HTMLElement | null, step: number, current: number): number {
  const items = Array.from(container?.querySelectorAll<HTMLElement>(ITEM_SELECTOR) ?? []);
  if (items.length === 0) {
    return -1;
  }
  const next =
    current < 0 ? (step > 0 ? 0 : items.length - 1) : (current + step + items.length) % items.length;
  items.forEach((item, index) => {
    item.dataset.active = String(index === next);
    item.setAttribute('aria-selected', String(index === next));
  });
  items[next]?.scrollIntoView({ block: 'nearest' });
  return next;
}

export function SearchPanel({ open, onOpenChange }: { open: boolean; onOpenChange(open: boolean): void }) {
  const t = useT();
  const navigate = useNavigate();
  const providers = useVisibleSearch();
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(-1);
  const results = useRef<HTMLDivElement>(null);
  const listId = useId();
  const close = () => onOpenChange(false);

  useEffect(() => {
    if (!open) {
      setQuery('');
      setActive(-1);
    }
  }, [open]);

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      setActive(moveActive(results.current, event.key === 'ArrowDown' ? 1 : -1, active));
      return;
    }
    if (event.key === 'Enter') {
      event.preventDefault();
      const items = results.current?.querySelectorAll<HTMLElement>(ITEM_SELECTOR);
      const chosen = active >= 0 ? items?.[active] : undefined;
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
      <Dialog.Portal>
        <Theme>
          <Dialog.Overlay className={styles.scrim} />
          <Dialog.Content className={styles.panel} aria-describedby={undefined}>
            <Dialog.Title className="visually-hidden">{t('shell.search.title')}</Dialog.Title>
            <div className={styles.inputRow}>
              <Search size={18} aria-hidden="true" color="var(--gray-10)" />
              <input
                className={styles.input}
                type="search"
                role="combobox"
                aria-expanded="true"
                aria-controls={listId}
                aria-autocomplete="list"
                aria-label={t('shell.search.label')}
                placeholder={t('shell.search.placeholder')}
                value={query}
                onChange={(event) => {
                  setQuery(event.target.value);
                  setActive(-1);
                }}
                onKeyDown={onKeyDown}
              />
              <Dialog.Close asChild>
                <IconButton variant="ghost" color="gray" aria-label={t('shell.search.close')}>
                  <X size={18} aria-hidden="true" />
                </IconButton>
              </Dialog.Close>
            </div>
            <SearchPanelContext.Provider value={{ close }}>
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
            </SearchPanelContext.Provider>
            <div className={styles.footer}>
              <span>{t('shell.search.keysMove')}</span>
              <span>{t('shell.search.keysOpen')}</span>
              <span>{t('shell.search.keysClose')}</span>
            </div>
          </Dialog.Content>
        </Theme>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
