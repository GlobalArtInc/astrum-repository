

/**
 * Crowd settings data
 */
export interface CrowdConfig {
  enabled: boolean;
  realmActive: boolean;
  url: string;
  useTrustStoreForUrl: boolean;
  applicationName: string;
  applicationPassword: string;
  timeout?: number;
}

/**
 * Default Crowd configuration values
 */
export const DEFAULT_CROWD_CONFIG: CrowdConfig = {
  enabled: false,
  realmActive: false,
  url: '',
  useTrustStoreForUrl: false,
  applicationName: '',
  applicationPassword: '',
  timeout: undefined,
};

/**
 * Page props
 */
export interface CrowdPageProps {
  className?: string;
}


