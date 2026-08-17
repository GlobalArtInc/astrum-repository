

import type { TagDetail } from '../tags.types';

/**
 * Mock tag detail for testing.
 */
export const mockTagDetail: TagDetail = {
  name: 'release-1.0',
  firstCreated: '2026-01-15T10:30:45Z',
  lastUpdated: '2026-01-18T14:15:22Z',
  attributes: {
    env: 'production',
    version: '1.0.0',
    buildId: '142',
  },
};


