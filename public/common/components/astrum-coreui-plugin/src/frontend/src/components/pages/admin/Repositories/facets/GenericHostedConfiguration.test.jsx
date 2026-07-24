
import React from 'react';
import {render, screen} from '@testing-library/react';
import {useMachine} from '@xstate/react';

import GenericHostedConfiguration from './GenericHostedConfiguration';
import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

jest.mock('@xstate/react', () => ({
  useMachine: jest.fn()
}));

jest.mock('./DockerRedeployLatesConfiguration', () => ({
  __esModule: true,
  default: () => <div data-testid="docker-redeploy-latest-config">Docker Redeploy Latest Configuration</div>
}));

describe('GenericHostedConfiguration', () => {
  const mockSendParent = jest.fn();

  const createParentMachine = (format, type = 'hosted') => [
    {
      context: {
        data: {
          format,
          type,
          storage: {
            writePolicy: 'ALLOW_ONCE'
          },
          component: {
            proprietaryComponents: false
          }
        }
      }
    },
    mockSendParent
  ];

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should render deployment policy dropdown', () => {
    const parentMachine = createParentMachine('maven2');
    render(<GenericHostedConfiguration parentMachine={parentMachine} />);

    expect(screen.getByText(EDITOR.DEPLOYMENT_POLICY_LABEL)).toBeInTheDocument();
  });

  it('should show proprietary components checkbox for non-Docker formats', () => {
    const parentMachine = createParentMachine('maven2');
    render(<GenericHostedConfiguration parentMachine={parentMachine} />);

    expect(screen.getByText(EDITOR.PROPRIETARY_COMPONENTS_LABEL)).toBeInTheDocument();
    expect(screen.getByText(EDITOR.PROPRIETARY_COMPONENTS_DESCR)).toBeInTheDocument();
  });

  it('should show proprietary components checkbox for npm format', () => {
    const parentMachine = createParentMachine('npm');
    render(<GenericHostedConfiguration parentMachine={parentMachine} />);

    expect(screen.getByText(EDITOR.PROPRIETARY_COMPONENTS_LABEL)).toBeInTheDocument();
    expect(screen.getByText(EDITOR.PROPRIETARY_COMPONENTS_DESCR)).toBeInTheDocument();
  });

  it('should show proprietary components checkbox for pypi format', () => {
    const parentMachine = createParentMachine('pypi');
    render(<GenericHostedConfiguration parentMachine={parentMachine} />);

    expect(screen.getByText(EDITOR.PROPRIETARY_COMPONENTS_LABEL)).toBeInTheDocument();
    expect(screen.getByText(EDITOR.PROPRIETARY_COMPONENTS_DESCR)).toBeInTheDocument();
  });

  it('should hide proprietary components checkbox for Docker format', () => {
    const parentMachine = createParentMachine('docker');
    render(<GenericHostedConfiguration parentMachine={parentMachine} />);

    expect(screen.queryByText(EDITOR.PROPRIETARY_COMPONENTS_LABEL)).not.toBeInTheDocument();
    expect(screen.queryByText(EDITOR.PROPRIETARY_COMPONENTS_DESCR)).not.toBeInTheDocument();
  });

  it('should show DockerRedeployLatesConfiguration for Docker format', () => {
    const parentMachine = createParentMachine('docker');
    render(<GenericHostedConfiguration parentMachine={parentMachine} />);

    expect(screen.getByTestId('docker-redeploy-latest-config')).toBeInTheDocument();
  });

  it('should not show DockerRedeployLatesConfiguration for non-Docker formats', () => {
    const parentMachine = createParentMachine('maven2');
    render(<GenericHostedConfiguration parentMachine={parentMachine} />);

    expect(screen.queryByTestId('docker-redeploy-latest-config')).not.toBeInTheDocument();
  });
});