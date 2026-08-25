

import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { GASearchPagination } from '../GASearchPagination';

describe('GASearchPagination', () => {
  it('returns null when no more results and all loaded', () => {
    const { container } = render(
      <GASearchPagination
        hasMore={false}
        loading={false}
        loadedCount={10}
        totalCount={10}
        onLoadMore={jest.fn()}
      />
    );

    expect(container.firstChild).toBeNull();
  });

  it('renders when loadedCount < totalCount (even without hasMore)', () => {
    render(
      <GASearchPagination
        hasMore={false}
        loading={false}
        loadedCount={5}
        totalCount={10}
        onLoadMore={jest.fn()}
      />
    );

    expect(screen.getByText(/showing 5 of 10 results/i)).toBeInTheDocument();
  });

  it('shows Load More button when hasMore is true', () => {
    render(
      <GASearchPagination
        hasMore={true}
        loading={false}
        loadedCount={10}
        totalCount={50}
        onLoadMore={jest.fn()}
      />
    );

    expect(screen.getByRole('button', { name: /load more/i })).toBeInTheDocument();
  });

  it('disables button when loading', () => {
    render(
      <GASearchPagination
        hasMore={true}
        loading={true}
        loadedCount={10}
        totalCount={50}
        onLoadMore={jest.fn()}
      />
    );

    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('calls onLoadMore when button clicked', async () => {
    const onLoadMore = jest.fn();
    render(
      <GASearchPagination
        hasMore={true}
        loading={false}
        loadedCount={10}
        totalCount={50}
        onLoadMore={onLoadMore}
      />
    );

    await userEvent.click(screen.getByRole('button', { name: /load more/i }));

    expect(onLoadMore).toHaveBeenCalledTimes(1);
  });
});
