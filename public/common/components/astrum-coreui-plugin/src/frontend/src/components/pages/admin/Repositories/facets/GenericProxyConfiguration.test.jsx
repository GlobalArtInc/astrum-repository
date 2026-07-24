
import React from 'react';
import {render, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import GenericProxyConfiguration from './GenericProxyConfiguration';
import UIStrings from '../../../../../constants/UIStrings';

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    state: jest.fn()
  },
  UseAstrumTruststore: () => null
}));

jest.mock('./DockerIndexConfiguration', () => ({
  __esModule: true,
  default: () => null
}));

jest.mock('./DockerForeignLayerConfiguration', () => ({
  __esModule: true,
  default: () => null
}));

const {EDITOR} = UIStrings.REPOSITORIES;

describe('GenericProxyConfiguration', () => {
  const mockSend = jest.fn();

  const createParentMachine = (preserveEncodedCharacters = false) => [
    {
      context: {
        data: {
          format: 'maven2',
          type: 'proxy',
          replication: {preemptivePullEnabled: false},
          proxy: {
            remoteUrl: 'https://repo.example.com',
            contentMaxAge: -1,
            metadataMaxAge: 1440,
            preserveEncodedCharacters
          },
          httpClient: {
            blocked: false,
            autoBlock: true,
            connection: null,
            authentication: null
          }
        }
      }
    },
    mockSend
  ];

  beforeEach(() => {
    jest.clearAllMocks();
    ExtJS.state.mockReturnValue({
      getValue: jest.fn().mockReturnValue(false)
    });
  });

  it('renders the Preserve Encoded Characters checkbox', () => {
    render(<GenericProxyConfiguration parentMachine={createParentMachine()} />);

    expect(screen.getByText(EDITOR.PRESERVE_ENCODED_CHARS_LABEL)).toBeInTheDocument();
  });

  it('renders the Preserve Encoded Characters sublabel', () => {
    render(<GenericProxyConfiguration parentMachine={createParentMachine()} />);

    expect(screen.getByText(EDITOR.PRESERVE_ENCODED_CHARS_SUBLABEL)).toBeInTheDocument();
  });

  it('renders checkbox as unchecked by default', () => {
    render(<GenericProxyConfiguration parentMachine={createParentMachine(false)} />);

    const label = screen.getByText(EDITOR.PRESERVE_ENCODED_CHARS_LABEL);
    const checkbox = label.closest('.nx-form-group').querySelector('input[type="checkbox"]');
    expect(checkbox).not.toBeChecked();
  });

  it('dispatches UPDATE when Preserve Encoded Characters checkbox is toggled', () => {
    render(<GenericProxyConfiguration parentMachine={createParentMachine(false)} />);

    const label = screen.getByText(EDITOR.PRESERVE_ENCODED_CHARS_LABEL);
    const formGroup = label.closest('.nx-form-group');
    const checkbox = formGroup.querySelector('input[type="checkbox"]');

    userEvent.click(checkbox);

    expect(mockSend).toHaveBeenCalledWith({
      type: 'UPDATE',
      name: 'proxy.preserveEncodedCharacters',
      value: true
    });
  });
});
