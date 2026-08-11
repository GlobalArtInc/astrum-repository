
import React from 'react';
import {renderHook} from '@testing-library/react';
import {useContextAwareRouteName} from '../useContextAwareRouteName';
import {PreviewUIContext} from '../PreviewUIContext';

function wrapper(isPreview: boolean) {
  return ({children}: {children: React.ReactNode}) =>
    React.createElement(PreviewUIContext.Provider, {value: isPreview}, children);
}

describe('useContextAwareRouteName', () => {
  it('returns the name unchanged when not in preview mode', () => {
    const {result} = renderHook(() => useContextAwareRouteName('browse.search'), {
      wrapper: wrapper(false),
    });
    expect(result.current).toBe('browse.search');
  });

  it('prefixes with preview. when in preview mode', () => {
    const {result} = renderHook(() => useContextAwareRouteName('browse.search'), {
      wrapper: wrapper(true),
    });
    expect(result.current).toBe('preview.browse.search');
  });

  it('returns undefined unchanged', () => {
    const {result} = renderHook(() => useContextAwareRouteName(undefined), {
      wrapper: wrapper(true),
    });
    expect(result.current).toBeUndefined();
  });

  it('keeps already-prefixed preview. name in preview mode', () => {
    const {result} = renderHook(() => useContextAwareRouteName('preview.browse.search'), {
      wrapper: wrapper(true),
    });
    expect(result.current).toBe('preview.browse.search');
  });

  it('strips preview. prefix when not in preview mode', () => {
    const {result} = renderHook(() => useContextAwareRouteName('preview.browse.search'), {
      wrapper: wrapper(false),
    });
    expect(result.current).toBe('browse.search');
  });
});
