
import React, { useRef } from 'react';
import { render } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useExtJsUnsavedChangesGuard } from './extJsUnsavedChanges';

// Mock ExtJS and related globals
const warnBeforeNavigateMock = jest.fn();
const hasDirtMock = jest.fn();
const resetUnsavedChangesFlagMock = jest.fn();
const getFeatureContentMock = jest.fn(() => ({
  down: jest.fn(() => ({ getForm: jest.fn(() => ({ reset: jest.fn() })) })),
  getForm: jest.fn(() => ({ reset: jest.fn() })),
  resetUnsavedChangesFlag: resetUnsavedChangesFlagMock,
}));

// Mock NX.Bookmarks
const navigateToMock = jest.fn();
const fromTokenMock = jest.fn();

global.NX = {
  Bookmarks: {
    fromToken: fromTokenMock,
    navigateTo: navigateToMock,
  },
};

global.Ext = {
  getApplication: () => ({
    getController: () => ({
      hasDirt: hasDirtMock,
      warnBeforeNavigate: warnBeforeNavigateMock,
      getFeatureContent: getFeatureContentMock,
    }),
  }),
};

describe('useExtJsUnsavedChangesGuard', () => {
  function TestComponent() {
    const ref = useRef({ contains: () => false });
    useExtJsUnsavedChangesGuard(ref);
    return <div ref={ref}>Test</div>;
  }

  beforeEach(() => {
    jest.clearAllMocks();
    // Spy on console methods
    jest.spyOn(console, 'warn').mockImplementation();
    jest.spyOn(console, 'error').mockImplementation();

    hasDirtMock.mockReturnValue(false);
    warnBeforeNavigateMock.mockImplementation(cb => {
      cb && cb();
      return false;
    });

    // Default successful bookmark navigation
    fromTokenMock.mockReturnValue({ token: 'mock-bookmark' });
    navigateToMock.mockImplementation(() => {});
  });

  afterEach(() => {
    // Clean up any changes to window.location
    window.history.pushState({}, '', '/');
    // Restore console methods
    jest.restoreAllMocks();
  });

  it('should not prevent navigation if there is no unsaved dirt', () => {
    render(<TestComponent />);
    const a = document.createElement('a');
    a.setAttribute('href', '#test');
    document.body.appendChild(a);
    userEvent.click(a);
    expect(warnBeforeNavigateMock).not.toHaveBeenCalled();
    document.body.removeChild(a);
  });

  it('should call warnBeforeNavigate if there is unsaved dirt on link click', () => {
    hasDirtMock.mockReturnValue(true);
    render(<TestComponent />);
    const a = document.createElement('a');
    a.setAttribute('href', '#test');
    document.body.appendChild(a);
    userEvent.click(a);
    expect(warnBeforeNavigateMock).toHaveBeenCalled();
    document.body.removeChild(a);
  });

  it('should handle popstate event with unsaved dirt', () => {
    hasDirtMock.mockReturnValue(true);
    render(<TestComponent />);
    const event = new PopStateEvent('popstate');
    window.dispatchEvent(event);
    expect(warnBeforeNavigateMock).toHaveBeenCalled();
  });

  it('should not navigate if it is a single hash click', () => {
    hasDirtMock.mockReturnValue(true);
    render(<TestComponent />);
    expect(window.location.hash).toBe('');
    const a = document.createElement('a');
    a.setAttribute('href', '#');
    document.body.appendChild(a);
    userEvent.click(a);
    expect(window.location.hash).toBe('');
    expect(warnBeforeNavigateMock).toHaveBeenCalled();
    document.body.removeChild(a);
  });

  it('should handle null ref during fast navigation without throwing', () => {
    function TestComponentWithNullRef() {
      const ref = useRef(null); // Simulate unmounted component
      useExtJsUnsavedChangesGuard(ref);
      return <div>Test</div>;
    }

    render(<TestComponentWithNullRef />);
    const a = document.createElement('a');
    a.setAttribute('href', '#test/path');
    document.body.appendChild(a);

    // Should not throw when ref.current is null
    expect(() => userEvent.click(a)).not.toThrow();

    // Should not attempt to navigate or show warning since ref is null
    expect(navigateToMock).not.toHaveBeenCalled();
    expect(warnBeforeNavigateMock).not.toHaveBeenCalled();

    document.body.removeChild(a);
  });

  describe('goToHash function', () => {
    it('should use NX.Bookmarks.navigateTo when bookmark is successfully created', () => {
      const mockBookmark = { token: 'browse/welcome' };
      fromTokenMock.mockReturnValue(mockBookmark);

      render(<TestComponent />);
      const a = document.createElement('a');
      a.setAttribute('href', '#browse/welcome');
      document.body.appendChild(a);
      userEvent.click(a);

      expect(fromTokenMock).toHaveBeenCalledWith('#browse/welcome');
      expect(navigateToMock).toHaveBeenCalledWith(mockBookmark);
      expect(console.warn).not.toHaveBeenCalled();
      expect(console.error).not.toHaveBeenCalled();

      document.body.removeChild(a);
    });

    it('should fallback to window.location.hash when fromToken returns null', () => {
      fromTokenMock.mockReturnValue(null);

      render(<TestComponent />);
      const a = document.createElement('a');
      a.setAttribute('href', '#test/path');
      document.body.appendChild(a);

      userEvent.click(a);

      expect(fromTokenMock).toHaveBeenCalledWith('#test/path');
      expect(navigateToMock).not.toHaveBeenCalled();
      expect(console.warn).toHaveBeenCalledWith(
        '[goToHash] Could not create bookmark from hash:',
        '#test/path'
      );

      document.body.removeChild(a);
    });

    it('should fallback to window.location.hash when an error is thrown', () => {
      const testError = new Error('Bookmark creation failed');
      fromTokenMock.mockImplementation(() => {
        throw testError;
      });

      render(<TestComponent />);
      const a = document.createElement('a');
      a.setAttribute('href', '#error/path');
      document.body.appendChild(a);

      userEvent.click(a);

      expect(fromTokenMock).toHaveBeenCalledWith('#error/path');
      expect(navigateToMock).not.toHaveBeenCalled();
      expect(console.error).toHaveBeenCalledWith(
        '[goToHash] Error with bookmark navigation:',
        testError
      );

      document.body.removeChild(a);
    });
  });
});
