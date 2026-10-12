import { Tooltip } from '@radix-ui/themes';
import { Fragment } from 'react';
import { NavLink, useLocation } from 'react-router';
import type { MenuEntry } from '../app/area';
import { useVisibleMenu } from '../app/registry';
import { useT } from '../i18n';
import styles from './Shell.module.css';

export function isActive(entry: MenuEntry, pathname: string): boolean {
  const prefixes = [entry.to, ...(entry.activeFor ?? [])];
  return prefixes.some((prefix) =>
    prefix === '/' ? pathname === '/' : pathname === prefix || pathname.startsWith(`${prefix}/`),
  );
}

export function MenuItems({
  collapsed = false,
  onNavigate,
}: {
  collapsed?: boolean;
  onNavigate?: () => void;
}) {
  const t = useT();
  const entries = useVisibleMenu();
  const { pathname } = useLocation();
  const groups = (['main', 'admin'] as const)
    .map((group) => entries.filter((entry) => entry.group === group))
    .filter((group) => group.length > 0);
  return (
    <>
      {groups.map((group, index) => (
        <Fragment key={group[0]?.group}>
          {index > 0 && <hr className={styles.divider} />}
          <ul className={styles.navList}>
            {group.map((entry) => {
              const Icon = entry.icon;
              const label = t(entry.label);
              const link = (
                <NavLink
                  to={entry.to}
                  className={styles.item}
                  aria-current={isActive(entry, pathname) ? 'page' : undefined}
                  aria-label={collapsed ? label : undefined}
                  onClick={onNavigate}
                  end={entry.to === '/'}
                >
                  <Icon size={18} aria-hidden="true" />
                  {!collapsed && <span>{label}</span>}
                </NavLink>
              );
              return (
                <li key={entry.id}>
                  {collapsed ? (
                    <Tooltip content={label} side="right">
                      {link}
                    </Tooltip>
                  ) : (
                    link
                  )}
                </li>
              );
            })}
          </ul>
        </Fragment>
      ))}
    </>
  );
}

export function SideMenu({ collapsed, version }: { collapsed: boolean; version: string }) {
  const t = useT();
  return (
    <aside className={styles.side} data-collapsed={collapsed}>
      <nav aria-label={t('shell.menu.label')} className={styles.nav}>
        <MenuItems collapsed={collapsed} />
      </nav>
      {!collapsed && <div className={styles.version}>{t('shell.menu.version', { version })}</div>}
    </aside>
  );
}
