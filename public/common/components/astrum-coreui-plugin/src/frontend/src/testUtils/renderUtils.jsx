import React from 'react';
import { getRouter } from '../routerConfig/routerConfig';
import { act } from 'react';
import { render } from '@testing-library/react';
import { UIRouter, UIView } from '@uirouter/react';

/**
 * Renders the page of a route. The Classic pages still in the code pass getClassicRouter, since the application
 * no longer registers their routes.
 */
export async function renderComponentRoute(routeName, createRouter = getRouter) {
  const router = createRouter();

  await router.stateService.go(routeName);


  return act(async () => {
    render(<UIRouter router={createRouter()}><UIView /></UIRouter>);
  });
}
