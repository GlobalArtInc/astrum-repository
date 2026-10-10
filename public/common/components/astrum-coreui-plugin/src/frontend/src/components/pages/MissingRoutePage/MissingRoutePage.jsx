

import React from 'react';
import { Page } from '@globalart/astrum-ui-plugin';
import {
  AsDivider,
  AsFontAwesomeIcon,
  AsH1,
  AsH2,
  AsP,
  AsTextLink
} from '@globalart/react-shared-components';
import { faBox, faGhost } from '@fortawesome/free-solid-svg-icons';

import { useSref } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../routerConfig/routeNames/routeNames';

export function MissingRoutePage()  {
  const isPreviewUI = window.location.hash.startsWith('#preview');
  const { href: classicDashboardHref } = useSref(ROUTE_NAMES.BROWSE.WELCOME.ROOT);
  const dashboardHref = isPreviewUI ? '#preview/browse/welcome' : classicDashboardHref;

  return <Page>
    <div className="as-missing-route-page-content">
      <section className="as-missing-route-page-content-message">
        <GhostAnimation/>

        <AsH1>404</AsH1>
        <AsH2>RESOURCE NOT FOUND</AsH2>

        <AsDivider/>

        <AsP>
          This resource is not available. It may have been moved, deleted, or you don’t have access.
        </AsP>
      </section>

        <section className="as-missing-route-page-actions">

          <AsP className="as-missing-route-page-actions-message">
            You might want to:
          </AsP>

          <div>
            <AsTextLink
                data-analytics-id="as-404-page-dashboard-lnk"
                className="as-btn"
                href={dashboardHref}
            >
              Return to Dashboard
            </AsTextLink>
            <AsTextLink
                data-analytics-id="as-404-page-support-lnk"
                className="as-btn"
                href="https://links.globalart.dev/products/astrum/docs?utm_medium=product&utm_source=astrum_repo&utm_campaign=menu-docs"
                external
            >
              Visit Documentation
            </AsTextLink>
          </div>
        </section>
    </div>
  </Page>
}

function GhostAnimation() {
  return <div className="as-ghost-animation">
    <span className="fa-layers fa-fw">
      <AsFontAwesomeIcon className="as-404-box" icon={faBox} />
      <AsFontAwesomeIcon className="as-404-ghost" icon={faGhost} />
    </span>
  </div>
}
