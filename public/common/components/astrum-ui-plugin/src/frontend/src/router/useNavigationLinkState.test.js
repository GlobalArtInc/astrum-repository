
import React from 'react';
import { renderHook } from '@testing-library/react';
import { UIRouter } from '@uirouter/react';
import { getTestRouter } from './testRouter';
import { useNavigationLinkState } from './useNavigationLinkState';

describe('useNavigationLinkState', () => {
  let router;

  beforeEach(() => {
    router = getTestRouter();
  });

  it('returns isVisible: false when route does not exist', () => {
    const { result } = renderHook(() => useNavigationLinkState('nonExistentRoute'), {
      wrapper: ({ children }) => (
        <UIRouter router={router}>
          {children}
        </UIRouter>
      )
    });

    expect(result.current.isVisible).toBe(false);
  });

  it('returns correct values when route exists', () => {
    const { result } = renderHook(() => useNavigationLinkState('preview.browse.welcome'), {
      wrapper: ({ children }) => (
        <UIRouter router={router}>
          {children}
        </UIRouter>
      )
    });

    expect(result.current.isVisible).toBe(true);
    expect(result.current.href).toBeTruthy();
  });
});
