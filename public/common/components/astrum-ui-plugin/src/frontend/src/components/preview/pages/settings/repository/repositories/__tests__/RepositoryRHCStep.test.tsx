

import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { RepositoryRHCStep } from '../RepositoryRHCStep';

jest.mock('../useRepositoriesApi', () => ({
  useRepositoriesApi: () => ({
    enableHealthCheck: jest.fn(),
  }),
}));

describe('RepositoryRHCStep', () => {
  it('renders with switch in deferred mode', () => {
    const onChoice = jest.fn();
    render(<RepositoryRHCStep mode="deferred" value="none" onChoice={onChoice} />);

    expect(screen.getByRole('switch')).toBeInTheDocument();
    expect(screen.getByText('Repository Health Check')).toBeInTheDocument();
  });

  it('calls onChoice with "enable" when switch is toggled on', async () => {
    const onChoice = jest.fn();

    render(<RepositoryRHCStep mode="deferred" value="none" onChoice={onChoice} />);

    const switchElement = screen.getByRole('switch');
    expect(switchElement).not.toBeChecked();

    await userEvent.click(switchElement);

    expect(onChoice).toHaveBeenCalledWith('enable');
  });

  it('calls onChoice with "none" when switch is toggled off', async () => {
    const onChoice = jest.fn();

    render(<RepositoryRHCStep mode="deferred" value="enable" onChoice={onChoice} />);

    const switchElement = screen.getByRole('switch');
    expect(switchElement).toBeChecked();

    await userEvent.click(switchElement);

    expect(onChoice).toHaveBeenCalledWith('none');
  });

  it('displays checked state correctly based on value prop', () => {
    const { rerender } = render(<RepositoryRHCStep mode="deferred" value="none" onChoice={jest.fn()} />);

    expect(screen.getByRole('switch')).not.toBeChecked();

    rerender(<RepositoryRHCStep mode="deferred" value="enable" onChoice={jest.fn()} />);

    expect(screen.getByRole('switch')).toBeChecked();
  });

  describe('when capabilityEnabled is false', () => {
    it('disables the switch', () => {
      render(<RepositoryRHCStep mode="deferred" value="none" onChoice={jest.fn()} capabilityEnabled={false} />);

      expect(screen.getByRole('switch')).toBeDisabled();
    });

    it('shows the capability-disabled callout', () => {
      render(<RepositoryRHCStep mode="deferred" value="none" onChoice={jest.fn()} capabilityEnabled={false} />);

      expect(screen.getByTestId('rhc-capability-disabled-callout')).toBeInTheDocument();
      expect(screen.getByText(/system capability is not enabled/i)).toBeInTheDocument();
    });

    it('shows a link to system capabilities', () => {
      render(<RepositoryRHCStep mode="deferred" value="none" onChoice={jest.fn()} capabilityEnabled={false} />);

      const link = screen.getByRole('link', { name: /go to system capabilities/i });
      expect(link).toHaveAttribute('href', '#preview/admin/system/capabilities');
    });

    it('does not call onChoice when switch is clicked', async () => {
      const onChoice = jest.fn();
      render(<RepositoryRHCStep mode="deferred" value="none" onChoice={onChoice} capabilityEnabled={false} />);

      await userEvent.click(screen.getByRole('switch'));

      expect(onChoice).not.toHaveBeenCalled();
    });
  });

  describe('when capabilityEnabled is true (default)', () => {
    it('switch is enabled', () => {
      render(<RepositoryRHCStep mode="deferred" value="none" onChoice={jest.fn()} capabilityEnabled={true} />);

      expect(screen.getByRole('switch')).not.toBeDisabled();
    });

    it('does not show the capability-disabled callout', () => {
      render(<RepositoryRHCStep mode="deferred" value="none" onChoice={jest.fn()} capabilityEnabled={true} />);

      expect(screen.queryByTestId('rhc-capability-disabled-callout')).not.toBeInTheDocument();
    });
  });
});
