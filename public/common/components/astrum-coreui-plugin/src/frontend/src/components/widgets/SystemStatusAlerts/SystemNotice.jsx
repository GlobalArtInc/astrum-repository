

import React, { useState } from 'react';
import { AsErrorAlert, AsH3, AsInfoAlert, AsSuccessAlert, AsWarningAlert } from '@globalart/astrum-react-shared-components';

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
          className="astrum-system-notice as-system-notice as-system-notice--alert astrum-system-notice"
          role="complementary"
          aria-label="alert system notice"
      >
        <Alert className={additionalAlertClassNames} onClose={nonDismissable ? undefined : onInternalClose}>
          { isTitle && <AsH3>{title}</AsH3> }
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
      return AsWarningAlert;
    } else if (noticeLevel === 'error') {
      return AsErrorAlert;
    } else if (noticeLevel === 'info') {
      return AsInfoAlert;
    } else if (noticeLevel === 'success') {
      return AsSuccessAlert;
    } else {
      return AsInfoAlert;
    }
  }
}
