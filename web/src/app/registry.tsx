import { createContext, type ReactNode, useContext, useMemo } from 'react';
import { useSession } from '../auth/state';
import type { Area, MenuEntry, SearchProvider, SettingsTile } from './area';

const byOrder = <T extends { order: number }>(left: T, right: T) => left.order - right.order;

export interface Registry {
  areas: Area[];
  menu: MenuEntry[];
  settings: SettingsTile[];
  search: SearchProvider[];
}

export function buildRegistry(areas: Area[]): Registry {
  return {
    areas,
    menu: areas.flatMap((area) => area.menu ?? []).sort(byOrder),
    settings: areas.flatMap((area) => area.settings ?? []).sort(byOrder),
    search: areas.flatMap((area) => area.search ?? []).sort(byOrder),
  };
}

const RegistryContext = createContext<Registry>(buildRegistry([]));

export function RegistryProvider({ areas, children }: { areas: Area[]; children: ReactNode }) {
  const registry = useMemo(() => buildRegistry(areas), [areas]);
  return <RegistryContext.Provider value={registry}>{children}</RegistryContext.Provider>;
}

export function useRegistry(): Registry {
  return useContext(RegistryContext);
}

export function useVisibleSettings(): SettingsTile[] {
  const { settings } = useRegistry();
  const { can } = useSession();
  return useMemo(() => settings.filter((tile) => can(tile.need)), [settings, can]);
}

export function useVisibleMenu(): MenuEntry[] {
  const { menu } = useRegistry();
  const { can, user } = useSession();
  const settingsTiles = useVisibleSettings();
  return useMemo(
    () =>
      menu.filter((entry) => {
        if (entry.requiresUser && !user) {
          return false;
        }
        if (!can(entry.need)) {
          return false;
        }
        return entry.visible ? entry.visible({ can, signedIn: user !== null, settingsTiles }) : true;
      }),
    [menu, can, user, settingsTiles],
  );
}

export function useVisibleSearch(): SearchProvider[] {
  const { search } = useRegistry();
  const { can } = useSession();
  return useMemo(() => search.filter((provider) => can(provider.need)), [search, can]);
}
