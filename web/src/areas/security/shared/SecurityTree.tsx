import { TextField } from '@radix-ui/themes';
import {
  ChevronDown,
  ChevronRight,
  ChevronsDown,
  ChevronsUp,
  Filter,
  Key,
  Search,
  Shield,
} from 'lucide-react';
import { useMemo, useState } from 'react';
import { useT } from '../../../i18n';
import type { ContentSelector, Privilege, Role } from '../api';
import styles from './SecurityTree.module.css';

export interface TreeNode {
  key: string;
  kind: 'role' | 'privilege' | 'selector';
  name: string;
  inherited: boolean;
  permission?: string;
  resource?: string;
  description?: string;
  children: TreeNode[];
}

function actionsOf(privilege: Privilege): string | undefined {
  if (privilege.type === 'wildcard') {
    return privilege.pattern;
  }
  return privilege.actions?.map((action) => action.toLowerCase()).join(',');
}

export function buildTree(
  roots: readonly string[],
  roles: ReadonlyMap<string, Role>,
  privileges: ReadonlyMap<string, Privilege>,
  selectors: ReadonlyMap<string, ContentSelector>,
): TreeNode[] {
  const privilegeNode = (name: string, path: string, inherited: boolean): TreeNode | null => {
    const privilege = privileges.get(name);
    if (!privilege) {
      return null;
    }
    const key = `${path}/p:${name}`;
    const selector = privilege.contentSelector ? selectors.get(privilege.contentSelector) : undefined;
    return {
      key,
      kind: 'privilege',
      name,
      inherited,
      permission: actionsOf(privilege),
      resource: privilege.type.startsWith('repository-') ? privilege.repository : undefined,
      description: privilege.description,
      children: selector
        ? [
            {
              key: `${key}/s:${selector.name}`,
              kind: 'selector',
              name: selector.name,
              inherited,
              resource: selector.expression,
              description: selector.description ?? undefined,
              children: [],
            },
          ]
        : [],
    };
  };
  const roleNode = (id: string, path: string, inherited: boolean, trail: readonly string[]): TreeNode => {
    const role = roles.get(id);
    const key = `${path}/r:${id}`;
    if (!role || trail.includes(id)) {
      return { key, kind: 'role', name: id, inherited, description: role?.description, children: [] };
    }
    const next = [...trail, id];
    return {
      key,
      kind: 'role',
      name: id,
      inherited,
      description: role.description,
      children: [
        ...role.roles.map((child) => roleNode(child, key, true, next)),
        ...role.privileges.flatMap((name) => {
          const node = privilegeNode(name, key, inherited);
          return node ? [node] : [];
        }),
      ],
    };
  };
  return roots.map((id) => roleNode(id, '', false, []));
}

function matches(node: TreeNode, words: string): boolean {
  return [node.name, node.permission, node.resource]
    .filter(Boolean)
    .some((text) => (text ?? '').toLowerCase().includes(words));
}

function filterTree(nodes: TreeNode[], words: string): TreeNode[] {
  if (!words) {
    return nodes;
  }
  return nodes.flatMap((node) => {
    const children = filterTree(node.children, words);
    return matches(node, words) || children.length > 0 ? [{ ...node, children }] : [];
  });
}

function allKeys(nodes: TreeNode[]): string[] {
  return nodes.flatMap((node) => (node.children.length > 0 ? [node.key, ...allKeys(node.children)] : []));
}

const ICONS = { role: Shield, privilege: Key, selector: Filter };

export function SecurityTree({ label, nodes }: { label: string; nodes: TreeNode[] }) {
  const t = useT();
  const [query, setQuery] = useState('');
  const [open, setOpen] = useState<ReadonlySet<string>>(() => new Set(nodes.map((node) => node.key)));
  const words = query.trim().toLowerCase();
  const shown = useMemo(() => filterTree(nodes, words), [nodes, words]);
  const expanded = (key: string) => (words ? true : open.has(key));

  const renderNode = (node: TreeNode, depth: number) => {
    const Icon = ICONS[node.kind];
    const isOpen = expanded(node.key);
    const hasChildren = node.children.length > 0;
    return (
      <div key={node.key} className={styles.group}>
        <div
          role="treeitem"
          tabIndex={-1}
          aria-expanded={hasChildren ? isOpen : undefined}
          aria-level={depth + 1}
          aria-selected={false}
          className={styles.row}
        >
          <div className={styles.name} style={{ paddingLeft: depth * 20 }}>
            {hasChildren ? (
              <button
                type="button"
                className={styles.toggle}
                aria-label={isOpen ? t('security.tree.collapse') : t('security.tree.expand')}
                onClick={() => {
                  const next = new Set(open);
                  if (isOpen) {
                    next.delete(node.key);
                  } else {
                    next.add(node.key);
                  }
                  setOpen(next);
                }}
              >
                {isOpen ? (
                  <ChevronDown size={14} aria-hidden="true" />
                ) : (
                  <ChevronRight size={14} aria-hidden="true" />
                )}
              </button>
            ) : (
              <span className={styles.spacer} />
            )}
            <Icon size={16} aria-hidden="true" className={styles[node.kind]} />
            <span className={styles.label}>{node.name}</span>
            {node.inherited && depth > 0 && (
              <span className={styles.inherited}>{t('security.tree.inherited')}</span>
            )}
          </div>
          <div className={styles.cell}>
            {node.permission && <span className={styles.actions}>{node.permission}</span>}
          </div>
          <div className={styles.cell}>
            {node.resource && <span className={styles.resource}>{node.resource}</span>}
          </div>
          <div className={styles.description}>{node.description}</div>
        </div>
        {hasChildren && isOpen && node.children.map((child) => renderNode(child, depth + 1))}
      </div>
    );
  };

  return (
    <div className={styles.outer}>
      <div className={styles.inner}>
        <div className={styles.toolbar}>
          <TextField.Root
            size="2"
            className={styles.search}
            value={query}
            placeholder={t('security.tree.search')}
            aria-label={t('security.tree.search')}
            onChange={(event) => setQuery(event.target.value)}
          >
            <TextField.Slot>
              <Search size={16} aria-hidden="true" />
            </TextField.Slot>
          </TextField.Root>
          <span className={styles.divider} aria-hidden="true" />
          <button
            type="button"
            className={`${styles.tool} ${styles.toolOn}`}
            aria-label={t('security.tree.expandAll')}
            title={t('security.tree.expandAll')}
            onClick={() => setOpen(new Set(allKeys(nodes)))}
          >
            <ChevronsDown size={16} aria-hidden="true" />
          </button>
          <button
            type="button"
            className={styles.tool}
            aria-label={t('security.tree.collapseAll')}
            title={t('security.tree.collapseAll')}
            onClick={() => setOpen(new Set())}
          >
            <ChevronsUp size={16} aria-hidden="true" />
          </button>
        </div>
        <div className={styles.header} aria-hidden="true">
          <span>{t('security.tree.name')}</span>
          <span>{t('security.tree.permissions')}</span>
          <span>{t('security.tree.resource')}</span>
          <span>{t('security.tree.description')}</span>
        </div>
        <div role="tree" aria-label={label} className={styles.tree}>
          {shown.length === 0 ? (
            <p className={styles.empty}>{t('security.tree.empty')}</p>
          ) : (
            shown.map((node) => renderNode(node, 0))
          )}
        </div>
      </div>
    </div>
  );
}
