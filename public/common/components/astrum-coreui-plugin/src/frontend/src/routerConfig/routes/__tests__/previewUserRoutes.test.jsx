import { previewUserRoutes } from '../previewUserRoutes';
import { userRoutes } from '../userRoutes';
import { ROUTE_NAMES } from '../../routeNames/routeNames';

const previewRoute = (name) => previewUserRoutes.find((r) => r.name === name);
const classicRoute = (name) => userRoutes.find((r) => r.name === name);

describe('previewUserRoutes visibility', () => {
  it('offers the NuGet API key under the same conditions as the Classic page', () => {
    expect(previewRoute('preview.user.nugetapitoken').data.visibilityRequirements).toEqual(
      classicRoute(ROUTE_NAMES.USER.NUGETAPITOKEN).data.visibilityRequirements
    );
  });

  it('offers the user token under the same conditions as the Classic page', () => {
    expect(previewRoute('preview.user.usertoken').data.visibilityRequirements).toEqual(
      classicRoute(ROUTE_NAMES.USER.USER_TOKEN).data.visibilityRequirements
    );
  });
});
