

import {scrollToUsageCenter} from './LocationUtils';

describe('scrollToUsageCenter', () => {
  let getElementByIdSpy;

  beforeEach(() => {
    jest.useRealTimers();
    getElementByIdSpy = jest.spyOn(document, 'getElementById');
  });

  afterEach(() => {
    window.location.hash = '';
    getElementByIdSpy.mockRestore();
    jest.useRealTimers();
  });

  describe('Preview UI', () => {
    it('routes to usage-metrics tab from welcome overview', () => {
      window.location.hash = '#preview/browse/welcome';
      scrollToUsageCenter();
      expect(decodeURIComponent(window.location.hash)).toBe('#preview/browse/welcome?tab=usage-metrics');
    });

    it('routes to usage-metrics tab from another preview page', () => {
      window.location.hash = '#preview/browse/browse';
      scrollToUsageCenter();
      expect(decodeURIComponent(window.location.hash)).toBe('#preview/browse/welcome?tab=usage-metrics');
    });

    it('does not scroll in Preview UI (Usage Center is a tab, not an in-page element)', () => {
      window.location.hash = '#preview/browse/welcome';
      scrollToUsageCenter();
      expect(getElementByIdSpy).not.toHaveBeenCalled();
    });
  });

  describe('Classic UI', () => {
    beforeEach(() => {
      jest.useFakeTimers();
    });

    afterEach(() => {
      jest.useRealTimers();
    });

    it('navigates to browse/welcome and scrolls to usage center element', () => {
      window.location.hash = '#admin/system/licensing';
      const mockElement = {scrollIntoView: jest.fn()};
      getElementByIdSpy.mockReturnValue(mockElement);

      scrollToUsageCenter();

      expect(window.location.hash).toBe('#browse/welcome');
      expect(window.location.hash).not.toContain('tab=usage-metrics');

      jest.advanceTimersByTime(200);

      expect(mockElement.scrollIntoView).toHaveBeenCalledWith({behavior: 'smooth'});
    });

    it('scrolls immediately when already on browse/welcome', () => {
      window.location.hash = '#browse/welcome';
      const mockElement = {scrollIntoView: jest.fn()};
      getElementByIdSpy.mockReturnValue(mockElement);

      scrollToUsageCenter();

      expect(window.location.hash).toBe('#browse/welcome');
      expect(mockElement.scrollIntoView).toHaveBeenCalledWith({behavior: 'smooth'});
      expect(window.location.hash).not.toContain('tab=usage-metrics');
    });
  });
});
