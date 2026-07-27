
import React from 'react';
import {NxFontAwesomeIcon} from '@globalart/react-shared-components';
import {faChevronLeft} from '@fortawesome/free-solid-svg-icons';
import './BackButton.scss';

export default function BackButton({onClick, text}) {
  return (
        <button className="as-back-button nx-text-link" onClick={onClick}>
          <NxFontAwesomeIcon icon={faChevronLeft} />
          <span>{text}</span>
        </button>
  );
}
