

import React from 'react';
import {render, screen, fireEvent} from '@testing-library/react';
import {Theme} from '@radix-ui/themes';
import '@testing-library/jest-dom';

const mockHealthChecksFailed = {value: false};
const mockUseIsVisible = jest.fn(() => true);
const mockUseUnreadStatusFailure = jest.fn();
const mockMarkAcknowledged = jest.fn();
const mockStateServiceGo = jest.fn();
const mockStateRegistryGet = jest.fn(() => ({
  data: {visibilityRequirements: {requiresPermission: 'astrum:metrics:read'}},
}));

jest.mock('../../../../../interface/ExtJS', () => ({
  ExtJS: {
    useState: (fn) => fn(),
    state: () => ({
      getValue: (key, defaultVal) =>
        key === 'health_checks_failed' ? mockHealthChecksFailed.value : defaultVal,
    }),
  },
}));

jest.mock('../../../../../interface/NavigationUtils', () => ({
  useIsVisible: (...args) => mockUseIsVisible(...args),
}));

jest.mock('../../hooks', () => ({
  useUnreadStatusFailure: (...args) => mockUseUnreadStatusFailure(...args),
}));

jest.mock('@uirouter/react', () => ({
  useRouter: jest.fn(() => ({
    stateService: {go: mockStateServiceGo},
    stateRegistry: {get: mockStateRegistryGet},
  })),
}));

import SystemStatusBell from '../SystemStatusBell';

function renderBell(props = {}) {
  return render(
    <Theme>
      <SystemStatusBell className="as-system-status-radix" {...props} />
    </Theme>,
  );
}

describe('SystemStatusBell', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockHealthChecksFailed.value = false;
    mockUseIsVisible.mockReturnValue(true);
    mockUseUnreadStatusFailure.mockReturnValue({
      showDot: false,
      markAcknowledged: mockMarkAcknowledged,
    });
    mockStateRegistryGet.mockReturnValue({
      data: {visibilityRequirements: {requiresPermission: 'astrum:metrics:read'}},
    });
  });

  it('navigates to the metric-health state when clicked', () => {
    renderBell();
    fireEvent.click(screen.getByLabelText('System Status'));

    expect(mockStateRegistryGet).toHaveBeenCalledWith('preview.admin.support.metrichealth');
    expect(mockUseIsVisible).toHaveBeenCalledWith(
      expect.objectContaining({requiresPermission: 'astrum:metrics:read'}),
    );
    expect(mockStateServiceGo).toHaveBeenCalledWith('preview.admin.support.metrichealth');
  });

  it('renders nothing when the target state is not registered', () => {
    mockStateRegistryGet.mockReturnValue(undefined);

    const {container} = renderBell();

    expect(container.querySelector('.as-system-status-radix')).toBeNull();
    expect(screen.queryByLabelText('System Status')).not.toBeInTheDocument();
  });

  it('renders nothing when the target state is not visible to the user', () => {
    mockUseIsVisible.mockReturnValue(false);

    const {container} = renderBell();

    expect(container.querySelector('.as-system-status-radix')).toBeNull();
    expect(screen.queryByLabelText('System Status')).not.toBeInTheDocument();
  });

  it('passes the current health-check state to useUnreadStatusFailure', () => {
    mockHealthChecksFailed.value = true;

    renderBell();

    expect(mockUseUnreadStatusFailure).toHaveBeenLastCalledWith(true);
  });

  it('does not render the red dot when the hook reports no unread failure', () => {
    mockUseUnreadStatusFailure.mockReturnValue({
      showDot: false,
      markAcknowledged: mockMarkAcknowledged,
    });

    const {container} = renderBell();

    expect(container.querySelector('.as-system-status-radix__badge')).toBeNull();
    expect(screen.getByLabelText('System Status')).toBeInTheDocument();
  });

  it('renders the red dot and unhealthy aria-label when the hook reports an unread failure', () => {
    mockUseUnreadStatusFailure.mockReturnValue({
      showDot: true,
      markAcknowledged: mockMarkAcknowledged,
    });

    const {container} = renderBell();

    expect(container.querySelector('.as-system-status-radix__badge')).not.toBeNull();
    expect(screen.getByLabelText('System status -- unhealthy')).toBeInTheDocument();
  });

  it('calls markAcknowledged before navigating when clicked while unhealthy', () => {
    mockUseUnreadStatusFailure.mockReturnValue({
      showDot: true,
      markAcknowledged: mockMarkAcknowledged,
    });

    renderBell();
    fireEvent.click(screen.getByLabelText('System status -- unhealthy'));

    expect(mockMarkAcknowledged).toHaveBeenCalledTimes(1);
    expect(mockStateServiceGo).toHaveBeenCalledWith('preview.admin.support.metrichealth');
    const ackOrder = mockMarkAcknowledged.mock.invocationCallOrder[0];
    const goOrder = mockStateServiceGo.mock.invocationCallOrder[0];
    expect(ackOrder).toBeLessThan(goOrder);
  });

  it('applies data-analytics-id when the prop is provided', () => {
    renderBell({dataAnalyticsId: 'as-global-header-system-status'});

    expect(screen.getByLabelText('System Status')).toHaveAttribute(
      'data-analytics-id',
      'as-global-header-system-status',
    );
  });

  it('omits data-analytics-id when the prop is not provided', () => {
    renderBell();

    expect(screen.getByLabelText('System Status')).not.toHaveAttribute('data-analytics-id');
  });

  it('takes the look and the labels of the button from its host', () => {
    mockUseUnreadStatusFailure.mockReturnValue({
      showDot: false,
      markAcknowledged: mockMarkAcknowledged,
    });

    renderBell({
      buttonClassName: 'header-button',
      variant: 'outline',
      label: 'Состояние системы',
      unhealthyLabel: 'Состояние системы: есть сбои',
    });

    const button = screen.getByRole('button', {name: 'Состояние системы'});
    expect(button).toHaveClass('header-button');
    expect(button).toHaveClass('rt-variant-outline');
  });

  it('uses the unhealthy label of its host while a failure is unread', () => {
    mockUseUnreadStatusFailure.mockReturnValue({
      showDot: true,
      markAcknowledged: mockMarkAcknowledged,
    });

    const {container} = renderBell({label: 'Status', unhealthyLabel: 'Status: failing'});

    expect(screen.getByRole('button', {name: 'Status: failing'})).toBeInTheDocument();
    expect(container.querySelector('.as-system-status-radix__badge')).not.toBeNull();
  });

  it('stays a ghost button by default', () => {
    renderBell();

    expect(screen.getByRole('button', {name: 'System Status'})).toHaveClass('rt-variant-ghost');
  });
});
