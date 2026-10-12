import { previewUserRoutes } from '../previewUserRoutes';

const previewRoute = (name) => previewUserRoutes.find((r) => r.name === name);

describe('previewUserRoutes visibility', () => {
  it('offers the NuGet API key only where its realm is active', () => {
    expect(previewRoute('preview.user.nugetapitoken').data.visibilityRequirements).toEqual({
      requiresUser: true,
      permissions: ['astrum:apikey:*'],
      statesEnabled: [{key: 'nugetApiKeyRealmEnabled', defaultValue: false}],
    });
  });

  it('offers the user token only where its realm is active', () => {
    expect(previewRoute('preview.user.usertoken').data.visibilityRequirements).toEqual({
      bundle: 'astrum-usertoken-plugin',
      statesEnabled: [{key: 'userTokenRealmEnabled', defaultValue: false}],
      permissions: ['astrum:usertoken-current:read'],
      editions: ['PRO'],
    });
  });
});
