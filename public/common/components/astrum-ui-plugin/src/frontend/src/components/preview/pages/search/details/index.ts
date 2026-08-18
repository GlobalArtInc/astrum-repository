

/**
 * GA Detail Module - Agent 2 Workspace
 * 
 * Exports components for GA detail views with 5 tabs:
 * - Overview: description, license, project URL
 * - Versions: version list with status badges
 * - Repositories: repos containing this GA
 * - Files: assets for selected version
 * - Security: vulnerabilities for selected version
 */

// Main page component
export { GADetailPage } from './GADetailPage';

// Tab components
export { GAOverviewTab } from './GAOverviewTab';
export { GAVersionsTab } from './GAVersionsTab';
export { GARepositoriesTab } from './GARepositoriesTab';
export { GAFilesTab } from './GAFilesTab';
export { GASecurityTab } from './GASecurityTab';

// Shared components
export { Breadcrumbs } from './Breadcrumbs';
export type { BreadcrumbItem } from './Breadcrumbs';

// Hooks
export { useGADetail } from './useGADetail';

