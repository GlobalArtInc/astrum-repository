import type { Area } from '../../app/area';

export const userArea: Area = {
  id: 'user',
  routes: [
    { path: 'user/account', load: () => import('./AccountPage'), requiresUser: true },
    { path: 'user/access-tokens', load: () => import('./AccessTokensPage'), requiresUser: true },
  ],
};
