
import { loginRoutes } from './loginRoutes';

describe('loginRoutes (CoreUI)', () => {
  it('should export login route with correct configuration', () => {
    expect(loginRoutes).toHaveLength(1);
    expect(loginRoutes[0].name).toBe('login');
    expect(loginRoutes[0].url).toBe('/login?returnTo');
    expect(loginRoutes[0].data.visibilityRequirements).toEqual({});
  });
});
