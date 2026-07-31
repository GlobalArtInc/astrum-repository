

import {UIView} from "@uirouter/react";
import {createRouter} from "./createRouter";
import React from "react";

export function getTestRouter() {
  const initialRoute = 'browse.welcome';
  const menuRoutes = [
    {
      name: 'login',
      url: '/login',
      component: () => <div>login page</div>,
      data: { visibilityRequirements: {} }
    },
    {
      name: 'browse',
      url: 'browse',
      component: UIView,
      data: {
        visibilityRequirements: {
          ignoreForMenuVisibilityCheck: true
        },
        title: 'Browse'
      }
    },
    {
      name: 'browse.welcome',
      url: '/welcome',
      component: () => <div>Welcome</div>,
      data: {
        visibilityRequirements: {},
        title: 'Dashboard'
      }
    },
    {
      name: 'browse.search',
      url: '/search',
      component: UIView,
      abstract: true,
      data: {
        // make sure we don't inherit from BROWSE
        visibilityRequirements: {},
        title: 'Search'
      }
    },
    {
      name: 'browse.search.generic',
      url: '/generic:keyword',
      component: () => <div>Generic Search</div>,
      params: {
        keyword: {
          value: null,
          raw: true,
          dynamic: true
        }
      },
      data: {
        visibilityRequirements: {
          permissions: ['astrum:search:read']
        },
        title: 'Search'
      }
    }
  ];
  const missingRoute = {
    name: 'missing.route',
    url: 404,
    component: () => <div>Missing Route</div>,
    data: {
      visibilityRequirements: {}
    }
  };

  return createRouter({ initialRoute, menuRoutes, missingRoute });
}
