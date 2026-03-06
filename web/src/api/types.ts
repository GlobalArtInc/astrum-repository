export type RepositoryType = 'hosted' | 'proxy' | 'group';

export interface UiUser {
  userId: string;
  name: string;
  email: string;
  source: string;
}

export interface UiFormat {
  id: string;
  apiName: string;
  types: RepositoryType[];
}

export interface UiState {
  version: string;
  revision: string | null;
  user: UiUser | null;
  anonymousAccess: boolean;
  permissions: string[];
  formats: UiFormat[];
  baseUrl: string;
  sessionTimeoutMinutes: number;
  adminPasswordFile: string | null;
  oidcEnabled?: boolean;
  defaultRole?: { id: string; name: string } | null;
  nugetApiKeyRealm?: boolean;
  uiSettings?: { title: string; debugAllowed: boolean };
  branding?: { headerHtml: string | null; footerHtml: string | null };
}
