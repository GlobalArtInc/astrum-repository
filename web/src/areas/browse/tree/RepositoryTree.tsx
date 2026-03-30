import { Button, Spinner, Text } from '@radix-ui/themes';
import { ChevronRight, File, Folder, FolderOpen, Package, Tag } from 'lucide-react';
import { type KeyboardEvent, type ReactNode, useEffect, useId, useRef, useState } from 'react';
import { formatNumber, useI18n } from '../../../i18n';
import { describeError } from '../../../ui';
import type { TreeNode } from '../types';
import { keyAction, type Row, sameNode } from './model';
import styles from './Tree.module.css';

export interface RepositoryTreeProps {
  label: string;
  format: string;
  rows: Row[];
  selection: string;
  filter?: string;
  onSelect(node: TreeNode): void;
  onToggle(path: string, open: boolean): void;
  onRetry(folder: string): void;
}

function Icon({ format, node, open }: { format: string; node: TreeNode; open: boolean }) {
  if (format === 'docker') {
    return node.kind === 'asset' ? <Tag size={16} aria-hidden="true" className={styles.icon} /> : null;
  }
  if (node.kind === 'component') {
    return <Package size={16} aria-hidden="true" className={styles.icon} />;
  }
  if (node.kind === 'asset' || (format === 'npm' && !node.name.startsWith('@'))) {
    return <File size={16} aria-hidden="true" className={styles.icon} />;
  }
  return open ? (
    <FolderOpen size={16} aria-hidden="true" className={styles.icon} />
  ) : (
    <Folder size={16} aria-hidden="true" className={styles.icon} />
  );
}

interface Branch {
  row: Row;
  children: Branch[];
}

export function nest(rows: readonly Row[]): Branch[] {
  const top: Branch[] = [];
  const stack: Branch[] = [];
  for (const row of rows) {
    const branch: Branch = { row, children: [] };
    while (stack.length >= row.level) {
      stack.pop();
    }
    const parent = stack[stack.length - 1];
    if (parent) {
      parent.children.push(branch);
    } else {
      top.push(branch);
    }
    if (row.type === 'node') {
      stack.push(branch);
    }
  }
  return top;
}

export function matching(rows: readonly Row[], filter: string): Set<string> | null {
  const wanted = filter.trim().toLowerCase();
  if (wanted === '') {
    return null;
  }
  const shown = new Set<string>();
  const path: string[] = [];
  for (const row of rows) {
    if (row.type !== 'node') {
      continue;
    }
    path.length = row.level - 1;
    path.push(row.node.path);
    if (row.node.name.toLowerCase().includes(wanted)) {
      for (const known of path) {
        shown.add(known);
      }
    }
  }
  return shown;
}

export function RepositoryTree({
  label,
  format,
  rows,
  selection,
  filter = '',
  onSelect,
  onToggle,
  onRetry,
}: RepositoryTreeProps) {
  const { t, language } = useI18n();
  const container = useRef<HTMLDivElement>(null);
  const base = useId();
  const shown = matching(rows, filter);
  const visible = shown ? rows.filter((row) => row.type === 'node' && shown.has(row.node.path)) : rows;
  const nodes = visible.filter((row) => row.type === 'node');
  const selected = nodes.find((row) => sameNode(row.node, selection))?.node.path;
  const [focused, setFocused] = useState<string | undefined>(selected);
  const tabStop = nodes.some((row) => row.node.path === focused)
    ? focused
    : (selected ?? nodes[0]?.node.path);
  const pendingFocus = useRef<string | null>(null);

  useEffect(() => {
    if (selected) {
      setFocused(selected);
    }
  }, [selected]);

  useEffect(() => {
    const wanted = pendingFocus.current;
    if (!wanted || !container.current) {
      return;
    }
    const element = container.current.querySelector<HTMLElement>(`[data-path="${CSS.escape(wanted)}"]`);
    if (element) {
      pendingFocus.current = null;
      element.focus();
      element.scrollIntoView?.({ block: 'nearest' });
    }
  });

  const onKeyDown = (event: KeyboardEvent<HTMLElement>, path: string) => {
    const action = keyAction(visible, path, event.key);
    if (!action) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    switch (action.type) {
      case 'focus':
        pendingFocus.current = action.path;
        setFocused(action.path);
        break;
      case 'expand':
        onToggle(action.path, true);
        break;
      case 'collapse':
        onToggle(action.path, false);
        break;
      case 'select':
        onSelect(action.node);
        if (action.node.kind !== 'asset' && event.key === 'Enter') {
          const row = nodes.find((candidate) => candidate.node.path === action.node.path);
          onToggle(action.node.path, !row?.expanded);
        }
        break;
    }
  };

  const render = (branch: Branch): ReactNode => {
    const { row } = branch;
    if (row.type !== 'node') {
      return (
        <div key={`${row.type}:${row.parent}`} role="none" className={styles.notice}>
          {row.type === 'loading' && (
            <Text size="2" color="gray" className={styles.noticeText}>
              <Spinner size="1" />
              {t('browse.tree.loading')}
            </Text>
          )}
          {row.type === 'empty' && (
            <Text size="2" color="gray">
              {t('browse.tree.emptyFolder')}
            </Text>
          )}
          {row.type === 'truncated' && (
            <Text size="1" color="amber" role="note">
              {t('browse.tree.truncated', { count: row.shown, shown: formatNumber(language, row.shown) })}
            </Text>
          )}
          {row.type === 'error' && (
            <Text size="2" color="red" className={styles.noticeText}>
              {t('browse.tree.error', { detail: describeError(t, row.error) })}
              <Button size="1" variant="ghost" onClick={() => onRetry(row.parent)}>
                {t('browse.tree.retry')}
              </Button>
            </Text>
          )}
        </div>
      );
    }
    const { node } = row;
    const isSelected = node.path === selected;
    const open = row.expanded === true;
    const branchNode = row.expanded !== undefined;
    return (
      <div
        key={node.path}
        role="treeitem"
        data-path={node.path}
        aria-level={row.level}
        aria-posinset={row.position}
        aria-setsize={row.size}
        aria-selected={isSelected}
        aria-expanded={branchNode ? open : undefined}
        aria-labelledby={`${base}${encodeURIComponent(node.path)}`}
        tabIndex={node.path === tabStop ? 0 : -1}
        className={styles.container}
        onKeyDown={(event) => onKeyDown(event, node.path)}
        onFocus={(event) => {
          if (event.target === event.currentTarget) {
            setFocused(node.path);
          }
        }}
      >
        {/* biome-ignore lint/a11y/noStaticElementInteractions: the tree item handles the keyboard */}
        {/* biome-ignore lint/a11y/useKeyWithClickEvents: the tree item handles the keyboard */}
        <div
          className={styles.item}
          data-selected={isSelected}
          onClick={(event) => {
            event.stopPropagation();
            setFocused(node.path);
            onSelect(node);
            if (branchNode && !open) {
              onToggle(node.path, true);
            }
          }}
        >
          <span
            className={styles.twisty}
            data-open={open}
            data-hidden={!branchNode}
            aria-hidden="true"
            onClick={(event) => {
              if (branchNode) {
                event.stopPropagation();
                setFocused(node.path);
                onToggle(node.path, !open);
              }
            }}
          >
            <ChevronRight size={16} />
          </span>
          <Icon format={format} node={node} open={open || isSelected} />
          <span
            id={`${base}${encodeURIComponent(node.path)}`}
            className={styles.label}
            data-mono={format === 'docker' && node.kind === 'asset'}
          >
            {node.name}
          </span>
        </div>
        {branch.children.length > 0 && (
          <div role="group" className={styles.group}>
            {branch.children.map(render)}
          </div>
        )}
      </div>
    );
  };

  return (
    <div ref={container} role="tree" aria-label={label} className={styles.tree}>
      {shown && nodes.length === 0 ? (
        <Text as="p" size="2" color="gray" className={styles.noMatches}>
          {t('browse.tree.noMatches')}
        </Text>
      ) : (
        nest(visible).map(render)
      )}
    </div>
  );
}
