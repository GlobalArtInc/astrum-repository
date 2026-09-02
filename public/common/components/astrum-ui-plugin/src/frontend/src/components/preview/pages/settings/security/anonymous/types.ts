

/**
 * Anonymous access settings data model
 */
export interface AnonymousSettings {
  enabled: boolean;
  userId: string;
  realmName: string;
}

/**
 * Realm type for the anonymous user dropdown
 */
export interface RealmType {
  id: string;
  name: string;
}

/**
 * Props for AnonymousPage component
 */
export interface AnonymousPageProps {
  className?: string;
}

/**
 * Default anonymous settings
 */
export const DEFAULT_ANONYMOUS_SETTINGS: AnonymousSettings = {
  enabled: false,
  userId: 'anonymous',
  realmName: 'AstrumAuthorizingRealm',
};


