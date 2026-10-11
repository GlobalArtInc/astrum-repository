

import React from 'react';
import PropTypes from 'prop-types';
import { AlertCircle, CheckCircle, Info, AlertTriangle, X } from 'lucide-react';

import './SettingsAlert.scss';
import SharedStrings from '../../constants/SharedStrings';

const ICONS = {
  error: AlertCircle,
  success: CheckCircle,
  info: Info,
  warning: AlertTriangle,
};

/**
 * SettingsAlert - Alert/notification banner
 * 
 * @example
 * <SettingsAlert type="warning">
 *   Anonymous access allows anyone to browse repositories.
 * </SettingsAlert>
 * 
 * <SettingsAlert type="error" onClose={() => setError(null)}>
 *   Failed to save settings: Connection timeout
 * </SettingsAlert>
 */
export function SettingsAlert({
  type = 'info',
  children,
  onClose,
  className = '',
  ...restProps
}) {
  const Icon = ICONS[type] || Info;

  // Use role="alert" (assertive) for errors, aria-live="polite" for others
  const ariaProps = type === 'error'
    ? { role: 'alert' }
    : { 'aria-live': 'polite', role: 'status' };

  return (
    <div
      className={`settings-alert settings-alert--${type} ${className}`.trim()}
      {...ariaProps}
      {...restProps}
    >
      <Icon size={18} className="settings-alert__icon" />
      <div className="settings-alert__content">
        {children}
      </div>
      {onClose && (
        <button
          type="button"
          className="settings-alert__close"
          onClick={onClose}
          aria-label={SharedStrings.SETTINGS_ALERT.DISMISS_ARIA}
        >
          <X size={16} />
        </button>
      )}
    </div>
  );
}

SettingsAlert.propTypes = {
  /** Alert type determines color scheme */
  type: PropTypes.oneOf(['error', 'success', 'info', 'warning']),
  /** Alert message content */
  children: PropTypes.node.isRequired,
  /** Optional close handler (shows close button when provided) */
  onClose: PropTypes.func,
  /** Additional CSS class */
  className: PropTypes.string,
};

export default SettingsAlert;

