

import AnonymousAccessStep from '../AnonymousAccessStep';
import ChangePasswordStep from '../ChangePasswordStep';
import { stepRegistry } from '../stepRegistry';

describe('stepRegistry', () => {
  it('resolves ChangeAdminPassword to the ChangePasswordStep component', () => {
    expect(stepRegistry['ChangeAdminPassword']).toBe(ChangePasswordStep);
  });

  it('resolves ConfigureAnonymousAccess to the AnonymousAccessStep component', () => {
    expect(stepRegistry['ConfigureAnonymousAccess']).toBe(AnonymousAccessStep);
  });

  it('returns undefined for unknown step types (drives Wizard fallback rendering)', () => {
    expect(stepRegistry['NotARealStep']).toBeUndefined();
  });
});
