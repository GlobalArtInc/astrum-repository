

import {isGlobalArtDevMode} from './devModeUtils';

describe('isGlobalArtDevMode', () => {
  afterEach(() => {
    localStorage.clear();
  });

  it('is false when neither the build flag nor the localStorage flag is set', () => {
    expect(isGlobalArtDevMode()).toBe(false);
  });

  it('is true when the localStorage flag is set to "true"', () => {
    localStorage.setItem('GLOBALART_INTERNAL', 'true');
    expect(isGlobalArtDevMode()).toBe(true);
  });

  it('is false when the localStorage flag is set to a non-"true" value', () => {
    localStorage.setItem('GLOBALART_INTERNAL', 'false');
    expect(isGlobalArtDevMode()).toBe(false);
  });

  it('is false when localStorage access throws', () => {
    const getItemSpy = jest.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('localStorage is unavailable');
    });

    expect(isGlobalArtDevMode()).toBe(false);

    getItemSpy.mockRestore();
  });
});
