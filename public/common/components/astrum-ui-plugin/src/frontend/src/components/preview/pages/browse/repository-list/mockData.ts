

import type {
  Repository,
  HealthCheckStatus,
  FirewallStatus,
} from './repository-list.types';

/**
 * Mock repository data for testing.
 */
export const mockRepositories: Repository[] = [
  {
    name: 'maven-central',
    type: 'proxy',
    format: 'maven2',
    status: { repositoryName: 'maven-central', online: true, description: 'Ready to Connect' },
    url: 'http://localhost:8081/repository/maven-central/',
  },
  {
    name: 'maven-public',
    type: 'group',
    format: 'maven2',
    status: { repositoryName: 'maven-public', online: true },
    url: 'http://localhost:8081/repository/maven-public/',
  },
  {
    name: 'maven-releases',
    type: 'hosted',
    format: 'maven2',
    status: { repositoryName: 'maven-releases', online: true },
    url: 'http://localhost:8081/repository/maven-releases/',
  },
  {
    name: 'maven-snapshots',
    type: 'hosted',
    format: 'maven2',
    status: { repositoryName: 'maven-snapshots', online: false },
    url: 'http://localhost:8081/repository/maven-snapshots/',
  },
  {
    name: 'nuget-group',
    type: 'group',
    format: 'nuget',
    status: { repositoryName: 'nuget-group', online: true },
    url: 'http://localhost:8081/repository/nuget-group/',
  },
  {
    name: 'nuget-hosted',
    type: 'hosted',
    format: 'nuget',
    status: { repositoryName: 'nuget-hosted', online: true },
    url: 'http://localhost:8081/repository/nuget-hosted/',
  },
  {
    name: 'nuget.org-proxy',
    type: 'proxy',
    format: 'nuget',
    status: { repositoryName: 'nuget.org-proxy', online: true, description: 'Ready to Connect' },
    url: 'http://localhost:8081/repository/nuget.org-proxy/',
  },
];

/**
 * Mock health check data for testing.
 */
export const mockHealthCheckData: HealthCheckStatus[] = [
  {
    repositoryName: 'maven-central',
    enabled: false,
    analyzing: false,
  },
  {
    repositoryName: 'nuget.org-proxy',
    enabled: false,
    analyzing: false,
  },
];

/**
 * Mock firewall status data for testing.
 */
export const mockFirewallStatusData: FirewallStatus[] = [
  {
    repositoryName: 'maven-central',
    affectedComponentCount: 0,
    criticalComponentCount: 10,
    severeComponentCount: 20,
    moderateComponentCount: 30,
    quarantinedComponentCount: 15,
    reportUrl: 'http://localhost:8070/ui/links/repository/b3b7a0c423014acb9ad1e414b221a111/result',
    message: null,
    errorMessage: null,
  },
  {
    repositoryName: 'nuget.org-proxy',
    affectedComponentCount: 0,
    criticalComponentCount: 11,
    severeComponentCount: 22,
    moderateComponentCount: 33,
    quarantinedComponentCount: 17,
    reportUrl: 'http://localhost:8070/ui/links/repository/b3b7a0c423014acb9ad1e414b221a222/result',
    message: null,
    errorMessage: null,
  },
];

/**
 * Mock firewall status data with messages.
 */
export const mockFirewallStatusDataWithMessages: FirewallStatus[] = [
  {
    repositoryName: 'maven-central',
    affectedComponentCount: 0,
    criticalComponentCount: 10,
    severeComponentCount: 20,
    moderateComponentCount: 30,
    quarantinedComponentCount: 15,
    reportUrl: 'http://localhost:8070/ui/links/repository/b3b7a0c423014acb9ad1e414b221a111/result',
    message: 'foo',
    errorMessage: null,
  },
  {
    repositoryName: 'nuget.org-proxy',
    affectedComponentCount: 0,
    criticalComponentCount: 11,
    severeComponentCount: 22,
    moderateComponentCount: 33,
    quarantinedComponentCount: 17,
    reportUrl: 'http://localhost:8070/ui/links/repository/b3b7a0c423014acb9ad1e414b221a222/result',
    message: null,
    errorMessage: 'error',
  },
];

/**
 * Row indices for testing (1-based for table rows, including header).
 */
export const ROW_INDICES = {
  MAVEN_CENTRAL: 1,
  MAVEN_PUBLIC: 2,
  MAVEN_RELEASES: 3,
  MAVEN_SNAPSHOTS: 4,
  NUGET_GROUP: 5,
  NUGET_HOSTED: 6,
  NUGET_ORG_PROXY: 7,
};

/**
 * Field names for testing.
 */
export const FIELDS = {
  NAME: 'name' as const,
  TYPE: 'type' as const,
  FORMAT: 'format' as const,
  STATUS: 'status' as const,
};

/**
 * Convert health check data to map.
 */
export function healthCheckDataToMap(
  data: HealthCheckStatus[]
): Record<string, HealthCheckStatus> {
  return data.reduce<Record<string, HealthCheckStatus>>((acc, item) => {
    acc[item.repositoryName] = item;
    return acc;
  }, {});
}

/**
 * Convert firewall status data to map.
 */
export function firewallStatusDataToMap(
  data: FirewallStatus[]
): Record<string, FirewallStatus> {
  return data.reduce<Record<string, FirewallStatus>>((acc, item) => {
    acc[item.repositoryName] = item;
    return acc;
  }, {});
}

