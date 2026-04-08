import { Button, Tabs } from '@radix-ui/themes';
import { ArrowLeft, Check, ChevronRight, type LucideIcon, Pencil } from 'lucide-react';
import type { ReactNode } from 'react';
import { Link, useSearchParams } from 'react-router';
import { useT } from '../../../i18n';
import styles from './Profile.module.css';

export interface ProfileTab {
  value: string;
  label: string;
  icon: LucideIcon;
  content: ReactNode;
}

export function ProfileTabs({ tabs }: { tabs: ProfileTab[] }) {
  const [params, setParams] = useSearchParams();
  const fallback = tabs[0]?.value ?? '';
  const asked = params.get('tab');
  const current = tabs.some((tab) => tab.value === asked) ? (asked ?? fallback) : fallback;
  return (
    <Tabs.Root
      className={styles.root}
      value={current}
      onValueChange={(value) =>
        setParams(
          (previous) => {
            const next = new URLSearchParams(previous);
            if (value === fallback) {
              next.delete('tab');
            } else {
              next.set('tab', value);
            }
            return next;
          },
          { replace: true },
        )
      }
    >
      <div className={styles.tabsScroll}>
        <Tabs.List className={styles.tabList}>
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <Tabs.Trigger key={tab.value} value={tab.value} className={styles.tab}>
                <span className={styles.tabInner}>
                  <Icon size={14} aria-hidden="true" />
                  {tab.label}
                </span>
              </Tabs.Trigger>
            );
          })}
        </Tabs.List>
      </div>
      {tabs.map((tab) => (
        <Tabs.Content key={tab.value} value={tab.value} className={styles.panel}>
          {tab.content}
        </Tabs.Content>
      ))}
    </Tabs.Root>
  );
}

export function ProfileActions({
  edit,
  editLabel,
  back,
}: {
  edit?: string;
  editLabel: string;
  back: string;
}) {
  const t = useT();
  return (
    <>
      {edit && (
        <Button asChild size="3" className={styles.bigButton}>
          <Link to={edit}>
            <Pencil size={16} aria-hidden="true" />
            {editLabel}
          </Link>
        </Button>
      )}
      <Button asChild size="3" variant="ghost" color="gray" className={styles.backButton}>
        <Link to={back}>
          <ArrowLeft size={16} aria-hidden="true" />
          {t('security.profile.back')}
        </Link>
      </Button>
    </>
  );
}

export function ProfileSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className={styles.section}>
      <h2 className={styles.sectionTitle}>{title}</h2>
      <div role="list" className={styles.rows}>
        {children}
      </div>
    </section>
  );
}

export function ProfileRow({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div role="listitem" className={styles.row}>
      <span className={styles.rowLabel}>{label}</span>
      <div className={styles.rowValue}>{children}</div>
    </div>
  );
}

export function Separator() {
  return <span className={styles.separator} aria-hidden="true" />;
}

export function ActionChips({ items }: { items: { label: string; on: boolean }[] }) {
  return (
    <div className={styles.chips}>
      {items.map((item) => (
        <div key={item.label} className={item.on ? `${styles.chip} ${styles.chipOn}` : styles.chip}>
          <span className={item.on ? `${styles.box} ${styles.boxOn}` : styles.box} aria-hidden="true">
            {item.on && <Check size={12} />}
          </span>
          <span>{item.label}</span>
          <span className="visually-hidden">{item.on ? '✓' : '—'}</span>
        </div>
      ))}
    </div>
  );
}

export function ProfileCard({ children }: { children: ReactNode }) {
  return <div className={styles.card}>{children}</div>;
}

export function LinkTable<T>({
  label,
  columns,
  rows,
  getRowId,
  empty,
}: {
  label: string;
  columns: { id: string; header: string; cell: (row: T) => ReactNode }[];
  rows: readonly T[];
  getRowId(row: T): string;
  empty: ReactNode;
}) {
  if (rows.length === 0) {
    return <div className={styles.empty}>{empty}</div>;
  }
  return (
    <div className={styles.tableWrap} role="region" aria-label={label}>
      <table className={styles.table}>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.id}>
                <span>{column.header}</span>
              </th>
            ))}
            <th />
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={getRowId(row)}>
              {columns.map((column) => (
                <td key={column.id}>{column.cell(row)}</td>
              ))}
              <td className={styles.chevronCell}>
                <ChevronRight size={14} aria-hidden="true" />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function SourceBadge({ children }: { children: ReactNode }) {
  return <span className={styles.sourceBadge}>{children}</span>;
}

export function OnlineStatus({ label, active }: { label: string; active: boolean }) {
  return (
    <span className={styles.status}>
      <span className={active ? `${styles.dot} ${styles.dotOn}` : styles.dot} aria-hidden="true" />
      {label}
    </span>
  );
}
