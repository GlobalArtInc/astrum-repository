import { Button, Checkbox, Dialog, IconButton, Text } from '@radix-ui/themes';
import { ChevronDown, ChevronLeft, ChevronUp, RefreshCw, X } from 'lucide-react';
import { type CSSProperties, type ReactNode, useId, useState } from 'react';
import { useT } from '../i18n';
import styles from './FilterLayout.module.css';
import { useMediaQuery } from './hooks';

export const NARROW = '(max-width: 519px)';

export interface FilterLayoutProps {
  label: string;
  contentLabel: string;
  width: number;
  gap: number;
  filters: ReactNode;
  header?: ReactNode;
  sheetOpen: boolean;
  onSheetOpenChange(open: boolean): void;
  onResetAll(): void;
  className?: string;
  children: ReactNode;
}

export function FilterLayout({
  label,
  contentLabel,
  width,
  gap,
  filters,
  header,
  sheetOpen,
  onSheetOpenChange,
  onResetAll,
  className,
  children,
}: FilterLayoutProps) {
  const t = useT();
  const narrow = useMediaQuery(NARROW);
  const style = { '--filter-width': `${width}px`, '--filter-gap': `${gap}px` } as CSSProperties;
  return (
    <div className={className ? `${styles.layout} ${className}` : styles.layout} style={style}>
      {header && <div className={styles.header}>{header}</div>}
      {!narrow && (
        <aside className={styles.aside} aria-label={label}>
          {filters}
        </aside>
      )}
      <section className={styles.main} aria-label={contentLabel}>
        {children}
      </section>
      {narrow && (
        <Dialog.Root open={sheetOpen} onOpenChange={onSheetOpenChange}>
          <Dialog.Content className={styles.sheet} aria-describedby={undefined}>
            <div className={styles.sheetHead}>
              <Dialog.Title size="4" className={styles.sheetTitle}>
                {t('ui.filters.title')}
              </Dialog.Title>
              <div className={styles.sheetActions}>
                <Button type="button" variant="ghost" onClick={onResetAll}>
                  {t('ui.filters.resetAll')}
                </Button>
                <Dialog.Close>
                  <IconButton type="button" variant="ghost" aria-label={t('ui.filters.close')}>
                    <X size={18} aria-hidden="true" />
                  </IconButton>
                </Dialog.Close>
              </div>
            </div>
            <div className={styles.sheetBody}>{filters}</div>
          </Dialog.Content>
        </Dialog.Root>
      )}
    </div>
  );
}

export function ResetFiltersButton({ onClick }: { onClick(): void }) {
  const t = useT();
  return (
    <Button type="button" variant="outline" color="gray" className={styles.reset} onClick={onClick}>
      <RefreshCw size={12} aria-hidden="true" />
      {t('ui.filters.reset')}
    </Button>
  );
}

export function FilterSection({
  title,
  weight = 'bold',
  children,
}: {
  title: string;
  weight?: 'bold' | 'semibold';
  children: ReactNode;
}) {
  return (
    <div className={styles.section}>
      <Text as="p" size="2" className={weight === 'bold' ? styles.titleBold : styles.titleSemibold}>
        {title}
      </Text>
      {children}
    </div>
  );
}

export interface CheckOption {
  value: string;
  label: string;
  count?: number;
}

export function CheckList({
  options,
  selected,
  onChange,
  visible,
  clearable = false,
  className,
}: {
  options: CheckOption[];
  selected: readonly string[];
  onChange(next: string[]): void;
  visible?: number;
  clearable?: boolean;
  className?: string;
}) {
  const t = useT();
  const id = useId();
  const [expanded, setExpanded] = useState(false);
  const limited = visible !== undefined && options.length > visible && !expanded;
  const shown = limited ? options.slice(0, visible) : options;
  const toggle = (value: string, checked: boolean) => {
    onChange(checked ? [...selected, value] : selected.filter((item) => item !== value));
  };
  return (
    <div className={className ? `${styles.checks} ${className}` : styles.checks}>
      {clearable && selected.length > 0 && (
        <button type="button" className={styles.clear} onClick={() => onChange([])}>
          <ChevronLeft size={14} aria-hidden="true" />
          {t('ui.filters.clear')}
        </button>
      )}
      {shown.map((option, index) => {
        const inputId = `${id}-${index}`;
        return (
          <div key={option.value} className={styles.check}>
            <Checkbox
              id={inputId}
              checked={selected.includes(option.value)}
              onCheckedChange={(checked) => toggle(option.value, checked === true)}
            />
            <Text as="label" size="2" htmlFor={inputId}>
              {option.label}
            </Text>
            {option.count !== undefined && <span className={styles.count}>{option.count}</span>}
          </div>
        );
      })}
      {visible !== undefined && options.length > visible && (
        <Button
          type="button"
          size="1"
          variant="ghost"
          className={styles.more}
          onClick={() => setExpanded(!expanded)}
        >
          {expanded ? (
            <ChevronUp size={12} aria-hidden="true" />
          ) : (
            <ChevronDown size={12} aria-hidden="true" />
          )}
          <Text size="2">{expanded ? t('ui.filters.less') : t('ui.filters.more')}</Text>
        </Button>
      )}
    </div>
  );
}
