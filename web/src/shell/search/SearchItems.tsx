import { type ReactNode, useId } from 'react';
import { Link } from 'react-router';
import { useSearchPanel } from './context';
import styles from './SearchPanel.module.css';

export function SearchGroup({ title, children }: { title: string; children: ReactNode }) {
  const id = useId();
  return (
    <div className={styles.group} role="group" aria-labelledby={id}>
      <div id={id} className={styles.groupTitle}>
        {title}
      </div>
      <ul role="none" className={styles.groupList}>
        {children}
      </ul>
    </div>
  );
}

export function SearchItem({
  to,
  children,
  onSelect,
}: {
  to: string;
  children: ReactNode;
  onSelect?: () => void;
}) {
  const { close } = useSearchPanel();
  const id = useId();
  return (
    <li role="none">
      <Link
        id={id}
        to={to}
        role="option"
        aria-selected={false}
        data-search-item=""
        className={styles.item}
        onClick={() => {
          onSelect?.();
          close();
        }}
      >
        {children}
      </Link>
    </li>
  );
}
