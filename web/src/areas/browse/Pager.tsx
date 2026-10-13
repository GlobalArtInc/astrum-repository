import { IconButton, Select, Text, TextField } from '@radix-ui/themes';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import { useEffect, useState } from 'react';
import { formatNumber, useI18n, useT } from '../../i18n';
import styles from './Pager.module.css';

export const PAGE_SIZES = [20, 40, 50, 100, 250] as const;

export function pageCount(total: number, size: number): number {
  return Math.max(1, Math.ceil(total / size));
}

export function pageSlice<T>(items: readonly T[], page: number, size: number): T[] {
  return items.slice(page * size, (page + 1) * size);
}

function pageButtons(page: number, pages: number): number[] {
  const first = Math.max(0, Math.min(page - 2, pages - 5));
  const last = Math.min(pages - 1, first + 4);
  const numbers: number[] = [];
  for (let index = first; index <= last; index += 1) {
    numbers.push(index);
  }
  return numbers;
}

export function Pager({
  total,
  page,
  size,
  onPage,
  onSize,
}: {
  total: number;
  page: number;
  size: number;
  onPage: (page: number) => void;
  onSize: (size: number) => void;
}) {
  const t = useT();
  const { language } = useI18n();
  const pages = pageCount(total, size);
  const [typed, setTyped] = useState(String(page + 1));
  useEffect(() => setTyped(String(page + 1)), [page]);
  const jump = () => {
    const wanted = Number.parseInt(typed, 10);
    if (Number.isFinite(wanted) && wanted >= 1 && wanted <= pages) {
      onPage(wanted - 1);
    } else {
      setTyped(String(page + 1));
    }
  };
  return (
    <nav className={styles.pager} aria-label={t('browse.pager.label')}>
      <div className={styles.info}>
        <span className={styles.group}>
          <Text size="2">{t('browse.pager.showing')}</Text>
          <Select.Root value={String(size)} onValueChange={(value) => onSize(Number(value))}>
            <Select.Trigger aria-label={t('browse.pager.pageSize')} />
            <Select.Content>
              {PAGE_SIZES.map((option) => (
                <Select.Item key={option} value={String(option)}>
                  {option}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
          <Text size="2">{t('browse.pager.ofItems', { total: formatNumber(language, total) })}</Text>
        </span>
        <span className={styles.group}>
          <Text size="2">{t('browse.pager.page')}</Text>
          <TextField.Root
            className={styles.jump}
            size="3"
            value={typed}
            inputMode="numeric"
            aria-label={t('browse.pager.jump')}
            onChange={(event) => setTyped(event.target.value)}
            onBlur={jump}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                jump();
              }
            }}
          />
          <Text size="2">{t('browse.pager.ofPages', { pages: formatNumber(language, pages) })}</Text>
        </span>
      </div>
      <div className={styles.buttons}>
        <IconButton
          variant="outline"
          color="gray"
          size="2"
          disabled={page === 0}
          aria-label={t('browse.pager.first')}
          onClick={() => onPage(0)}
        >
          <ChevronsLeft size={14} aria-hidden="true" />
        </IconButton>
        <IconButton
          variant="outline"
          color="gray"
          size="2"
          disabled={page === 0}
          aria-label={t('browse.pager.previous')}
          onClick={() => onPage(page - 1)}
        >
          <ChevronLeft size={14} aria-hidden="true" />
        </IconButton>
        {pageButtons(page, pages).map((number) => (
          <IconButton
            key={number}
            variant={number === page ? 'solid' : 'outline'}
            color="gray"
            size="2"
            aria-current={number === page ? 'page' : undefined}
            aria-label={t('browse.pager.pageNumber', { page: number + 1 })}
            onClick={() => onPage(number)}
          >
            {number + 1}
          </IconButton>
        ))}
        <IconButton
          variant="outline"
          color="gray"
          size="2"
          disabled={page >= pages - 1}
          aria-label={t('browse.pager.next')}
          onClick={() => onPage(page + 1)}
        >
          <ChevronRight size={14} aria-hidden="true" />
        </IconButton>
        <IconButton
          variant="outline"
          color="gray"
          size="2"
          disabled={page >= pages - 1}
          aria-label={t('browse.pager.last')}
          onClick={() => onPage(pages - 1)}
        >
          <ChevronsRight size={14} aria-hidden="true" />
        </IconButton>
      </div>
    </nav>
  );
}
