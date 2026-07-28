
import React from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faCaretUp, faCaretDown, faMinus } from '@fortawesome/free-solid-svg-icons';

const ChangeIcon = ({ value }) => {
  let icon = null;

  if (value === 'N/A') {
    icon = faMinus;
  } else if (value > 0) {
    icon = faCaretUp;
  } else if (value < 0) {
    icon = faCaretDown;
  }

  return icon ? <FontAwesomeIcon icon={icon} /> : null;
};

export default ChangeIcon;
