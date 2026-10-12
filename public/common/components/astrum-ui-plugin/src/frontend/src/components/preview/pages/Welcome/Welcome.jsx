
import React, {useEffect, useRef, useState} from 'react';
import {useMachine} from '@xstate/react';
import {useRouter} from '@uirouter/react';
import {Box, Tabs} from '@radix-ui/themes';
import { ExtJS } from '../../../../interface/ExtJS';
import { toURIParams } from '../../../../interface/urlUtil';
import { getVersionMajorMinor } from '../../../../interface/versionUtil';
import { Permissions } from '../../../../constants/Permissions';
import UIStrings from '../../../../constants/UIStrings';
import welcomeMachine from '../../../pages/user/Welcome/WelcomeMachine';
import OutreachActions from './OutreachActions';
import { UsageDashboard } from './usage';
import { ErrorBoundary } from '../../shared';

import './Welcome.scss';

const iframeUrlPath = './service/outreach/';
const iframeDefaultHeight = 1000;
const iframePadding = 48;

function getUserType(user) {
  if (!user) {
    return 'anonymous';
  }
  else if (user.administrator) {
    return 'admin';
  }
  else {
    return 'normal';
  }
}

function getDatabaseType() {
  return ExtJS.state().getValue('datastore.isPostgresql') ? 'postgres' : 'h2';
}

function WelcomeDashboard({
  state, iframeProps, proxyDownloadNumberParams, iframeRef, onLoad, iframeHeight,
}) {
  const router = useRouter();
  const { params } = router.globals;
  const user = ExtJS.useUser();
  const isAuthenticated = !!user;
  const canSeeUsageMetrics = false;
  const canSearch = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.SEARCH.READ), []);
  const canConfigureCleanup = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.ADMIN), []) && isAuthenticated;
  const showAnnouncements = isAuthenticated && !!state.context.data?.showOutreachIframe;
  const initialTab = params?.tab || 'overview';
  const [activeTab, setActiveTab] = useState(initialTab);
  // isAdmin can resolve after first render, so re-derive on every render rather than
  // only in the useState initializer/redirect effect below.
  const effectiveTab = activeTab === 'usage-metrics' && !canSeeUsageMetrics ? 'overview' : activeTab;
  const outreachRef = useRef();

  useEffect(() => {
    if (params?.tab && params.tab !== activeTab) {
      const nextTab = params.tab === 'usage-metrics' && !canSeeUsageMetrics
        ? 'overview'
        : params.tab;
      setActiveTab(nextTab);
      if (nextTab !== params.tab) {
        router.stateService.go('preview.browse.welcome', {tab: nextTab}, {notify: false, location: 'replace'});
      }
    } else if (isAuthenticated && activeTab === 'usage-metrics' && !canSeeUsageMetrics) {
      // params.tab already equals activeTab (both 'usage-metrics') on a fresh deep-link,
      // so the branch above never fires. Correct the URL once the user object has
      // resolved (isAuthenticated) and we know for sure they can't see the tab.
      setActiveTab('overview');
      router.stateService.go('preview.browse.welcome', {tab: 'overview'}, {notify: false, location: 'replace'});
    }
  }, [params?.tab, activeTab, canSeeUsageMetrics, isAuthenticated, router]);

  const handleTabChange = (tab) => {
    setActiveTab(tab);
    router.stateService.go('preview.browse.welcome', { tab }, { notify: false, location: 'replace' });
  };

  const dashboard = (
    <ErrorBoundary>
      <UsageDashboard canSearch={canSearch} canConfigureCleanup={canConfigureCleanup} />
    </ErrorBoundary>
  );

  return (
    <Box px={{ initial: '4', md: '6' }} py="5">
      <OutreachActions ref={outreachRef} showCards={false} />

      {showAnnouncements ? (
        <Tabs.Root value={effectiveTab} onValueChange={handleTabChange}>
          <Tabs.List className="as-welcome-tabs">
            <Tabs.Trigger value="overview">{UIStrings.USAGE_DASHBOARD.TITLE}</Tabs.Trigger>
            <Tabs.Trigger value="announcements">{UIStrings.USAGE_DASHBOARD.ANNOUNCEMENTS}</Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content value="overview">
            <Box pt="4">{dashboard}</Box>
          </Tabs.Content>

          <Tabs.Content value="announcements">
            <Box pt="4" style={{ width: '100%', minHeight: '400px' }}>
              <iframe
                id="as-welcome-outreach-frame"
                role="document"
                height={iframeHeight}
                ref={iframeRef}
                scrolling="no"
                onLoad={onLoad}
                aria-label="Outreach Frame"
                src={`${iframeUrlPath}?${toURIParams(iframeProps)}${proxyDownloadNumberParams ?? ''}`}
                style={{
                  width: '100%',
                  height: iframeHeight,
                  border: 'none',
                  display: 'block',
                  borderRadius: 'var(--radius-3)',
                  pointerEvents: 'auto',
                }}
              />
            </Box>
          </Tabs.Content>
        </Tabs.Root>
      ) : dashboard}
    </Box>
  );
}

export default function Welcome() {
  const [state, send] = useMachine(welcomeMachine, {devtools: false});
  const [iframeHeight, setIframeHeight] = useState(iframeDefaultHeight);
  const ref = useRef();

  const user = ExtJS.useUser();
  const status = ExtJS.useStatus();
  const license = ExtJS.useLicense();

  const proxyDownloadNumberParams = state.context.data?.proxyDownloadNumberParams;

  const iframeProps = {
    version: status?.version || '',
    versionMm: getVersionMajorMinor(status?.version || ''),
    edition: status?.edition || '',
    usertype: getUserType(user),
    daysToExpiry: license?.daysToExpiry,
    databaseType: getDatabaseType()
  };

  function load() {
    send({type: 'LOAD'});
  }

  const onLoad = () => {
    try {
      if (ref.current?.contentWindow?.document?.body) {
        setIframeHeight(
          ref.current.contentWindow.document.body.scrollHeight + iframePadding * 4
        );
      }
    } catch {
      setIframeHeight(400);
    }
  };

  useEffect(load, [user?.id]);

  useEffect(() => {
    let timeout;
    const debounce = () => { timeout = setTimeout(onLoad, 500); };
    window.addEventListener('resize', debounce);
    return () => {
      if (timeout) clearTimeout(timeout);
      window.removeEventListener('resize', debounce);
    };
  }, []);

  return (
    <Box
      className="as-welcome-radix"
      p={undefined}
      data-testid="welcome-page"
      style={{ height: '100%' }}
    >
      <WelcomeDashboard
        state={state}
        iframeProps={iframeProps}
        proxyDownloadNumberParams={proxyDownloadNumberParams}
        iframeRef={ref}
        onLoad={onLoad}
        iframeHeight={iframeHeight}
      />
    </Box>
  );
}
