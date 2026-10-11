import {UIView} from "@uirouter/react";
import {createRouter} from "./createRouter";
import React from "react";

/**
 * A router with a few states shaped like the Astrum One ones: the 'preview' root owns the root of the hash, so
 * 'preview.browse.welcome' opens at #browse/welcome.
 */
export function getTestRouter() {
  const initialRoute = 'preview.browse.welcome';
  const menuRoutes = [
    {
      name: 'login',
      url: '/login',
      component: () => <div>login page</div>,
      data: { visibilityRequirements: {} }
    },
    {
      name: 'preview',
      url: '',
      abstract: true,
      component: UIView,
    },
    {
      name: 'preview.browse',
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
      name: 'preview.browse.welcome',
      url: '/welcome',
      component: () => <div>Welcome</div>,
      data: {
        visibilityRequirements: {},
        title: 'Dashboard'
      }
    },
    {
      name: 'preview.browse.search',
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
      name: 'preview.browse.search.generic',
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
