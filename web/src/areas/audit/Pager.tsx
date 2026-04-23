import { Button, IconButton, Select, Text } from '@radix-ui/themes';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useT } from '../../i18n';
import styles from './Audit.module.css';
import { PAGE_SIZES, pageWindow } from './query';

export function Pager({
  page,
  pages,
  total,
  size,
  onPage,
  onSize,
}: {
  page: number;
  pages: number;
  total: number;
  size: number;
  onPage(page: number): void;
  onSize(size: number): void;
}) {
  const t = useT();
  const [jump, setJump] = useState(String(page));
  useEffect(() => {
    setJump(String(page));
  }, [page]);
  const go = (target: number) => {
    const next = Math.min(Math.max(1, target), Math.max(1, pages));
    if (next !== page) {
      onPage(next);
    }
  };
  const first = page <= 1;
  const last = page >= pages;
  return (
    <div className={styles.pager}>
      <div className={styles.pagerInfo}>
        <div className={styles.pagerGroup}>
          <Text size="2">{t('audit.pager.shown')}</Text>
          <Select.Root value={String(size)} onValueChange={(value) => onSize(Number(value))}>
            <Select.Trigger aria-label={t('audit.pager.size')} />
            <Select.Content position="popper">
              {PAGE_SIZES.map((option) => (
                <Select.Item key={option} value={String(option)}>
                  {option}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
          <Text size="2">{t('audit.pager.of', { count: total })}</Text>
        </div>
        <div className={styles.pagerGroup}>
          <Text size="2">{t('audit.pager.page')}</Text>
          <input
            className={styles.jump}
            value={jump}
            inputMode="numeric"
            aria-label={t('audit.pager.jump')}
            onChange={(event) => setJump(event.target.value.replace(/\D/g, ''))}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                go(Number(jump) || 1);
              }
            }}
            onBlur={() => setJump(String(page))}
          />
          <Text size="2">{t('audit.pager.ofPages', { count: pages })}</Text>
        </div>
      </div>
      <nav aria-label={t('audit.pager.label', { page, pages })} className={styles.pages}>
        <IconButton
          type="button"
          variant="outline"
          color="gray"
          disabled={first}
          aria-label={t('audit.pager.first')}
          onClick={() => go(1)}
        >
          <ChevronsLeft size={16} aria-hidden="true" />
        </IconButton>
        <IconButton
          type="button"
          variant="outline"
          color="gray"
          disabled={first}
          aria-label={t('audit.pager.previous')}
          onClick={() => go(page - 1)}
        >
          <ChevronLeft size={16} aria-hidden="true" />
        </IconButton>
        <div className={styles.pageNumbers}>
          {pageWindow(page, pages).map((item, index) =>
            item === 'gap' ? (
              <Text key={`gap-${index === 1 ? 'start' : 'end'}`} size="2" className={styles.gap}>
                …
              </Text>
            ) : (
              <Button
                key={item}
                type="button"
                variant={item === page ? 'solid' : 'outline'}
                color="gray"
                className={styles.pageButton}
                aria-current={item === page ? 'page' : undefined}
                onClick={() => go(item)}
              >
                {item}
              </Button>
            ),
          )}
        </div>
        <IconButton
          type="button"
          variant="outline"
          color="gray"
          disabled={last}
          aria-label={t('audit.pager.next')}
          onClick={() => go(page + 1)}
        >
          <ChevronRight size={16} aria-hidden="true" />
        </IconButton>
        <IconButton
          type="button"
          variant="outline"
          color="gray"
          disabled={last}
          aria-label={t('audit.pager.last')}
          onClick={() => go(pages)}
        >
          <ChevronsRight size={16} aria-hidden="true" />
        </IconButton>
      </nav>
    </div>
  );
}
