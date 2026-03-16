import { Flex, Spinner } from '@radix-ui/themes';
import { type ComponentType, lazy, Suspense } from 'react';
import { createBrowserRouter, Navigate, type RouteObject } from 'react-router';
import { Guard } from '../auth/Guard';
import { LoginPage } from '../auth/LoginPage';
import { NotFoundPage } from '../shell/NotFoundPage';
import { Shell } from '../shell/Shell';
import type { Area, AreaRoute } from './area';
import { RootLayout } from './RootLayout';
import { RouteError } from './RouteError';

export function PageLoading() {
  return (
    <Flex justify="center" py="9">
      <Spinner size="3" />
    </Flex>
  );
}

function pageOf(route: AreaRoute): ComponentType {
  if (route.page) {
    return route.page;
  }
  if (route.load) {
    return lazy(route.load);
  }
  throw new Error(`the route ${route.path} has neither page nor load`);
}

export function toRouteObject(route: AreaRoute): RouteObject {
  const Page = pageOf(route);
  return {
    path: route.path,
    element: (
      <Guard need={route.need} requiresUser={route.requiresUser}>
        <Suspense fallback={<PageLoading />}>
          <Page />
        </Suspense>
      </Guard>
    ),
  };
}

export const SECTION_REDIRECTS: Record<string, string> = {
  admin: '/settings',
  'admin/repository': '/admin/repository/repositories',
  'admin/security': '/admin/security/privileges',
  'admin/system': '/admin/system/tasks',
};

export function buildRoutes(areas: Area[]): RouteObject[] {
  const pages = areas.flatMap((area) => area.routes ?? []).map(toRouteObject);
  const redirects = Object.entries(SECTION_REDIRECTS).map(([path, to]) => ({
    path,
    element: <Navigate to={to} replace />,
  }));
  return [
    {
      element: <RootLayout />,
      errorElement: <RouteError />,
      children: [
        { path: 'login', element: <LoginPage /> },
        {
          element: <Shell />,
          children: [...pages, ...redirects, { path: '*', element: <NotFoundPage /> }],
        },
      ],
    },
  ];
}

export function createAppRouter(areas: Area[]) {
  return createBrowserRouter(buildRoutes(areas));
}
