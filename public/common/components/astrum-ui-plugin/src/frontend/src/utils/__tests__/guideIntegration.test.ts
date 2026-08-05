

import {
  buildGuideComponentUrl,
  buildGuideComponentUrlFromGaId,
  guidePackageNameFromGaId,
  isGuideSupported,
  isGuideSupportedForGaId,
  parseMavenCoordinates,
} from '../guideIntegration';

describe('guideIntegration', () => {
  describe('buildGuideComponentUrl', () => {
    it('builds npm URL with simple package name', () => {
      expect(buildGuideComponentUrl('npm', 'lodash', '4.17.21')).toBe(
        'https://guide.globalart.dev/component/npm/lodash/4.17.21'
      );
    });

    it('builds maven URL with coordinates', () => {
      expect(buildGuideComponentUrl('maven2', 'org.apache.commons:commons-lang3', '3.12.0')).toBe(
        'https://guide.globalart.dev/component/maven/org.apache.commons%3Acommons-lang3/3.12.0'
      );
    });

    it('builds pypi URL', () => {
      expect(buildGuideComponentUrl('pypi', 'requests', '2.28.1')).toBe(
        'https://guide.globalart.dev/component/pypi/requests/2.28.1'
      );
    });

    it('builds nuget URL', () => {
      expect(buildGuideComponentUrl('nuget', 'Newtonsoft.Json', '13.0.1')).toBe(
        'https://guide.globalart.dev/component/nuget/Newtonsoft.Json/13.0.1'
      );
    });

    it('encodes special characters in package name', () => {
      expect(buildGuideComponentUrl('npm', '@angular/core', '15.0.0')).toBe(
        'https://guide.globalart.dev/component/npm/%40angular%2Fcore/15.0.0'
      );
    });

    it('encodes special characters in version', () => {
      expect(buildGuideComponentUrl('npm', 'test', '1.0.0-beta+build')).toBe(
        'https://guide.globalart.dev/component/npm/test/1.0.0-beta%2Bbuild'
      );
    });

    it('returns null for unsupported ecosystem', () => {
      expect(buildGuideComponentUrl('docker', 'nginx', 'latest')).toBeNull();
      expect(buildGuideComponentUrl('helm', 'wordpress', '1.0.0')).toBeNull();
    });

    it('handles case insensitive ecosystem', () => {
      expect(buildGuideComponentUrl('NPM', 'lodash', '4.17.21')).toBe(
        'https://guide.globalart.dev/component/npm/lodash/4.17.21'
      );
    });
  });

  describe('isGuideSupported', () => {
    it('returns true for supported ecosystems', () => {
      expect(isGuideSupported('npm')).toBe(true);
      expect(isGuideSupported('maven2')).toBe(true);
      expect(isGuideSupported('maven')).toBe(true);
      expect(isGuideSupported('pypi')).toBe(true);
      expect(isGuideSupported('nuget')).toBe(true);
    });

    it('returns false for unsupported ecosystems', () => {
      expect(isGuideSupported('docker')).toBe(false);
      expect(isGuideSupported('helm')).toBe(false);
      expect(isGuideSupported('apt')).toBe(false);
    });

    it('handles case insensitive check', () => {
      expect(isGuideSupported('NPM')).toBe(true);
      expect(isGuideSupported('Maven2')).toBe(true);
    });
  });

  describe('guidePackageNameFromGaId', () => {
    it('strips format prefix for simple npm package', () => {
      expect(guidePackageNameFromGaId('npm:lodash')).toBe('lodash');
    });

    it('maps scoped npm gaId to slash form', () => {
      expect(guidePackageNameFromGaId('npm:@angular:core')).toBe('@angular/core');
    });

    it('returns maven coordinates after format', () => {
      expect(guidePackageNameFromGaId('maven2:org.apache.commons:commons-lang3')).toBe(
        'org.apache.commons:commons-lang3'
      );
    });
  });

  describe('buildGuideComponentUrlFromGaId', () => {
    it('builds npm URL from gaId', () => {
      expect(
        buildGuideComponentUrlFromGaId('npm:lodash', '4.17.20', 'search-component-detail')
      ).toBe(
        'https://guide.globalart.dev/component/npm/lodash/4.17.20?referrer=search-component-detail'
      );
    });

    it('builds scoped npm URL from gaId', () => {
      expect(buildGuideComponentUrlFromGaId('npm:@types:node', '20.0.0')).toBe(
        'https://guide.globalart.dev/component/npm/%40types%2Fnode/20.0.0'
      );
    });

    it('builds maven URL when gaId uses maven prefix', () => {
      expect(
        buildGuideComponentUrlFromGaId('maven:org.springframework:spring-core', '1.2.2')
      ).toBe(
        'https://guide.globalart.dev/component/maven/org.springframework%3Aspring-core/1.2.2'
      );
    });

    it('returns null for unsupported format', () => {
      expect(buildGuideComponentUrlFromGaId('docker:nginx', 'latest')).toBeNull();
    });
  });

  describe('isGuideSupportedForGaId', () => {
    it('returns true for supported gaId prefixes', () => {
      expect(isGuideSupportedForGaId('npm:lodash')).toBe(true);
      expect(isGuideSupportedForGaId('maven:g:a')).toBe(true);
    });

    it('returns false for unsupported or invalid gaId', () => {
      expect(isGuideSupportedForGaId('docker:nginx')).toBe(false);
      expect(isGuideSupportedForGaId('')).toBe(false);
    });
  });

  describe('parseMavenCoordinates', () => {
    it('parses standard Maven coordinates', () => {
      const result = parseMavenCoordinates('org.apache.commons:commons-lang3');
      expect(result).toEqual({
        group: 'org.apache.commons',
        artifact: 'commons-lang3',
        combined: 'org.apache.commons:commons-lang3',
      });
    });

    it('returns null for invalid coordinates', () => {
      expect(parseMavenCoordinates('invalid')).toBeNull();
      expect(parseMavenCoordinates('too:many:colons')).toBeNull();
    });
  });
});
