

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';

import { SearchResultItem, searchResultName } from '../SearchResultItem';

// Helper to wrap components with Radix Theme
function renderWithTheme(ui) {
  return render(<Theme>{ui}</Theme>);
}

describe('SearchResultItem', () => {
  const mockOnClick = jest.fn();

  const mavenResult = {
    id: '1',
    repository: 'maven-central',
    format: 'maven2',
    group: 'org.apache.commons',
    name: 'commons-lang3',
    version: '3.12.0',
  };

  const npmResult = {
    id: '2',
    repository: 'npm-proxy',
    format: 'npm',
    name: 'lodash',
    version: '4.17.21',
  };

  const noVersionResult = {
    id: '3',
    repository: 'raw-hosted',
    format: 'raw',
    name: 'my-file.txt',
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('rendering', () => {
    it('renders Maven result with group:name format', () => {
      renderWithTheme(
        <SearchResultItem result={mavenResult} onClick={mockOnClick} />
      );

      expect(screen.getByText('org.apache.commons:commons-lang3')).toBeInTheDocument();
      expect(screen.getByText('3.12.0')).toBeInTheDocument();
    });

    it('renders npm result with just name', () => {
      renderWithTheme(
        <SearchResultItem result={npmResult} onClick={mockOnClick} />
      );

      expect(screen.getByText('lodash')).toBeInTheDocument();
      expect(screen.getByText('4.17.21')).toBeInTheDocument();
    });

    it('renders result without version', () => {
      renderWithTheme(
        <SearchResultItem result={noVersionResult} onClick={mockOnClick} />
      );

      expect(screen.getByText('my-file.txt')).toBeInTheDocument();
      expect(screen.queryByText('undefined')).not.toBeInTheDocument();
    });

    it('renders with correct role', () => {
      renderWithTheme(
        <SearchResultItem result={mavenResult} onClick={mockOnClick} />
      );

      expect(screen.getByRole('option')).toBeInTheDocument();
    });
  });

  describe('selection state', () => {
    it('has selected class when isSelected is true', () => {
      renderWithTheme(
        <SearchResultItem result={mavenResult} onClick={mockOnClick} isSelected />
      );

      const item = screen.getByRole('option');
      expect(item).toHaveClass('in-repo-search__result-item--selected');
      expect(item).toHaveAttribute('aria-selected', 'true');
    });

    it('does not have selected class when isSelected is false', () => {
      renderWithTheme(
        <SearchResultItem result={mavenResult} onClick={mockOnClick} isSelected={false} />
      );

      const item = screen.getByRole('option');
      expect(item).not.toHaveClass('in-repo-search__result-item--selected');
      expect(item).toHaveAttribute('aria-selected', 'false');
    });
  });

  describe('click behavior', () => {
    it('calls onClick when clicked', () => {
      renderWithTheme(
        <SearchResultItem result={mavenResult} onClick={mockOnClick} />
      );

      fireEvent.click(screen.getByRole('option'));

      expect(mockOnClick).toHaveBeenCalledTimes(1);
    });
  });
});

describe('searchResultName', () => {
  it('writes the name of a result as its format does', () => {
    expect(searchResultName({ format: 'maven2', group: 'org.example', name: 'demo-api' })).toBe('org.example:demo-api');
    expect(searchResultName({ format: 'npm', group: 'acme', name: 'ui-kit' })).toBe('@acme/ui-kit');
    expect(searchResultName({ format: 'npm', name: 'left-pad' })).toBe('left-pad');
  });

  it('does not repeat the directory of a raw file, which its name already holds', () => {
    expect(searchResultName({ format: 'raw', group: '/tools/linux', name: '/tools/linux/install.txt' }))
      .toBe('/tools/linux/install.txt');
  });
});
