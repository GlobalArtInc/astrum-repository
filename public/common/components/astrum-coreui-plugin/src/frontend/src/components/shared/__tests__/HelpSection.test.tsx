

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { HelpSection } from '../HelpSection';

const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

describe('HelpSection', () => {
  it('renders title and content', () => {
    renderWithTheme(
      <HelpSection
        title="What is a blob store?"
        content="A blob store provides physical storage for repository contents."
      />
    );

    expect(screen.getByTestId('help-section')).toBeInTheDocument();
    expect(screen.getByText('What is a blob store?')).toBeInTheDocument();
    expect(screen.getByText('A blob store provides physical storage for repository contents.')).toBeInTheDocument();
  });

  it('renders multi-paragraph content', () => {
    renderWithTheme(
      <HelpSection
        title="Help"
        content={`First paragraph.
Second paragraph.
Third paragraph.`}
      />
    );

    // Each paragraph should be rendered separately
    expect(screen.getByText('First paragraph.')).toBeInTheDocument();
    expect(screen.getByText('Second paragraph.')).toBeInTheDocument();
    expect(screen.getByText('Third paragraph.')).toBeInTheDocument();
  });

  it('renders documentation link', () => {
    renderWithTheme(
      <HelpSection
        title="Blob Stores"
        content="Learn about blob stores."
        docLink={{
          label: 'View Documentation',
          href: 'https://help.globalart.dev/blob-stores',
        }}
      />
    );

    const link = screen.getByRole('link', { name: /view documentation/i });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', 'https://help.globalart.dev/blob-stores');
    expect(link).toHaveAttribute('target', '_blank');
    expect(link).toHaveAttribute('rel', 'noopener noreferrer');
  });

  it('hides icon when showIcon is false', () => {
    const { container } = renderWithTheme(
      <HelpSection
        title="Help"
        content="Content"
        showIcon={false}
      />
    );

    // Check that there's no SVG icon in the title
    const title = container.querySelector('.help-section__title');
    expect(title?.querySelector('svg')).not.toBeInTheDocument();
  });

  it('shows icon by default', () => {
    const { container } = renderWithTheme(
      <HelpSection
        title="Help"
        content="Content"
      />
    );

    const title = container.querySelector('.help-section__title');
    expect(title?.querySelector('svg')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = renderWithTheme(
      <HelpSection
        title="Help"
        content="Content"
        className="custom-class"
      />
    );

    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });

  it('filters empty paragraphs', () => {
    renderWithTheme(
      <HelpSection
        title="Help"
        content={`First paragraph.


Second paragraph.`}
      />
    );

    // Should only render non-empty paragraphs (empty lines filtered out)
    const paragraphs = screen.getAllByText(/paragraph/);
    expect(paragraphs).toHaveLength(2);
  });
});


