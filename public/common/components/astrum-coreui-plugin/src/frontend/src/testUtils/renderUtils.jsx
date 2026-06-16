

import React from 'react';
import { getRouter } from '../routerConfig/routerConfig';
import { act } from 'react';
import { render } from '@testing-library/react';
import { UIRouter, UIView } from '@uirouter/react';

export async function renderComponentRoute(routeName) {
  const router = getRouter();

  await router.stateService.go(routeName);


  return act(async () => {
    render(<UIRouter router={getRouter()}><UIView /></UIRouter>);
  });
}
