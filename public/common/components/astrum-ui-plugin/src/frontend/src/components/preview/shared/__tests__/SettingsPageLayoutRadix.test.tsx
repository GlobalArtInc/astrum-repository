

import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import SettingsPageLayoutRadix from '../SettingsPageLayoutRadix';

jest.mock('@uirouter/react', () => ({
  UIView: () => <div data-testid="ui-view">Router Content</div>,
}));

describe('SettingsPageLayoutRadix', () => {
  it('renders back link to Settings hub', () => {
    render(<SettingsPageLayoutRadix />);
    const back = screen.getByRole('link', { name: /back to settings/i });
    expect(back).toHaveAttribute('href', '#preview/settings');
  });

  it('renders router outlet for nested settings routes', () => {
    render(<SettingsPageLayoutRadix />);
    expect(screen.getByTestId('ui-view')).toBeInTheDocument();
  });

  it('renders the expected scroll container classes for settings pages', () => {
    const { container } = render(<SettingsPageLayoutRadix />);

    expect(container.querySelector('.settings-layout-radix')).toBeInTheDocument();
    expect(container.querySelector('.settings-layout-radix__content')).toBeInTheDocument();
  });
});
