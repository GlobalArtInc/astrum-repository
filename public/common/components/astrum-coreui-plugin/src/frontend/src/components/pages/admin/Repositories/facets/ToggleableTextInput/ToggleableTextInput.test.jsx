
import React from 'react';
import {render, screen} from '@testing-library/react';

import ToggleableTextInput from './ToggleableTextInput';

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  FormUtils: {
    fieldProps: jest.fn(() => ({value: ''}))
  }
}));

describe('ToggleableTextInput', () => {
  const sendParent = jest.fn();

  const makeParentMachine = (initialValue = null) => [
    {
      context: {
        data: {
          oci: {
            httpPort: initialValue
          }
        }
      }
    },
    sendParent
  ];

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the field label', () => {
    render(
      <ToggleableTextInput
        parentMachine={makeParentMachine()}
        contextPropName="oci.httpPort"
        label="HTTP Connector Port"
        sublabel="Create an HTTP connector"
      />
    );
    expect(screen.getByText('HTTP Connector Port')).toBeInTheDocument();
  });

  // ASTRUM-53064 B3: the toggle checkbox sits in the sublabel row, NOT inside
  //   the input row. This verifies the new DOM tree.
  it('places the toggle checkbox alongside the sublabel, not next to the input', () => {
    const {container} = render(
      <ToggleableTextInput
        parentMachine={makeParentMachine()}
        contextPropName="oci.httpPort"
        label="HTTP Connector Port"
        sublabel="Create an HTTP connector"
      />
    );

    const sublabelRow = container.querySelector('.as-toggleable-sublabel-row');
    expect(sublabelRow).not.toBeNull();

    // The toggle checkbox must be a descendant of the sublabel row.
    const checkboxInSublabel = sublabelRow.querySelector('input[type="checkbox"]');
    expect(checkboxInSublabel).not.toBeNull();
    // NxCheckbox places the aria-label on the wrapper .nx-checkbox node,
    // so look for it on the closest ancestor with that class.
    const checkboxWrapper = checkboxInSublabel.closest('.nx-checkbox');
    expect(checkboxWrapper).not.toBeNull();
    expect(checkboxWrapper.getAttribute('aria-label')).toBe('Toggle Text Input');

    // The input row must NOT contain a checkbox any more.
    const inputContainer = container.querySelector('.as-toggleable-text-input-container');
    expect(inputContainer).not.toBeNull();
    expect(inputContainer.querySelector('input[type="checkbox"]')).toBeNull();
  });

  it('renders the sublabel text alongside the checkbox', () => {
    render(
      <ToggleableTextInput
        parentMachine={makeParentMachine()}
        contextPropName="oci.httpPort"
        label="HTTP Connector Port"
        sublabel="Create an HTTP connector"
      />
    );
    expect(screen.getByText('Create an HTTP connector')).toBeInTheDocument();
  });

  it('defaults the toggle to unchecked when no value is supplied', () => {
    render(
      <ToggleableTextInput
        parentMachine={makeParentMachine(null)}
        contextPropName="oci.httpPort"
        label="HTTP Connector Port"
        sublabel="Create an HTTP connector"
      />
    );
    const checkbox = screen.getByRole('checkbox', {name: /toggle text input/i});
    expect(checkbox).not.toBeChecked();
  });

  it('marks the toggle as checked when an initial value is supplied', () => {
    render(
      <ToggleableTextInput
        parentMachine={makeParentMachine(8443)}
        contextPropName="oci.httpPort"
        label="HTTPS Connector Port"
        sublabel="Create an HTTPS connector"
      />
    );
    const checkbox = screen.getByRole('checkbox', {name: /toggle text input/i});
    expect(checkbox).toBeChecked();
  });
});
