
import React from 'react';
import {render, screen} from '@testing-library/react';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import NugetSymbolServerConfiguration from './NugetSymbolServerConfiguration';
import UIStrings from '../../../../../constants/UIStrings';

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    useState: jest.fn(),
    state: jest.fn()
  }
}));

const {NUGET} = UIStrings.REPOSITORIES.EDITOR;

describe('NugetSymbolServerConfiguration', () => {
  const parentMachine = [
    {
      context: {
        data: {nugetProxy: {symbolServerUrl: '', allowAnonymousSymbolAccess: true}},
        pristineData: {nugetProxy: {symbolServerUrl: '', allowAnonymousSymbolAccess: true}}
      }
    },
    jest.fn()
  ];

  beforeEach(() => {
    jest.clearAllMocks();
  });

  const mockFlag = (enabled) => {
    ExtJS.useState.mockImplementation((selector) => selector());
    ExtJS.state.mockReturnValue({
      getValue: jest.fn().mockReturnValue(enabled)
    });
  };

  it('renders nothing when the symbol server feature flag is off', () => {
    mockFlag(false);

    const {container} = render(<NugetSymbolServerConfiguration parentMachine={parentMachine} />);

    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByText(NUGET.SYMBOL_SERVER_URL.LABEL)).not.toBeInTheDocument();
    expect(screen.queryByText(NUGET.ALLOW_ANONYMOUS_SYMBOL_ACCESS.LABEL)).not.toBeInTheDocument();
  });

  it('renders the Symbol Server URL and Anonymous Access fields when the flag is on', () => {
    mockFlag(true);

    render(<NugetSymbolServerConfiguration parentMachine={parentMachine} />);

    expect(screen.getByText(NUGET.SYMBOL_SERVER_URL.LABEL)).toBeInTheDocument();
    // The checkbox label text appears twice: once as the AsFormGroup label, once as the checkbox
    // child text. Use getAllByText to accept both occurrences.
    expect(screen.getAllByText(NUGET.ALLOW_ANONYMOUS_SYMBOL_ACCESS.LABEL).length).toBeGreaterThan(0);
  });
});
