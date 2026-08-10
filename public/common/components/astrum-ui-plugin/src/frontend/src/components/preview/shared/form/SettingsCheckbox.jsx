

import React from 'react';
import PropTypes from 'prop-types';
import { Box, Text } from '@radix-ui/themes';
import { Check } from 'lucide-react';

import './SettingsCheckbox.scss';

/**
 * SettingsCheckbox - Checkbox with label and optional description
 * 
 * @example
 * <SettingsCheckbox
 *   name="enabled"
 *   label="Enable email server"
 *   checked={enabled}
 *   onChange={setEnabled}
 *   description="Allow the system to send email notifications"
 * />
 */
export function SettingsCheckbox({
  name,
  label,
  checked = false,
  onChange,
  description = '',
  disabled = false,
  className = '',
  analyticsId,
}) {
  const inputId = `settings-checkbox-${name}`;
  const descriptionId = `settings-checkbox-desc-${name}`;

  const handleChange = (e) => {
    if (onChange) {
      onChange(e.target.checked, e);
    }
  };

  return (
    <Box className={`settings-checkbox ${disabled ? 'settings-checkbox--disabled' : ''} ${className}`.trim()}>
      <label htmlFor={inputId} className="settings-checkbox__container">
        <span className={`settings-checkbox__box ${checked ? 'settings-checkbox__box--checked' : ''}`}>
          <input
            id={inputId}
            name={name}
            type="checkbox"
            checked={checked}
            onChange={handleChange}
            disabled={disabled}
            aria-describedby={description ? descriptionId : undefined}
            data-testid={`checkbox-${name}`}
            {...(analyticsId ? { 'data-analytics-id': analyticsId } : {})}
            className="settings-checkbox__input"
          />
          {checked && <Check size={14} className="settings-checkbox__icon" />}
        </span>
        <span className="settings-checkbox__content">
          <span className="settings-checkbox__label">{label}</span>
          {description && (
            <Text as="span" size="1" id={descriptionId} className="settings-checkbox__description">
              {description}
            </Text>
          )}
        </span>
      </label>
    </Box>
  );
}

SettingsCheckbox.propTypes = {
  /** Field name for form submission */
  name: PropTypes.string.isRequired,
  /** Label text */
  label: PropTypes.string.isRequired,
  /** Checked state */
  checked: PropTypes.bool,
  /** Change handler (receives checked state and event) */
  onChange: PropTypes.func,
  /** Description text below label */
  description: PropTypes.node,
  /** Disable checkbox */
  disabled: PropTypes.bool,
  /** Additional CSS class */
  className: PropTypes.string,
  /** Analytics ID for the checkbox input */
  analyticsId: PropTypes.string,
};

export default SettingsCheckbox;

