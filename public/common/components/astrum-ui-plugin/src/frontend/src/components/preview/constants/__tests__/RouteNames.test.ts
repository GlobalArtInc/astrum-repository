
import { ROUTE_NAMES } from '../RouteNames';
import { RouteNames as BaseRouteNames } from '../../../../constants/RouteNames';

describe('preview ROUTE_NAMES', () => {
  it('spreads the base RouteNames', () => {
    expect(ROUTE_NAMES.LOGIN).toBe(BaseRouteNames.LOGIN);
    expect(ROUTE_NAMES.MISSING_ROUTE).toBe(BaseRouteNames.MISSING_ROUTE);
    expect(ROUTE_NAMES.WELCOME).toBe(BaseRouteNames.WELCOME);
  });

  it('defines BROWSE.WELCOME with the Astrum One welcome state name', () => {
    expect(ROUTE_NAMES.BROWSE.WELCOME.ROOT).toBe('preview.browse.welcome');
    expect(BaseRouteNames.WELCOME).toBe('preview.browse.welcome');
    expect(ROUTE_NAMES.BROWSE.WELCOME.TITLE).toBe('Dashboard');
  });
});
