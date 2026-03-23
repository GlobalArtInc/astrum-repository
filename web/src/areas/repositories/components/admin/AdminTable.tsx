import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react';
import type { KeyboardEvent, ReactNode } from 'react';
import styles from './admin.module.css';
import type { Direction } from './FilterPanel';

export interface AdminColumn<T> {
  id: string;
  header: ReactNode;
  cell: (row: T) => ReactNode;
  sortable?: boolean;
  width?: string;
  label?: string;
}

export function AdminTable<T>({
  label,
  columns,
  rows,
  getRowId,
  sort,
  onSort,
  onOpen,
  openLabel,
  empty,
  footer,
}: {
  label: string;
  columns: AdminColumn<T>[];
  rows: readonly T[];
  getRowId(row: T): string;
  sort?: { column: string; direction: Direction };
  onSort?(column: string): void;
  onOpen?(row: T): void;
  openLabel?(row: T): string;
  empty?: ReactNode;
  footer?: ReactNode;
}) {
  const open = (row: T) => onOpen?.(row);
  const key = (row: T) => (event: KeyboardEvent<HTMLTableRowElement>) => {
    if (event.target === event.currentTarget && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      open(row);
    }
  };
  return (
    <div className={styles.tableWrap}>
      <table className={styles.table}>
        <caption className="visually-hidden">{label}</caption>
        <thead>
          <tr>
            {columns.map((column) => {
              const sorted = sort?.column === column.id ? sort.direction : undefined;
              const Arrow = sorted === 'asc' ? ArrowUp : sorted === 'desc' ? ArrowDown : ArrowUpDown;
              return (
                <th
                  key={column.id}
                  style={column.width ? { width: column.width } : undefined}
                  aria-sort={sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined}
                  aria-label={column.label}
                >
                  {column.sortable && onSort ? (
                    <button type="button" className={styles.sort} onClick={() => onSort(column.id)}>
                      {column.header}
                      <Arrow size={14} aria-hidden="true" className={styles.sortIcon} />
                    </button>
                  ) : (
                    column.header
                  )}
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr
              key={getRowId(row)}
              data-link={onOpen ? true : undefined}
              tabIndex={onOpen ? 0 : undefined}
              aria-label={openLabel?.(row)}
              onClick={(event) => {
                if (!(event.target as HTMLElement).closest('button, a, [role="menuitem"]')) {
                  open(row);
                }
              }}
              onKeyDown={onOpen ? key(row) : undefined}
            >
              {columns.map((column) => (
                <td key={column.id}>{column.cell(row)}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {rows.length === 0 && empty && <div className={styles.empty}>{empty}</div>}
      {footer}
    </div>
  );
}

export function compareText(left: string, right: string): number {
  return left.localeCompare(right, undefined, { numeric: true, sensitivity: 'base' });
}
