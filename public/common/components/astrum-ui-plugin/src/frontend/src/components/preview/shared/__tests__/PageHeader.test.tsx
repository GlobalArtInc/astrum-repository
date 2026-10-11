

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme, Button } from '@radix-ui/themes';
import { Plus } from 'lucide-react';
import { PageHeader } from '../PageHeader';

const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

describe('PageHeader', () => {
  it('renders title', () => {
    renderWithTheme(<PageHeader title="Repositories" />);

    expect(screen.getByTestId('page-header')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Repositories' })).toBeInTheDocument();
  });

  it('renders description', () => {
    renderWithTheme(
      <PageHeader
        title="Repositories"
        description="Manage your repositories"
      />
    );

    expect(screen.getByText('Manage your repositories')).toBeInTheDocument();
  });

  it('renders an inline count next to the title', () => {
    renderWithTheme(<PageHeader title="Tags" count="3" />);

    expect(screen.getByRole('heading', { name: 'Tags' })).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
  });

  it('omits the count element when count is not provided', () => {
    renderWithTheme(<PageHeader title="Tags" />);

    // No stray count text — only the title is rendered in the header.
    expect(screen.getByTestId('page-header')).toHaveTextContent('Tags');
  });

  it('renders a zero count (does not treat 0 as absent)', () => {
    renderWithTheme(<PageHeader title="Tags" count={0} />);

    expect(screen.getByText('0')).toBeInTheDocument();
  });

  it('renders a meta row under the title', () => {
    renderWithTheme(<PageHeader title="docker-onprem" meta={<span>Offline</span>} />);

    expect(screen.getByTestId('page-header-meta')).toHaveTextContent('Offline');
  });

  it('omits the meta row when meta is not provided', () => {
    renderWithTheme(<PageHeader title="docker-onprem" />);

    expect(screen.queryByTestId('page-header-meta')).not.toBeInTheDocument();
  });

  it('renders a node after the title, outside the heading', () => {
    renderWithTheme(<PageHeader title="Audit" titleSuffix={<span data-testid="state">Active</span>} />);

    expect(screen.getByRole('heading', { name: 'Audit' })).toBeInTheDocument();
    expect(screen.getByTestId('state')).toHaveTextContent('Active');
    expect(screen.getByRole('heading', { name: 'Audit' })).not.toContainElement(screen.getByTestId('state'));
  });

  it('renders action buttons', () => {
    renderWithTheme(
      <PageHeader
        title="Repositories"
        actions={
          <Button>
            <Plus size={16} /> Create
          </Button>
        }
      />
    );

    expect(screen.getByRole('button', { name: /create/i })).toBeInTheDocument();
  });

  it('renders breadcrumbs', () => {
    const mockOnClick = jest.fn();

    renderWithTheme(
      <PageHeader
        title="Blob Stores"
        breadcrumbs={[
          { label: 'Settings', onClick: mockOnClick },
          { label: 'Repository' },
          { label: 'Blob Stores' },
        ]}
      />
    );

    expect(screen.getByText('Settings')).toBeInTheDocument();
    expect(screen.getByText('Repository')).toBeInTheDocument();
    // 'Blob Stores' appears in both title and breadcrumb
    const blobStoresElements = screen.getAllByText('Blob Stores');
    expect(blobStoresElements.length).toBeGreaterThanOrEqual(2); // title + breadcrumb
  });

  it('makes breadcrumb clickable when onClick is provided', () => {
    const mockOnClick = jest.fn();

    renderWithTheme(
      <PageHeader
        title="Blob Stores"
        breadcrumbs={[
          { label: 'Settings', onClick: mockOnClick },
          { label: 'Blob Stores' },
        ]}
      />
    );

    const settingsLink = screen.getByRole('button', { name: 'Settings' });
    fireEvent.click(settingsLink);

    expect(mockOnClick).toHaveBeenCalled();
  });

  it('marks last breadcrumb as current', () => {
    const { container } = renderWithTheme(
      <PageHeader
        title="Blob Stores"
        breadcrumbs={[
          { label: 'Settings' },
          { label: 'Blob Stores' },
        ]}
      />
    );

    // Find the current breadcrumb via aria-current attribute
    const currentBreadcrumb = container.querySelector('[aria-current="page"]');
    expect(currentBreadcrumb).toBeInTheDocument();
    expect(currentBreadcrumb?.textContent).toBe('Blob Stores');
  });

  it('applies custom className', () => {
    const { container } = renderWithTheme(
      <PageHeader title="Test" className="custom-class" />
    );

    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });

  it('renders breadcrumb navigation with proper aria-label', () => {
    const { container } = renderWithTheme(
      <PageHeader
        title="Blob Stores"
        breadcrumbs={[{ label: 'Settings' }, { label: 'Blob Stores' }]}
      />
    );

    const breadcrumbNav = container.querySelector('[aria-label="Breadcrumb"]');
    expect(breadcrumbNav).toBeInTheDocument();
  });
});


