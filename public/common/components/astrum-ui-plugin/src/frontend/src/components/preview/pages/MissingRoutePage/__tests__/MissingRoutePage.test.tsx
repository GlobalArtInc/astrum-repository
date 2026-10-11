import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';

import { MissingRoutePage } from '../MissingRoutePage';

const mockUseSref = jest.fn((name: string) => ({ href: `#${name}` }));

jest.mock('@uirouter/react', () => ({
  useSref: (name: string) => mockUseSref(name),
}));

describe('MissingRoutePage', () => {
  afterEach(() => {
    window.location.hash = '';
    mockUseSref.mockClear();
  });

  it('leads back to the dashboard of Astrum One', () => {
    window.location.hash = '#browse/nothing-here';

    render(<MissingRoutePage />);

    expect(screen.getByText('404')).toBeInTheDocument();
    expect(mockUseSref).toHaveBeenCalledWith('preview.browse.welcome');
    expect(screen.getByRole('link', { name: /dashboard/i })).toHaveAttribute('href', '#preview.browse.welcome');
  });
});
