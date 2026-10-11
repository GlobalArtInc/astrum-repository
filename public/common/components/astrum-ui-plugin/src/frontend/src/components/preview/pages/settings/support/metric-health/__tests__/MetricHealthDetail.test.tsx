import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { MetricHealthDetail } from '../MetricHealthDetail';

const renderWithTheme = (ui: React.ReactElement) => render(<Theme>{ui}</Theme>);

describe('MetricHealthDetail', () => {
  it('names a check with a readable name once', () => {
    renderWithTheme(
      <MetricHealthDetail check={{ name: 'Default Secret Encryption Key', result: { healthy: false, message: 'm' } }} />,
    );

    expect(screen.getAllByText('Default Secret Encryption Key')).toHaveLength(1);
  });

  it('shows the original name of a check under its readable name', () => {
    renderWithTheme(
      <MetricHealthDetail
        check={{ name: 'com.codahale.metrics.health.jvm.ThreadDeadlockHealthCheck', result: { healthy: true } }}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Thread Deadlock' })).toBeInTheDocument();
    expect(screen.getByText('com.codahale.metrics.health.jvm.ThreadDeadlockHealthCheck')).toBeInTheDocument();
  });
});
