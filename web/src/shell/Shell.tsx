import { useCallback, useState } from 'react';
import { Navigate, Outlet, useLocation } from 'react-router';
import { loginPath } from '../auth/returnTo';
import { useSession } from '../auth/state';
import { useT } from '../i18n';
import { COMPACT, readStored, useMediaQuery, writeStored } from '../ui/hooks';
import { Header } from './Header';
import { MobileDrawer } from './MobileDrawer';
import styles from './Shell.module.css';
import { SideMenu } from './SideMenu';
import { SearchPanel, useSearchShortcut } from './search';

export const SIDEBAR_KEY = 'astrum.sidebar.open';

export function Shell() {
  const t = useT();
  const { user, anonymousAccess, state } = useSession();
  const location = useLocation();
  const compact = useMediaQuery(COMPACT);
  const [sidebarOpen, setSidebarOpen] = useState(() => readStored(SIDEBAR_KEY) !== 'false');
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const toggleSearch = useCallback(() => setSearchOpen((open) => !open), []);
  useSearchShortcut(toggleSearch);

  if (!user && !anonymousAccess) {
    return <Navigate to={loginPath(location.pathname + location.search + location.hash)} replace />;
  }

  const toggleMenu = () => {
    if (compact) {
      setDrawerOpen(true);
      return;
    }
    const next = !sidebarOpen;
    writeStored(SIDEBAR_KEY, String(next));
    setSidebarOpen(next);
  };

  return (
    <>
      <a href="#main" className={styles.skip}>
        {t('shell.main.skipToContent')}
      </a>
      <Header
        compact={compact}
        collapsed={!sidebarOpen}
        onToggleMenu={toggleMenu}
        onOpenSearch={() => setSearchOpen(true)}
      />
      <div className={styles.body}>
        {!compact && <SideMenu collapsed={!sidebarOpen} version={state.version} />}
        <main id="main" tabIndex={-1} className={styles.main}>
          <Outlet />
        </main>
      </div>
      {compact && <MobileDrawer open={drawerOpen} onOpenChange={setDrawerOpen} version={state.version} />}
      <SearchPanel open={searchOpen} onOpenChange={setSearchOpen} />
    </>
  );
}
