

import React from 'react';
import { render, screen } from '@testing-library/react';
import { NavigationLinkWithCollapsibleList } from './NavigationLinkWithCollapsibleList';
import { faAirFreshener } from '@fortawesome/free-solid-svg-icons';
import userEvent from '@testing-library/user-event';
import { UIRouter, useCurrentStateAndParams } from '@uirouter/react';
import { getTestRouter } from '../../../router/testRouter';

jest.mock('@uirouter/react', () => ({
  ...jest.requireActual('@uirouter/react'),
  useCurrentStateAndParams: jest.fn()
}));

describe('NavigationLinkWithCollapsibleList', () => {
  let renderComponent;
  beforeAll(() => {
    const minimalProps = { text: 'base text', href: 'base href', icon: faAirFreshener, name: 'router.other.name' };
    renderComponent = (props, children = 'this are some children items') => {
      useCurrentStateAndParams.mockReturnValue({
        state: { name: 'router.name' }
      });
      const router = getTestRouter();
      return render(
        <UIRouter router={router}>
          <NavigationLinkWithCollapsibleList {...minimalProps} {...props}>
            {children}
          </NavigationLinkWithCollapsibleList>
        </UIRouter>
      );
    };
  });

  afterEach(() => {
    useCurrentStateAndParams.mockReset();
  });

  it('renders the link without the list', () => {
    renderComponent();
    const parentLink = screen.getByRole('link', { name: 'base text' });
    expect(parentLink).toBeVisible();
    expect(parentLink).toHaveAttribute('href', 'base href');
    const expandableList = screen.queryByRole('list');
    expect(expandableList).not.toBeInTheDocument();
  });

  it('opens the list on chevron click and collapses it on second click', () => {
    renderComponent();
    const dropdownButton = screen.getByRole('button', { name: /expand menu/i });
    let expandableList = screen.queryByRole('list');
    // Initial collapsed state
    expect(expandableList).not.toBeInTheDocument();
    // Click to expand
    userEvent.click(dropdownButton);
    expandableList = screen.getByRole('list');
    expect(expandableList).toBeVisible();
    // Click again to collapse
    userEvent.click(dropdownButton);
    expandableList = screen.queryByRole('list');
    expect(expandableList).not.toBeInTheDocument();
  });

  it('renders the link with the list expanded by default', () => {
    renderComponent({ name: 'router.name' });
    const expandableList = screen.getByRole('list');
    expect(expandableList).toBeVisible();
  });

  it('renders the link without selected styles', () => {
    renderComponent();
    const parentLink = screen.getByRole('link', { name: 'base text' });
    expect(parentLink).not.toHaveClass('selected');
  });

  it('renders the link with selected styles', () => {
    renderComponent({ isSelected: true });
    const parentLink = screen.getByRole('link', { name: 'base text' });
    expect(parentLink).toHaveClass('selected');
  });
});
