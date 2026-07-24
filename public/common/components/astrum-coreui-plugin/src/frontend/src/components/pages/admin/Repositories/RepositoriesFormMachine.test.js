
import {evaluationSettingsUrl} from './RepositoriesFormMachine';

describe('RepositoriesFormMachine URL helpers', () => {
  describe('evaluationSettingsUrl', () => {
    it('does not start with / so Axios baseURL is respected', () => {
      expect(evaluationSettingsUrl('my-repo').startsWith('/')).toBe(false);
    });

    it('produces the correct path for a simple repository name', () => {
      expect(evaluationSettingsUrl('my-repo')).toBe(
        'service/rest/v1/repositories/my-repo/evaluation-settings'
      );
    });

    it('percent-encodes repository names with special characters', () => {
      expect(evaluationSettingsUrl('my/repo')).toBe(
        'service/rest/v1/repositories/my%2Frepo/evaluation-settings'
      );
    });
  });
});
