

import { ActionIcons, StatusIcons, NavIcons } from '../index';

describe('Icon Registries', () => {
  describe('ActionIcons', () => {
    it('maps all semantic action names', () => {
      expect(ActionIcons.Delete).toBeDefined();
      expect(ActionIcons.Add).toBeDefined();
      expect(ActionIcons.Edit).toBeDefined();
      expect(ActionIcons.Save).toBeDefined();
      expect(ActionIcons.Cancel).toBeDefined();
      expect(ActionIcons.Search).toBeDefined();
      expect(ActionIcons.Back).toBeDefined();
      expect(ActionIcons.Refresh).toBeDefined();
      expect(ActionIcons.Download).toBeDefined();
      expect(ActionIcons.ExternalLink).toBeDefined();
      expect(ActionIcons.Copy).toBeDefined();
      expect(ActionIcons.Settings).toBeDefined();
    });

    it('maps to Lucide components (functions)', () => {
      Object.values(ActionIcons).forEach(icon => {
        expect(typeof icon).toBe('object');
      });
    });
  });

  describe('StatusIcons', () => {
    it('maps all status names', () => {
      expect(StatusIcons.Loading).toBeDefined();
      expect(StatusIcons.Error).toBeDefined();
      expect(StatusIcons.Warning).toBeDefined();
      expect(StatusIcons.Success).toBeDefined();
      expect(StatusIcons.Info).toBeDefined();
    });
  });

  describe('NavIcons', () => {
    it('maps all navigation names', () => {
      expect(NavIcons.Forward).toBeDefined();
      expect(NavIcons.Back).toBeDefined();
      expect(NavIcons.Expand).toBeDefined();
      expect(NavIcons.Collapse).toBeDefined();
      expect(NavIcons.Return).toBeDefined();
    });
  });
});
