

import {scrollToUsageCenter} from './LocationUtils';

describe('scrollToUsageCenter', () => {
  let getElementByIdSpy;

  beforeEach(() => {
    getElementByIdSpy = jest.spyOn(document, 'getElementById');
  });

  afterEach(() => {
    window.location.hash = '';
    getElementByIdSpy.mockRestore();
  });

  it('routes to usage-metrics tab from welcome overview', () => {
    window.location.hash = '#browse/welcome';
    scrollToUsageCenter();
    expect(decodeURIComponent(window.location.hash)).toBe('#browse/welcome?tab=usage-metrics');
  });

  it('routes to usage-metrics tab from another page', () => {
    window.location.hash = '#admin/system/tasks';
    scrollToUsageCenter();
    expect(decodeURIComponent(window.location.hash)).toBe('#browse/welcome?tab=usage-metrics');
  });

  it('does not scroll (Usage Center is a tab, not an in-page element)', () => {
    window.location.hash = '#browse/welcome';
    scrollToUsageCenter();
    expect(getElementByIdSpy).not.toHaveBeenCalled();
  });
});
