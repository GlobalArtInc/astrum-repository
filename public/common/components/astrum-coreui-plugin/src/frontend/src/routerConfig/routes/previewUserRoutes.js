

import {UIView} from '@uirouter/react';
import {
  UserAccountPage,
  UserTokenPage,
  NuGetApiTokenPage,
  AccessTokensPage,
} from '@globalart/astrum-ui-plugin';
import PreviewRouteTitles from './previewRouteTitles';

export const previewUserRoutes = [
  {
    name: 'preview.user',
    url: 'user',
    abstract: true,
    component: UIView,
  },
  {
    name: 'preview.user.account',
    url: '/account',
    component: UserAccountPage,
    data: {title: PreviewRouteTitles.ACCOUNT},
  },
  {
    name: 'preview.user.accesstokens',
    url: '/accesstokens',
    component: AccessTokensPage,
    data: {title: PreviewRouteTitles.ACCESS_TOKENS, visibilityRequirements: {requiresUser: true}},
  },
  // Both mirror the retired Classic pages: the pages work only where their realm is active.
  {
    name: 'preview.user.nugetapitoken',
    url: '/nugetapitoken',
    component: NuGetApiTokenPage,
    data: {
      title: PreviewRouteTitles.NUGET_API_TOKEN,
      visibilityRequirements: {
        requiresUser: true,
        permissions: ['astrum:apikey:*'],
        statesEnabled: [{key: 'nugetApiKeyRealmEnabled', defaultValue: false}],
      },
    },
  },
  {
    name: 'preview.user.usertoken',
    url: '/usertoken',
    component: UserTokenPage,
    data: {
      title: PreviewRouteTitles.USER_TOKEN,
      visibilityRequirements: {
        bundle: 'astrum-usertoken-plugin',
        statesEnabled: [{key: 'userTokenRealmEnabled', defaultValue: false}],
        permissions: ['astrum:usertoken-current:read'],
        editions: ['PRO'],
      },
    },
  },
];
