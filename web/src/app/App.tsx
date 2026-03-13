import { type QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from 'react-router/dom';
import { useUiStateQuery } from '../auth/state';
import { I18nProvider } from '../i18n';
import { ThemeProvider } from '../shell/theme';
import type { Area } from './area';
import { RegistryProvider } from './registry';
import type { createAppRouter } from './router';
import { Starting, StartupFailed } from './Startup';

type Router = ReturnType<typeof createAppRouter>;

function Bootstrap({ router }: { router: Router }) {
  const state = useUiStateQuery();
  if (!state.data) {
    return state.isError ? (
      <StartupFailed error={state.error} onRetry={() => void state.refetch()} />
    ) : (
      <Starting />
    );
  }
  return <RouterProvider router={router} />;
}

export function App({ client, router, areas }: { client: QueryClient; router: Router; areas: Area[] }) {
  return (
    <QueryClientProvider client={client}>
      <I18nProvider>
        <ThemeProvider>
          <RegistryProvider areas={areas}>
            <Bootstrap router={router} />
          </RegistryProvider>
        </ThemeProvider>
      </I18nProvider>
    </QueryClientProvider>
  );
}
