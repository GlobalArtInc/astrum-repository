
import React from 'react';
import classNames from 'classnames';

import {faChevronRight} from '@fortawesome/free-solid-svg-icons';
import {NxFontAwesomeIcon} from '@globalart/react-shared-components';

export default function QuickAction({title, subTitle, icon, action, className}) {
  const handleEnter = (event) => {
    if (event.key === 'Enter') {
      action();
    }
  };

  return (
      <div
          className={classNames('as-quick-action', className)}
          onClick={action}
          onKeyDown={handleEnter}
      >
        <div
            className="as-quick-action-outline"
            tabIndex="0"
            role="button"
        >
          <div className="as-quick-action-body">
            <div className="as-quick-action-logo">
              <div className="as-quick-action-icon-container">
                <NxFontAwesomeIcon icon={icon} />
              </div>
            </div>
            <div className="as-quick-action-name">
              <h4 className="as-quick-action-title">{title}</h4>
              <div className="as-quick-action-subtitle">{subTitle}</div>
            </div>
            <div className="as-quick-action-chevron">
              <NxFontAwesomeIcon
                  icon={faChevronRight}
                  transform="down-1"
                  className="as-quick-action-chevron-icon"
              />
            </div>
          </div>
        </div>
      </div>
  );
}
