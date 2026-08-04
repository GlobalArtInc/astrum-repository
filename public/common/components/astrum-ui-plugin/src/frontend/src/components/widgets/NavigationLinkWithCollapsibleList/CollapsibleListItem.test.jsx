

import React from 'react';
import { render, screen } from '@testing-library/react';
import CollapsibleListItem from './CollapsibleListItem';

describe('CollapsibleListItem', () => {
  let renderComponent;
  beforeAll(() => {
    const minimalProps = { text: 'base text', href: 'base href' };
    renderComponent = (props) => render(<CollapsibleListItem {...minimalProps} {...props}></CollapsibleListItem>);
  });

  it('renders the link', () => {
    renderComponent();
    const link = screen.getByRole('link', { name: 'base text' });
    expect(link).toBeVisible();
    expect(link).toHaveAttribute('href', 'base href');
    expect(link).not.toHaveClass('selected');
  });

  it('renders the link with selected styles', () => {
    renderComponent({ isSelected: true });
    const link = screen.getByRole('link', { name: 'base text' });
    expect(link).toHaveClass('selected');
  });
});
