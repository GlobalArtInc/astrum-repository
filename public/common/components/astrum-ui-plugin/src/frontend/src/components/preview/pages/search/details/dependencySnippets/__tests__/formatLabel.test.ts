

import { getFormatLabel } from '../formatLabel';

describe('getFormatLabel', () => {
  it('maps the raw "maven" format to the "Maven" label', () => {
    expect(getFormatLabel('maven')).toBe('Maven');
  });

  it('maps the "maven2" registry key to the "Maven" label', () => {
    expect(getFormatLabel('maven2')).toBe('Maven');
  });

  it('returns the shared human-readable labels for known formats', () => {
    expect(getFormatLabel('npm')).toBe('npm');
    expect(getFormatLabel('nuget')).toBe('NuGet');
    expect(getFormatLabel('pypi')).toBe('PyPI');
    expect(getFormatLabel('oci')).toBe('OCI');
    expect(getFormatLabel('rubygems')).toBe('RubyGems');
  });

  it('is case-insensitive, resolving aliases and labels for upper-case formats', () => {
    expect(getFormatLabel('Maven')).toBe('Maven');
    expect(getFormatLabel('MAVEN')).toBe('Maven');
    expect(getFormatLabel('NPM')).toBe('npm');
  });

  it('falls back to the raw format id when unknown', () => {
    expect(getFormatLabel('somenewformat')).toBe('somenewformat');
  });
});
