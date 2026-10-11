import { act, renderHook } from '@testing-library/react';
import { COMPACT_MEDIA_QUERY, PHONE_MEDIA_QUERY, useIsPhone, useMediaQuery } from './useMediaQuery';

type Listener = () => void;

function mockMatchMedia(initialWidth: number) {
  let width = initialWidth;
  const lists: { query: string; listeners: Set<Listener> }[] = [];
  const evaluate = (query: string) => {
    const max = Number(/max-width:\s*(\d+)px/.exec(query)?.[1]);
    return width <= max;
  };

  window.matchMedia = jest.fn((query: string) => {
    const entry = { query, listeners: new Set<Listener>() };
    lists.push(entry);
    return {
      get matches() {
        return evaluate(query);
      },
      media: query,
      addEventListener: (_: string, listener: Listener) => entry.listeners.add(listener),
      removeEventListener: (_: string, listener: Listener) => entry.listeners.delete(listener),
    } as unknown as MediaQueryList;
  });

  return {
    resize(newWidth: number) {
      width = newWidth;
      lists.forEach((entry) => entry.listeners.forEach((listener) => listener()));
    },
  };
}

describe('useMediaQuery', () => {
  const originalMatchMedia = window.matchMedia;

  afterEach(() => {
    window.matchMedia = originalMatchMedia;
  });

  it('tells whether the viewport matches the query', () => {
    mockMatchMedia(390);

    expect(renderHook(() => useMediaQuery(PHONE_MEDIA_QUERY)).result.current).toBe(true);
    expect(renderHook(() => useMediaQuery(COMPACT_MEDIA_QUERY)).result.current).toBe(true);
  });

  it('follows the viewport when it is resized', () => {
    const viewport = mockMatchMedia(390);
    const { result } = renderHook(() => useIsPhone());

    act(() => viewport.resize(768));
    expect(result.current).toBe(false);

    act(() => viewport.resize(600));
    expect(result.current).toBe(true);
  });

  it('treats a desktop as neither phone nor compact', () => {
    mockMatchMedia(1440);

    expect(renderHook(() => useMediaQuery(PHONE_MEDIA_QUERY)).result.current).toBe(false);
    expect(renderHook(() => useMediaQuery(COMPACT_MEDIA_QUERY)).result.current).toBe(false);
  });

  it('is false where media queries cannot be evaluated', () => {
    // @ts-expect-error jsdom has no matchMedia; the hook must not need one
    delete window.matchMedia;

    expect(renderHook(() => useMediaQuery(PHONE_MEDIA_QUERY)).result.current).toBe(false);
  });
});
