
import React, {useEffect, useState} from 'react';

import {path} from 'ramda';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxCheckbox, NxFieldset, NxTextInput} from '@globalart/react-shared-components';

import './ToggleableTextInput.scss';

export default function ToggleableTextInput({
  parentMachine,
  contextPropName,
  defaultValue = '',
  label,
  sublabel = '',
  placeholder = '',
  className = '',
  clearIfDisabled = false,
  onToggle
}) {
  const [currentParent, sendParent] = parentMachine;

  const actualValue = path(contextPropName.split('.'), currentParent.context.data);

  const [isEnabled, setIsEnabled] = useState(!!actualValue);

  const [tempValue, setTempValue] = useState(actualValue ? actualValue.toString() : '');

  // Sync tempValue when the machine context is updated externally (e.g. fetchSuggestedPort).
  // Only applies to primitive values (string/number) — guards against unexpected types.
  useEffect(() => {
    if (actualValue != null && actualValue !== '' &&
        (typeof actualValue === 'string' || typeof actualValue === 'number')) {
      setTempValue(prev => {
        const str = String(actualValue);
        return prev !== str ? str : prev;
      });
    }
  }, [actualValue]);

  const updateActualValue = (value) => sendParent({type: 'UPDATE', name: contextPropName, value});

  const toggleCheckbox = () => {
    const newIsEnabled = !isEnabled;

    if (newIsEnabled) {
      if (tempValue) {
          updateActualValue(tempValue);
      } else {
        updateActualValue(defaultValue);
        setTempValue(defaultValue);
      }
    } else {
      updateActualValue(null)
    }

    setIsEnabled(newIsEnabled);

    if (onToggle) {
      onToggle(newIsEnabled);
    }
  };

  const handleTextInputChange = (value) => {
    updateActualValue(value);
    setTempValue(value);
  };

  const displayedValue = !isEnabled && clearIfDisabled ? '' : tempValue;

  // ASTRUM-53064 B3: render the toggle checkbox alongside the sublabel rather
  //   than next to the input field. The checkbox controls whether this field
  //   is active, so it belongs with the descriptive copy, not crammed into the
  //   input row where it pushed the input flush against the form gutter.
  // ASTRUM-53064 B4: lifting the checkbox out of the input row gives the input
  //   the full available width, which fixes the right-edge truncation that
  //   was visible on Subdomain / HTTP / HTTPS fields.
  const sublabelWithCheckbox = (
    <span className="as-toggleable-sublabel-row">
      {sublabel ? (
        <span className="as-toggleable-sublabel-text">{sublabel}</span>
      ) : null}
      <NxCheckbox
        onChange={toggleCheckbox}
        isChecked={isEnabled}
        aria-label="Toggle Text Input"
      />
    </span>
  );

  return (
    <NxFieldset label={label} sublabel={sublabelWithCheckbox} className={className}>
      <div className="as-toggleable-text-input-container">
        <NxTextInput
          {...FormUtils.fieldProps(contextPropName, currentParent)}
          value={displayedValue}
          onChange={handleTextInputChange}
          disabled={!isEnabled}
          placeholder={placeholder}
        />
      </div>
    </NxFieldset>
  );
}
