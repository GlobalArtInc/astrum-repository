
import React from 'react';
import {render, screen} from '@testing-library/react';
import {Theme} from '@radix-ui/themes';
import '@testing-library/jest-dom';
import Welcome from '../Welcome';
import {isFeatureEnabled} from '../../../config/featureFlags';

jest.mock('../../../config/featureFlags', () => ({
  isFeatureEnabled: jest.fn(),
}));

const mockStateService = {go: jest.fn()};
let mockParams = {};
jest.mock('@uirouter/react', () => ({
  useRouter: () => ({
    globals: {params: mockParams},
    stateService: mockStateService,
  }),
}));

jest.mock('@xstate/react', () => ({
  useMachine: () => [
    {matches: () => false, context: {data: {}}},
    jest.fn(),
  ],
}));

jest.mock('../../../../../interface/ExtJS', () => ({
  ExtJS: {
    useUser: jest.fn().mockReturnValue({userId: 'admin', administrator: true}),
    useStatus: jest.fn().mockReturnValue({version: '3.90.0', edition: 'PRO'}),
    useLicense: jest.fn().mockReturnValue({daysToExpiry: 365}),
    // Mirror the real reactive hook: evaluate the getter immediately.
    usePermission: (fn) => fn(),
    checkPermission: jest.fn(() => true),
    state: jest.fn().mockReturnValue({
      getValue: jest.fn().mockReturnValue(false),
      getUser: jest.fn().mockReturnValue({id: 'admin'}),
    }),
  },
}));

jest.mock('../../../../../interface/urlUtil', () => ({toURIParams: () => ''}));
jest.mock('../../../../../interface/versionUtil', () => ({getVersionMajorMinor: () => '3.0'}));
jest.mock('../../../../pages/user/Welcome/WelcomeMachine', () => ({__esModule: true, default: {}}));

jest.mock('../../../shared', () => ({
  LoadingState: ({message}) => <div>{message}</div>,
  ErrorState: ({message}) => <div>{message}</div>,
  ErrorBoundary: ({children}) => <>{children}</>,
}));

jest.mock('../dashboard', () => ({
  RepositoriesByFormatPanel: () => <div data-testid="repos-by-format" />,
  QuickActionStatsPanel: () => <div data-testid="quick-action-stats" />,
  useRepositoriesByFormat: () => ({data: [], loading: false, error: null, refetch: jest.fn()}),
}));

jest.mock('../OutreachActions', () => {
  const {forwardRef} = require('react');
  return {
    __esModule: true,
    default: forwardRef(function MockOutreachActions(_props, _ref) {
      return <div data-testid="outreach-actions" />;
    }),
  };
});

jest.mock('../MalwareStatusCard', () => ({
  __esModule: true,
  default: () => <div data-testid="malware-card" />,
}));

jest.mock('../HealthCheckStatusCard', () => ({
  __esModule: true,
  default: () => <div data-testid="hc-card" />,
}));

jest.mock('../../../shared/security/MalwareBanner', () => ({
  __esModule: true,
  default: () => <div data-testid="malware-banner" />,
}));

const {ExtJS} = require('../../../../../interface/ExtJS');

describe('Welcome', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockParams = {};
    ExtJS.checkPermission.mockReturnValue(true);
  });

  it('hides the Usage Metrics tab for a non-admin even when the flag is enabled', () => {
    isFeatureEnabled.mockReturnValue(true);
    ExtJS.useUser.mockReturnValue({userId: 'user', administrator: false});

    render(<Welcome />);

    expect(screen.queryByRole('tab', {name: 'Usage Metrics'})).not.toBeInTheDocument();
  });

  it('redirects to Overview once a resolving user settles as a non-admin', () => {
    isFeatureEnabled.mockReturnValue(true);
    ExtJS.useUser.mockReturnValue(undefined);
    mockParams = {tab: 'usage-metrics'};

    const {rerender} = render(<Welcome />);

    // Still resolving (isAuthenticated false): no redirect yet either way
    expect(mockStateService.go).not.toHaveBeenCalledWith(
      'preview.browse.welcome',
      {tab: 'overview'},
      expect.anything()
    );

    // User resolves to a non-admin
    ExtJS.useUser.mockReturnValue({userId: 'user', administrator: false});
    rerender(<Welcome />);

    expect(mockStateService.go).toHaveBeenCalledWith(
      'preview.browse.welcome',
      {tab: 'overview'},
      {notify: false, location: 'replace'}
    );
    expect(screen.getByRole('tab', {name: /Overview/, selected: true})).toBeInTheDocument();
  });

  it('hides the Usage Metrics tab entirely when the flag is disabled', () => {
    isFeatureEnabled.mockReturnValue(false);
    ExtJS.useUser.mockReturnValue({userId: 'admin', administrator: true});

    render(<Welcome />);

    expect(screen.queryByRole('tab', {name: 'Usage Metrics'})).not.toBeInTheDocument();
  });
});

// ASTRUM-54212: The dashboard "Repository Security" section renders a Health Check card
// backed by GET /service/rest/internal/ui/healthcheck/summary, which requires
// astrum:healthcheck:read. Without it the card surfaced a raw "Request failed with status
// code 403". The section must be hidden entirely for users lacking that permission, so it
// matches Classic (which never shows repository-security status to unpermitted users).
