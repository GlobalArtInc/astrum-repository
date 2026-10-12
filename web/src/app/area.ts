import type { LucideIcon } from 'lucide-react';
import type { ComponentType } from 'react';
import type { Need } from '../auth/permissions';

export interface AreaRoute {
  path: string;
  page?: ComponentType;
  load?: () => Promise<{ default: ComponentType }>;
  need?: Need;
  requiresUser?: boolean;
}

export interface MenuContext {
  can(need: Need | undefined): boolean;
  signedIn: boolean;
  settingsTiles: SettingsTile[];
}

export interface MenuEntry {
  id: string;
  to: string;
  label: string;
  icon: LucideIcon;
  group: 'main' | 'admin';
  order: number;
  need?: Need;
  requiresUser?: boolean;
  visible?: (context: MenuContext) => boolean;
  activeFor?: string[];
}

export type SettingsSection = 'repository' | 'security' | 'support' | 'system';

export interface SettingsTile {
  id: string;
  to: string;
  need?: Need;
  section?: SettingsSection;
  title?: string;
  description?: string;
  icon?: LucideIcon;
  order?: number;
}

export const MENU_ORDER = {
  dashboard: 10,
  search: 20,
  browse: 30,
  upload: 40,
  api: 50,
  settings: 900,
} as const;

export interface SearchResultsProps {
  query: string;
  close(): void;
}

export interface SearchProvider {
  id: string;
  order: number;
  need?: Need;
  Results: ComponentType<SearchResultsProps>;
  submit?: (query: string) => string | null;
}

export interface Area {
  id: string;
  routes?: AreaRoute[];
  menu?: MenuEntry[];
  settings?: SettingsTile[];
  search?: SearchProvider[];
  watchers?: ComponentType[];
}
