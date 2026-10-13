import { IconButton, Select } from '@radix-ui/themes';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useT } from '../../../../i18n';
import styles from './admin.module.css';

export const PAGE_SIZES = [10, 20, 40, 100];

export function Pager({
  total,
  page,
  pageSize,
  onPage,
  onPageSize,
}: {
  total: number;
  page: number;
  pageSize: number;
  onPage(page: number): void;
  onPageSize(size: number): void;
}) {
  const t = useT();
  const pages = Math.max(1, Math.ceil(total / pageSize));
  const [typed, setTyped] = useState(String(page + 1));
  useEffect(() => setTyped(String(page + 1)), [page]);
  const go = (next: number) => onPage(Math.min(pages - 1, Math.max(0, next)));
  return (
    <div className={styles.pager}>
      <div className={styles.pagerLeft}>
        <span className={styles.pagerPart}>
          {t('repositories.admin.shownLabel')}
          <Select.Root value={String(pageSize)} onValueChange={(value) => onPageSize(Number(value))}>
            <Select.Trigger aria-label={t('repositories.admin.pageSize')} />
            <Select.Content position="popper">
              {PAGE_SIZES.map((size) => (
                <Select.Item key={size} value={String(size)}>
                  {size}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
          {t('repositories.admin.ofTotal', { total })}
        </span>
        <span className={styles.pagerPart}>
          {t('repositories.admin.page')}
          <input
            className={styles.pageInput}
            inputMode="numeric"
            aria-label={t('repositories.admin.goToPage')}
            value={typed}
            onChange={(event) => setTyped(event.target.value.replace(/\D/g, ''))}
            onBlur={() => go(Number(typed || '1') - 1)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                go(Number(typed || '1') - 1);
              }
            }}
          />
          {t('repositories.admin.ofPages', { pages })}
        </span>
      </div>
      <nav
        className={styles.pagerNav}
        aria-label={t('repositories.admin.pagination', { page: page + 1, pages })}
      >
        <IconButton
          variant="outline"
          color="gray"
          disabled={page === 0}
          aria-label={t('repositories.admin.firstPage')}
          onClick={() => go(0)}
        >
          <ChevronsLeft size={16} />
        </IconButton>
        <IconButton
          variant="outline"
          color="gray"
          disabled={page === 0}
          aria-label={t('repositories.admin.previousPage')}
          onClick={() => go(page - 1)}
        >
          <ChevronLeft size={16} />
        </IconButton>
        <IconButton variant="solid" color="gray" aria-current="page" style={{ width: 30 }}>
          {page + 1}
        </IconButton>
        <IconButton
          variant="outline"
          color="gray"
          disabled={page >= pages - 1}
          aria-label={t('repositories.admin.nextPage')}
          onClick={() => go(page + 1)}
        >
          <ChevronRight size={16} />
        </IconButton>
        <IconButton
          variant="outline"
          color="gray"
          disabled={page >= pages - 1}
          aria-label={t('repositories.admin.lastPage')}
          onClick={() => go(pages - 1)}
        >
          <ChevronsRight size={16} />
        </IconButton>
      </nav>
    </div>
  );
}
