
import React, { Suspense } from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { lazyLoad } from './lazyLoad';

describe('lazyLoad', () => {
  const MockComponentLoadedContent = "Mock Component Loaded";
  // Mock component for testing
  const MockComponent = () => <div>{MockComponentLoadedContent}</div>;

  beforeEach(() => {
    jest.clearAllMocks();
    // Clear console.error mock if it was set
    if (console.error.mockClear) {
      console.error.mockClear();
    }
  });

  describe('successful module loading', () => {
    it('should load a component with default export', async () => {
      // Arrange: Create a mock import function that returns a module with default export
      const mockImportFn = jest.fn(() =>
        Promise.resolve({ default: MockComponent })
      );

      // Act: Create lazy component and render it
      const LazyComponent = lazyLoad(mockImportFn);

      const LoadingContent = "Loading...";

      render(
        <Suspense fallback={<div>{LoadingContent}</div>}>
          <LazyComponent />
        </Suspense>
      );

      // Assert: Should show loading state first
      expect(screen.getByText(LoadingContent)).toBeInTheDocument();

      // Assert: Should eventually load the component
      await waitFor(() => {
        expect(screen.getByText(MockComponentLoadedContent)).toBeInTheDocument();
      });

      // Assert: Import function should be called once
      expect(mockImportFn).toHaveBeenCalledTimes(1);
    });

    it('should return a lazy component that can be rendered multiple times', async () => {
      // Arrange
      const mockImportFn = jest.fn(() =>
        Promise.resolve({ default: MockComponent })
      );
      const LazyComponent = lazyLoad(mockImportFn);

      const LoadingContent = "Loading...";

      // Act: Render the same lazy component twice
      const { unmount } = render(
        <Suspense fallback={<div>{LoadingContent}</div>}>
          <LazyComponent />
        </Suspense>
      );

      await waitFor(() => {
        expect(screen.getByText(MockComponentLoadedContent)).toBeInTheDocument();
      });

      unmount();

      // Render again
      render(
        <Suspense fallback={<div>{LoadingContent}</div>}>
          <LazyComponent />
        </Suspense>
      );

      // Assert: Should load again
      await waitFor(() => {
        expect(screen.getByText(MockComponentLoadedContent)).toBeInTheDocument();
      });

      // Note: React may cache the loaded module, so importFn might only be called once
      expect(mockImportFn).toHaveBeenCalled();
    });
  });

  describe('error handling', () => {
    it('should throw error when module has no default export', async () => {
      // Arrange: Mock console.error to suppress error output in tests
      const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

      const mockImportFn = jest.fn(() =>
        Promise.resolve({ namedExport: MockComponent })
      );

      const LazyComponent = lazyLoad(mockImportFn);

      const LoadingContent = "Loading...";

      // Act & Assert: Expect error to be thrown
      render(
        <Suspense fallback={<div>{LoadingContent}</div>}>
          <LazyComponent />
        </Suspense>
      );

      // Wait for the error to be logged
      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalled();
      });

      // Verify console.error was called with our error message
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        'Failed to load lazy-loaded chunk:',
        expect.any(Error)
      );

      consoleErrorSpy.mockRestore();
    });

    it('should handle network errors gracefully', async () => {
      // Arrange: Mock console.error
      const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

      const networkError = new Error('Network error');
      const mockImportFn = jest.fn(() => Promise.reject(networkError));

      const LazyComponent = lazyLoad(mockImportFn);

      const LoadingContent = "Loading...";

      // Act
      render(
        <Suspense fallback={<div>{LoadingContent}</div>}>
          <LazyComponent />
        </Suspense>
      );

      // Assert: Should log the error
      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalledWith(
          'Failed to load lazy-loaded chunk:',
          networkError
        );
      });

      consoleErrorSpy.mockRestore();
    });

    it('should handle chunk loading errors', async () => {
      // Arrange: Mock console.error
      const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

      const chunkError = new Error('ChunkLoadError: Loading chunk failed');
      const mockImportFn = jest.fn(() => Promise.reject(chunkError));

      const LazyComponent = lazyLoad(mockImportFn);

      const LoadingContent = "Loading...";

      // Act
      render(
        <Suspense fallback={<div>{LoadingContent}</div>}>
          <LazyComponent />
        </Suspense>
      );

      // Assert: Should log the error
      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalledWith(
          'Failed to load lazy-loaded chunk:',
          chunkError
        );
      });

      consoleErrorSpy.mockRestore();
    });
  });

  describe('props passed to the loaded component', () => {

    it('should accept props and pass them to the loaded component', async () => {
      // Arrange
      const MockComponentWithProps = ({ name }) => <div>Hello {name}</div>;

      const mockImportFn = jest.fn(() =>
        Promise.resolve({ default: MockComponentWithProps })
      );

      const LazyComponent = lazyLoad(mockImportFn);

      const LoadingContent = "Loading...";

        // Act
      render(
        <Suspense fallback={<div>{LoadingContent}</div>}>
          <LazyComponent name="Test User" />
        </Suspense>
      );

      // Assert
      await waitFor(() => {
        expect(screen.getByText('Hello Test User')).toBeInTheDocument();
      });
    });
  });

  describe('lazy loading behavior', () => {
    it('should not call import function until component is rendered', () => {
      // Arrange
      const mockImportFn = jest.fn(() =>
        Promise.resolve({ default: MockComponent })
      );

      // Act: Create lazy component but don't render it
      lazyLoad(mockImportFn);

      // Assert: Import function should not be called yet
      expect(mockImportFn).not.toHaveBeenCalled();
    });
  });
});

describe('Route Configuration Integration', () => {
  // Import route files
  const adminRoutes = require('./adminRoutes').adminRoutes;
  const browseRoutes = require('./browseRoutes').browseRoutes;
  const userRoutes = require('./userRoutes').userRoutes;

  describe('route name validation', () => {
    it('should not have duplicate route names', () => {
      // Combine all routes from all route files
      const allRoutes = [...adminRoutes, ...browseRoutes, ...userRoutes];

      // Extract just the route names
      const routeNames = allRoutes.map(r => r.name);

      // Create a Set to remove duplicates
      const uniqueNames = new Set(routeNames);

      // If lengths match, there are no duplicates
      expect(routeNames.length).toBe(uniqueNames.size);
    });
  });
});
