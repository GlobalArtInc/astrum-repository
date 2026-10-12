import { Theme } from '@radix-ui/themes';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { type RenderResult, render } from '@testing-library/react';
import type { ReactNode } from 'react';
import { createMemoryRouter, type RouteObject, RouterProvider } from 'react-router';
import type { UiState } from '../api';
import type { Area } from '../app/area';
import { RegistryProvider } from '../app/registry';
import { STATE_KEY } from '../auth/state';
import { I18nProvider, type Language } from '../i18n';
import { ThemeProvider } from '../shell/theme';
import { ConfirmProvider } from '../ui/ConfirmDialog';
import { Toaster } from '../ui/Toaster';

export const ADMIN_STATE: UiState = {
  version: '3.0.0-test',
  revision: null,
  user: { userId: 'admin', name: 'Administrator User', email: 'admin@example.org', source: 'default' },
  anonymousAccess: true,
  permissions: ['astrum:*'],
  formats: [{ id: 'raw', apiName: 'raw', types: ['hosted', 'proxy', 'group'] }],
  baseUrl: 'http://localhost',
  sessionTimeoutMinutes: 0,
  adminPasswordFile: null,
};

export const ANONYMOUS_STATE: UiState = {
  ...ADMIN_STATE,
  user: null,
  permissions: ['astrum:search:read', 'astrum:repository-view:*:*:browse,read'],
};

export interface Options {
  state?: UiState;
  path?: string;
  routes?: RouteObject[];
  areas?: Area[];
  language?: Language;
  client?: QueryClient;
}

export function testClient(state: UiState = ADMIN_STATE): QueryClient {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: Number.POSITIVE_INFINITY },
      mutations: { retry: false },
    },
  });
  client.setQueryData(STATE_KEY, state);
  return client;
}

export function Providers({
  children,
  client,
  areas = [],
  language = 'en',
}: {
  children: ReactNode;
  client: QueryClient;
  areas?: Area[];
  language?: Language;
}) {
  return (
    <QueryClientProvider client={client}>
      <I18nProvider language={language}>
        <ThemeProvider>
          <RegistryProvider areas={areas}>
            <Theme>
              <Toaster>
                <ConfirmProvider>{children}</ConfirmProvider>
              </Toaster>
            </Theme>
          </RegistryProvider>
        </ThemeProvider>
      </I18nProvider>
    </QueryClientProvider>
  );
}

export function renderWithApp(
  element: ReactNode,
  options: Options = {},
): RenderResult & { client: QueryClient; router: ReturnType<typeof createMemoryRouter> } {
  const client = options.client ?? testClient(options.state);
  const routes = options.routes ?? [{ path: '*', element }];
  const router = createMemoryRouter(routes, { initialEntries: [options.path ?? '/'] });
  const result = render(
    <Providers client={client} areas={options.areas} language={options.language}>
      <RouterProvider router={router} />
    </Providers>,
  );
  return { ...result, client, router };
}

export { fakeServer, jsonResponse } from './http';
