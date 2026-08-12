
import React, {useEffect, useRef, useState, useCallback} from 'react';
import {useMachine} from '@xstate/react';
import {useRouter} from '@uirouter/react';
import {Box, Flex, Tabs, Heading} from '@radix-ui/themes';
import { ExtJS } from '../../../../interface/ExtJS';
import { toURIParams } from '../../../../interface/urlUtil';
import { getVersionMajorMinor } from '../../../../interface/versionUtil';
import welcomeMachine from '../../../pages/user/Welcome/WelcomeMachine';
import OutreachActions from './OutreachActions';

import {
  RepositoriesByFormatPanel,
  QuickActionStatsPanel,
  useRepositoriesByFormat,
} from './dashboard';
import { LoadingState, ErrorState, ErrorBoundary } from '../../shared';

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
  loading, error, load, state, iframeProps, proxyDownloadNumberParams,
  reposByFormat, handleViewRepos, iframeRef, onLoad, iframeHeight,
}) {
  const router = useRouter();
  const { params } = router.globals;
  const user = ExtJS.useUser();
  const isAdmin = user?.administrator;
  const isAuthenticated = !!user;
  const canSeeUsageMetrics = false;
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

  if (loading) {
    return (
      <Box p="6">
        <LoadingState message="Loading dashboard..." />
      </Box>
    );
  }
  if (error) {
    return (
      <Box p="6">
        <ErrorState title="Dashboard Error" message={error} onRetry={load} />
      </Box>
    );
  }

  return (
    <Box p="5">
      <Heading as="h1" size="6" mb="4">Dashboard</Heading>

      <OutreachActions ref={outreachRef} showCards={false} />

      <Tabs.Root value={effectiveTab} onValueChange={handleTabChange}>
        <Tabs.List className="as-welcome-tabs">
          <Tabs.Trigger value="overview">Overview</Tabs.Trigger>
          {isAuthenticated && state.context.data?.showOutreachIframe && (
            <Tabs.Trigger value="announcements">Announcements</Tabs.Trigger>
          )}
        </Tabs.List>

        <Tabs.Content value="overview">
          <Flex direction="column" gap="6" pt="4">
            <ErrorBoundary>
              <QuickActionStatsPanel
                onConnectClick={() => outreachRef.current?.openConnectModal?.()}
              />
            </ErrorBoundary>
            <Flex direction="row" gap="6" wrap="wrap" align="start">
              {isAdmin && (
                <Box style={{ flex: '1 1 300px', minWidth: 0 }}>
                  <RepositoriesByFormatPanel
                    data={reposByFormat.data}
                    loading={reposByFormat.loading}
                    error={reposByFormat.error}
                    onRetry={reposByFormat.refetch}
                    onViewRepos={handleViewRepos}
                  />
                </Box>
              )}
            </Flex>
          </Flex>
        </Tabs.Content>


        {isAuthenticated && state.context.data?.showOutreachIframe && (
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
        )}
      </Tabs.Root>
    </Box>
  );
}

export default function Welcome() {
  const [state, send] = useMachine(welcomeMachine, {devtools: false});
  const [iframeHeight, setIframeHeight] = useState(iframeDefaultHeight);
  const ref = useRef();
  const router = useRouter();

  const user = ExtJS.useUser();
  const status = ExtJS.useStatus();
  const license = ExtJS.useLicense();

  const reposByFormat = useRepositoriesByFormat();

  const handleViewRepos = useCallback((formatCode) => {
    router.stateService.go('preview.browse.browse', { format: formatCode });
  }, [router]);

  const loading = state.matches('loading');
  const error = state.matches('error') ? state.context.error : null;
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
        loading={loading}
        error={error}
        load={load}
        state={state}
        iframeProps={iframeProps}
        proxyDownloadNumberParams={proxyDownloadNumberParams}
        reposByFormat={reposByFormat}
        handleViewRepos={handleViewRepos}
        iframeRef={ref}
        onLoad={onLoad}
        iframeHeight={iframeHeight}
      />
    </Box>
  );
}
