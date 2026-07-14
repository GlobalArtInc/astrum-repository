

import React, { useState } from 'react';
import { NxErrorAlert, NxH3, NxInfoAlert, NxSuccessAlert, NxWarningAlert } from '@globalart/react-shared-components';

/**
 *
 * @param title
 * @param noticeLevel info | warning | error | success
 * @param children
 * @param className
 * @param onClose
 * @returns {React.JSX.Element|null}
 * @constructor
 */
export default function SystemNotice({
  title,
  noticeLevel,
  children,
  additionalAlertClassNames,
  nonDismissable,
  onClose
}) {
  const [isOpen, setIsOpen] = useState(true);
  const isTitle = !!title;

  if (isClosed()) {
    return null;
  }

  const Alert = getWrapperComponentFromNoticeLevel(noticeLevel);

  return (
      <div
          className="as-system-notice nx-system-notice nx-system-notice--alert as-system-notice"
          role="complementary"
          aria-label="alert system notice"
      >
        <Alert className={additionalAlertClassNames} onClose={nonDismissable ? undefined : onInternalClose}>
          { isTitle && <NxH3>{title}</NxH3> }
          {children}
        </Alert>
      </div>);

  function onInternalClose() {
    setIsOpen(false);

    if (onClose) {
      onClose();
    }
  }

  function isClosed() {
    return !isOpen
  }

  function getWrapperComponentFromNoticeLevel(noticeLevel) {
    if (noticeLevel === 'warning') {
      return NxWarningAlert;
    } else if (noticeLevel === 'error') {
      return NxErrorAlert;
    } else if (noticeLevel === 'info') {
      return NxInfoAlert;
    } else if (noticeLevel === 'success') {
      return NxSuccessAlert;
    } else {
      return NxInfoAlert;
    }
  }
}
