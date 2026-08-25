

import { parseGaId } from '../gaIdUtils';

describe('parseGaId', () => {
  it('splits a namespaced identifier into format, group and name', () => {
    expect(parseGaId('maven2:org.test:artifact')).toEqual({
      format: 'maven2',
      group: 'org.test',
      name: 'artifact',
    });
  });

  it('reads the second segment as the name for formats without a namespace', () => {
    expect(parseGaId('npm:lodash')).toEqual({ format: 'npm', group: '', name: 'lodash' });
  });

  it('returns a lone segment as the name with no format, since the two are indistinguishable', () => {
    expect(parseGaId('lodash')).toEqual({ format: '', group: '', name: 'lodash' });
  });

  it('keeps only the first three segments when a name contains colons', () => {
    // Scoped npm names arrive as e.g. npm:@scope:pkg; anything beyond the third segment is
    // dropped rather than folded into the name.
    expect(parseGaId('npm:@scope:pkg:extra')).toEqual({
      format: 'npm',
      group: '@scope',
      name: 'pkg',
    });
  });

  it('treats the empty string as an empty name rather than throwing', () => {
    expect(parseGaId('')).toEqual({ format: '', group: '', name: '' });
  });
});
