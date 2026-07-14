

import React from 'react';
import { Page } from '@globalart/astrum-ui-plugin';
import {
  NxDivider,
  NxFontAwesomeIcon,
  NxH1,
  NxH2,
  NxP,
  NxTextLink
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

        <NxH1>404</NxH1>
        <NxH2>RESOURCE NOT FOUND</NxH2>

        <NxDivider/>

        <NxP>
          This resource is not available. It may have been moved, deleted, or you don’t have access.
        </NxP>
      </section>

        <section className="as-missing-route-page-actions">

          <NxP className="as-missing-route-page-actions-message">
            You might want to:
          </NxP>

          <div>
            <NxTextLink
                data-analytics-id="as-404-page-dashboard-lnk"
                className="nx-btn"
                href={dashboardHref}
            >
              Return to Dashboard
            </NxTextLink>
            <NxTextLink
                data-analytics-id="as-404-page-support-lnk"
                className="nx-btn"
                href="https://links.globalart.dev/products/astrum/docs?utm_medium=product&utm_source=astrum_repo&utm_campaign=menu-docs"
                external
            >
              Visit Documentation
            </NxTextLink>
          </div>
        </section>
    </div>
  </Page>
}

function GhostAnimation() {
  return <div className="as-ghost-animation">
    <span className="fa-layers fa-fw">
      <NxFontAwesomeIcon className="as-404-box" icon={faBox} />
      <NxFontAwesomeIcon className="as-404-ghost" icon={faGhost} />
    </span>
  </div>
}
