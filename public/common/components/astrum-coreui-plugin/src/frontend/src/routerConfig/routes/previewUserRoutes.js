

import {UIView} from '@uirouter/react';
import {
  UserAccountPage,
  UserTokenPage,
  NuGetApiTokenPage,
  AccessTokensPage,
} from '@globalart/astrum-ui-plugin';

export const previewUserRoutes = [
  {
    name: 'preview.user',
    url: '/user',
    abstract: true,
    component: UIView,
  },
  {
    name: 'preview.user.account',
    url: '/account',
    component: UserAccountPage,
    data: {title: 'Account'},
  },
  {
    name: 'preview.user.accesstokens',
    url: '/accesstokens',
    component: AccessTokensPage,
    data: {title: 'Access Tokens', visibilityRequirements: {requiresUser: true}},
  },
  {
    name: 'preview.user.nugetapitoken',
    url: '/nugetapitoken',
    component: NuGetApiTokenPage,
    data: {title: 'NuGet API Token'},
  },
  {
    name: 'preview.user.usertoken',
    url: '/usertoken',
    component: UserTokenPage,
    data: {title: 'User Token'},
  },
];
