

import { isFirewallSupportedFormat, FIREWALL_SUPPORTED_FORMATS } from '../firewallFormats';

describe('firewallFormats', () => {
  describe('FIREWALL_SUPPORTED_FORMATS', () => {
    it('includes all 17 supported formats', () => {
      expect(FIREWALL_SUPPORTED_FORMATS.size).toBe(17);
    });

    it('includes known supported formats', () => {
      const expected = ['maven2', 'npm', 'nuget', 'pypi', 'docker', 'cargo', 'go', 'rubygems'];
      expected.forEach((fmt) => {
        expect(FIREWALL_SUPPORTED_FORMATS.has(fmt)).toBe(true);
      });
    });

    it('does not include unsupported formats', () => {
      const unsupported = ['terraform', 'apt', 'helm', 'gitlfs', 'swift'];
      unsupported.forEach((fmt) => {
        expect(FIREWALL_SUPPORTED_FORMATS.has(fmt)).toBe(false);
      });
    });
  });

  describe('isFirewallSupportedFormat', () => {
    it('returns true for supported formats', () => {
      expect(isFirewallSupportedFormat('maven2')).toBe(true);
      expect(isFirewallSupportedFormat('npm')).toBe(true);
      expect(isFirewallSupportedFormat('docker')).toBe(true);
      expect(isFirewallSupportedFormat('cargo')).toBe(true);
      expect(isFirewallSupportedFormat('nuget')).toBe(true);
    });

    it('returns false for unsupported formats', () => {
      expect(isFirewallSupportedFormat('terraform')).toBe(false);
      expect(isFirewallSupportedFormat('apt')).toBe(false);
      expect(isFirewallSupportedFormat('helm')).toBe(false);
      expect(isFirewallSupportedFormat('gitlfs')).toBe(false);
      expect(isFirewallSupportedFormat('swift')).toBe(false);
    });

    it('is case-insensitive', () => {
      expect(isFirewallSupportedFormat('Maven2')).toBe(true);
      expect(isFirewallSupportedFormat('NPM')).toBe(true);
      expect(isFirewallSupportedFormat('TERRAFORM')).toBe(false);
    });

    it('returns false for null, undefined, and empty string', () => {
      expect(isFirewallSupportedFormat(null)).toBe(false);
      expect(isFirewallSupportedFormat(undefined)).toBe(false);
      expect(isFirewallSupportedFormat('')).toBe(false);
    });

    it('includes p2 and raw (backend enum but undocumented - treated as supported)', () => {
      expect(isFirewallSupportedFormat('p2')).toBe(true);
      expect(isFirewallSupportedFormat('raw')).toBe(true);
    });
  });
});
