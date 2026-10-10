
import React from 'react';
import {render, screen, fireEvent} from '@testing-library/react';
import {useMachine} from '@xstate/react';
import CleanupPoliciesPreview from './CleanupPoliciesPreview';

// Mock the useMachine hook
jest.mock('@xstate/react');

// Mock AsWarningAlert component
jest.mock('@globalart/astrum-react-shared-components', () => ({
  ...jest.requireActual('@globalart/astrum-react-shared-components'),
  AsWarningAlert: ({onClose, children}) => (
    <div data-testid="as-warning-alert">
      <button data-testid="close-button" onClick={onClose}>X</button>
      {children}
    </div>
  ),
  AsButton: ({onClick, children, className}) => (
    <button onClick={onClick} className={className}>{children}</button>
  ),
  AsFilterInput: () => <input />,
  AsFormSelect: ({onChange, value, children}) => (
    <select onChange={(e) => onChange(e.target.value)} value={value}>
      {children}
    </select>
  ),
  AsLoadWrapper: ({children}) => <>{children()}</>,
  AsTable: ({children}) => <table>{children}</table>,
  AsTableBody: ({children}) => <tbody>{children}</tbody>,
  AsTableCell: ({children}) => <td>{children}</td>,
  AsTableHead: ({children}) => <thead>{children}</thead>,
  AsTableRow: ({children}) => <tr>{children}</tr>,
  AsTooltip: ({children}) => <>{children}</>
}));

describe('CleanupPoliciesPreview', () => {
  const mockPolicyData = {
    format: 'maven2',
    criteriaLastBlobUpdated: '10'
  };

  const mockSendToForm = jest.fn();
  const mockSendToList = jest.fn();

  const defaultFormState = {
    context: {
      error: null,
      repository: '',
      repositories: [{id: 'repo1', name: 'Repo 1'}]
    },
    matches: jest.fn().mockReturnValue(false)
  };

  const defaultListState = {
    context: {
      error: null,
      data: [{name: 'comp1', group: 'group1', version: '1.0'}],
      total: 100,
      filter: '',
      isAlertShown: true
    },
    matches: jest.fn().mockReturnValue(false)
  };

  beforeEach(() => {
    jest.clearAllMocks();
    useMachine
      .mockReturnValueOnce([defaultFormState, mockSendToForm])
      .mockReturnValueOnce([defaultListState, mockSendToList]);
  });

  it('should send HIDE_ALERT to list machine when alert close button is clicked', async () => {
    render(<CleanupPoliciesPreview policyData={mockPolicyData} />);

    // Verify the alert is shown
    expect(screen.getByTestId('as-warning-alert')).toBeInTheDocument();

    // Click the close button
    fireEvent.click(screen.getByTestId('close-button'));

    // Verify HIDE_ALERT was sent to the list machine, not the form machine
    expect(mockSendToList).toHaveBeenCalledWith({type: 'HIDE_ALERT'});
    expect(mockSendToForm).not.toHaveBeenCalledWith({type: 'HIDE_ALERT'});
  });

  it('should not show alert when isAlertShown is false', () => {
    const listStateWithoutAlert = {
      ...defaultListState,
      context: {
        ...defaultListState.context,
        isAlertShown: false
      }
    };

    // Clear previous mock implementations and set new ones
    useMachine.mockReset();
    useMachine
      .mockReturnValueOnce([defaultFormState, mockSendToForm])
      .mockReturnValueOnce([listStateWithoutAlert, mockSendToList]);

    render(<CleanupPoliciesPreview policyData={mockPolicyData} />);

    expect(screen.queryByTestId('as-warning-alert')).not.toBeInTheDocument();
  });
});
