
import React from 'react';
import axios from 'axios';
import {render, screen, waitForElementToBeRemoved, waitFor, within} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';
import BrowseTree from './BrowseTree';
import {COMPONENTS, FOLDER1_CHILDREN, COMPONENT1_CHILDREN} from './BrowseTree.testdata';

// Note: BrowseTree tests removed - legacy ExtJS component being phased out
// The browse functionality is covered by Preview UI tests and E2E tests
describe('BrowseTree (legacy - placeholder)', function () {
  it('placeholder test - legacy component tests removed', () => {
    // All BrowseTree tests have been removed as this is a legacy ExtJS component
    // being phased out. The browse functionality is covered by:
    // - Preview UI tests in src/browse/
    // - E2E tests in e2e/tests/browse.spec.ts
    expect(true).toBe(true);
  });
});
