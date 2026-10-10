

import React from 'react';
import {render, screen} from '@testing-library/react';
import {DirectoryPage} from './DirectoryPage';
import {useCurrentStateAndParams} from '@uirouter/react';

// Mock dependencies
jest.mock('@uirouter/react', () => ({
  UIView: function MockUIView() {
    return <div data-testid="ui-view">UIView</div>;
  },
  useCurrentStateAndParams: jest.fn(),
  useRouter: jest.fn(() => ({
    stateService: {
      go: jest.fn()
    },
    stateRegistry: {
      get: jest.fn(() => [])
    }
  }))
}));

jest.mock('../../layout', () => ({
  Page: ({children, ...props}) => <div data-testid="page" {...props}>{children}</div>
}));

jest.mock('@globalart/react-shared-components', () => {
  const Tile = ({children}) => <div data-testid="tile">{children}</div>;
  const TileContent = ({children}) => <div data-testid="tile-content">{children}</div>;
  TileContent.displayName = 'TileContent';
  Tile.Content = TileContent;

  return {
    AsH1: ({children}) => <h1>{children}</h1>,
    AsP: ({children}) => <p>{children}</p>,
    AsTile: Tile
  };
});

describe('DirectoryPage', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders page content when route matches', () => {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: 'test.route'}
    });

    render(
      <DirectoryPage
        routeName="test.route"
        text="Test Page"
        description="Test description"
      >
        <div>Child content</div>
      </DirectoryPage>
    );

    expect(screen.getByText('Test Page')).toBeInTheDocument();
    expect(screen.getByText('Test description')).toBeInTheDocument();
    expect(screen.getByText('Child content')).toBeInTheDocument();
    expect(screen.getByTestId('page')).toBeInTheDocument();
    expect(screen.getByTestId('tile')).toBeInTheDocument();
  });

  it('renders UIView when route does not match', () => {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: 'other.route'}
    });

    render(
      <DirectoryPage
        routeName="test.route"
        text="Test Page"
        description="Test description"
      >
        <div>Child content</div>
      </DirectoryPage>
    );

    expect(screen.getByTestId('ui-view')).toBeInTheDocument();
    expect(screen.queryByText('Test Page')).not.toBeInTheDocument();
    expect(screen.queryByText('Child content')).not.toBeInTheDocument();
  });

  it('passes additional attributes to Page component', () => {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: 'test.route'}
    });

    render(
      <DirectoryPage
        routeName="test.route"
        text="Test Page"
        description="Test description"
        className="custom-class"
        data-test="custom-attr"
      >
        <div>Child content</div>
      </DirectoryPage>
    );

    const page = screen.getByTestId('page');
    expect(page).toHaveAttribute('class', 'custom-class');
    expect(page).toHaveAttribute('data-test', 'custom-attr');
  });

  it('renders without children', () => {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: 'test.route'}
    });

    render(
      <DirectoryPage
        routeName="test.route"
        text="Test Page"
        description="Test description"
      />
    );

    expect(screen.getByText('Test Page')).toBeInTheDocument();
    expect(screen.getByText('Test description')).toBeInTheDocument();
    expect(screen.getByTestId('tile')).toBeInTheDocument();
  });

  it('renders with multiple children', () => {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: 'test.route'}
    });

    render(
      <DirectoryPage
        routeName="test.route"
        text="Test Page"
        description="Test description"
      >
        <div>First child</div>
        <div>Second child</div>
      </DirectoryPage>
    );

    expect(screen.getByText('First child')).toBeInTheDocument();
    expect(screen.getByText('Second child')).toBeInTheDocument();
  });
});
