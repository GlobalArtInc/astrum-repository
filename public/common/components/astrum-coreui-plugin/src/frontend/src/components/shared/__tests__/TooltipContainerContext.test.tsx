

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { TooltipContainerProvider, usePortalContainer, Tooltip } from '../Tooltip/TooltipContainerContext';

// Test component to access and display the portal container
function PortalContainerConsumer() {
  const container = usePortalContainer();
  return (
    <div data-testid="consumer">
      {container ? 'has-container' : 'no-container'}
    </div>
  );
}

const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

describe('TooltipContainerProvider', () => {
  // Positioning, z-index, and overflow rules for the portal container live in
  // TooltipContainerContext.scss (ASTRUM-51836 moved them off the inline style).
  // The tests below assert the class wiring; the actual CSS values are owned by
  // the stylesheet and verified visually.
  it('renders a portal container with the as-tooltip-container class', () => {
    const { container } = renderWithTheme(
      <TooltipContainerProvider>
        <div data-testid="child">Child content</div>
      </TooltipContainerProvider>
    );

    const portalContainer = container.querySelector('[aria-hidden="true"]');
    expect(portalContainer).toBeInTheDocument();
    expect(portalContainer).toHaveClass('as-tooltip-container');
  });

  it('renders children correctly', () => {
    renderWithTheme(
      <TooltipContainerProvider>
        <div data-testid="child">Child content</div>
      </TooltipContainerProvider>
    );

    expect(screen.getByTestId('child')).toBeInTheDocument();
    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('provides portal container via usePortalContainer hook', async () => {
    renderWithTheme(
      <TooltipContainerProvider>
        <PortalContainerConsumer />
      </TooltipContainerProvider>
    );

    // Wait for useEffect to set the container
    const consumer = await screen.findByTestId('consumer');
    expect(consumer).toHaveTextContent('has-container');
  });
});

describe('Tooltip', () => {
  it('renders without throwing when used outside TooltipContainerProvider', () => {
    // Tooltip should gracefully fall back to body when no provider is present
    expect(() => {
      renderWithTheme(
        <Tooltip content="Tooltip text">
          <button>Hover me</button>
        </Tooltip>
      );
    }).not.toThrow();

    expect(screen.getByRole('button', { name: 'Hover me' })).toBeInTheDocument();
  });

  it('renders inside TooltipContainerProvider without throwing', () => {
    expect(() => {
      renderWithTheme(
        <TooltipContainerProvider>
          <Tooltip content="Tooltip text">
            <button>Hover me</button>
          </Tooltip>
        </TooltipContainerProvider>
      );
    }).not.toThrow();

    expect(screen.getByRole('button', { name: 'Hover me' })).toBeInTheDocument();
  });
});
