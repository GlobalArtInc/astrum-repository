import { Button, Spinner, Text } from '@radix-ui/themes';
import { ChevronRight, File, Folder, FolderOpen, Info, Package } from 'lucide-react';
import { type KeyboardEvent, useEffect, useRef, useState } from 'react';
import { formatNumber, useI18n } from '../../../i18n';
import { describeError } from '../../../ui';
import type { TreeNode } from '../types';
import { keyAction, type Row, sameNode } from './model';
import styles from './Tree.module.css';

export interface RepositoryTreeProps {
  label: string;
  rows: Row[];
  selection: string;
  onSelect(node: TreeNode): void;
  onToggle(path: string, open: boolean): void;
  onRetry(folder: string): void;
  onDetails?: (node: TreeNode) => void;
}

function Icon({ node, open }: { node: TreeNode; open: boolean }) {
  if (node.kind === 'component') {
    return <Package size={15} aria-hidden="true" className={styles.componentIcon} />;
  }
  if (node.kind === 'asset') {
    return <File size={15} aria-hidden="true" className={styles.fileIcon} />;
  }
  return open ? (
    <FolderOpen size={15} aria-hidden="true" className={styles.folderIcon} />
  ) : (
    <Folder size={15} aria-hidden="true" className={styles.folderIcon} />
  );
}

function indent(level: number, offset = '0px') {
  return { paddingInlineStart: `calc(${level - 1} * var(--tree-indent) + var(--space-2) + ${offset})` };
}

export function RepositoryTree({
  label,
  rows,
  selection,
  onSelect,
  onToggle,
  onRetry,
  onDetails,
}: RepositoryTreeProps) {
  const { t, language } = useI18n();
  const container = useRef<HTMLDivElement>(null);
  const nodes = rows.filter((row) => row.type === 'node');
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
    const action = keyAction(rows, path, event.key);
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

  return (
    <div ref={container} role="tree" aria-label={label} className={styles.tree}>
      {rows.map((row) => {
        if (row.type !== 'node') {
          const key = `${row.type}:${row.parent}`;
          return (
            <div key={key} role="none" className={styles.notice} style={indent(row.level, '26px')}>
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
        const branch = row.expanded !== undefined;
        return (
          <div
            key={node.path}
            role="treeitem"
            data-path={node.path}
            aria-level={row.level}
            aria-posinset={row.position}
            aria-setsize={row.size}
            aria-selected={isSelected}
            aria-expanded={branch ? open : undefined}
            tabIndex={node.path === tabStop ? 0 : -1}
            className={styles.item}
            data-selected={isSelected}
            style={indent(row.level)}
            onKeyDown={(event) => onKeyDown(event, node.path)}
            onFocus={(event) => {
              if (event.target === event.currentTarget) {
                setFocused(node.path);
              }
            }}
            onClick={(event) => {
              event.stopPropagation();
              setFocused(node.path);
              onSelect(node);
              if (branch && !open) {
                onToggle(node.path, true);
              }
            }}
          >
            <span
              className={styles.twisty}
              data-open={open}
              data-hidden={!branch}
              aria-hidden="true"
              onClick={(event) => {
                if (branch) {
                  event.stopPropagation();
                  setFocused(node.path);
                  onToggle(node.path, !open);
                }
              }}
            >
              <ChevronRight size={14} />
            </span>
            <Icon node={node} open={open} />
            <span className={styles.label}>{node.name}</span>
            {node.kind !== 'asset' && node.childCount !== undefined && (
              <span className={styles.count} aria-hidden="true">
                {formatNumber(language, node.childCount)}
              </span>
            )}
            {onDetails && branch && (
              <button
                type="button"
                tabIndex={-1}
                className={styles.info}
                aria-label={t('browse.tree.detailsOf', { name: node.name })}
                onClick={(event) => {
                  event.stopPropagation();
                  onDetails(node);
                }}
              >
                <Info size={16} aria-hidden="true" />
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}
