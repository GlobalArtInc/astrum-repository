
import Permissions from '../Permissions';

describe('Permissions constants (ASTRUM-54212)', () => {
  it('exposes tags create and delete', () => {
    expect(Permissions.TAGS.READ).toBe('astrum:tags:read');
    expect(Permissions.TAGS.CREATE).toBe('astrum:tags:create');
    expect(Permissions.TAGS.DELETE).toBe('astrum:tags:delete');
  });

  it('exposes healthcheck read and update', () => {
    expect(Permissions.HEALTHCHECK.READ).toBe('astrum:healthcheck:read');
    expect(Permissions.HEALTHCHECK.UPDATE).toBe('astrum:healthcheck:update');
  });

  it('exposes iq-violation-summary read', () => {
    expect(Permissions.IQ_VIOLATION_SUMMARY.READ).toBe('astrum:iq-violation-summary:read');
  });

  it('exposes repository-admin edit and delete wildcards', () => {
    expect(Permissions.REPOSITORY_ADMIN.READ).toBe('astrum:repository-admin:*:*:read');
    expect(Permissions.REPOSITORY_ADMIN.EDIT).toBe('astrum:repository-admin:*:*:edit');
    expect(Permissions.REPOSITORY_ADMIN.DELETE).toBe('astrum:repository-admin:*:*:delete');
  });
});
