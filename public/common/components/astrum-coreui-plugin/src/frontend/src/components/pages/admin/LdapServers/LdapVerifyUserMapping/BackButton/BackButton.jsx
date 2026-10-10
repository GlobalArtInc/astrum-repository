
import React from 'react';
import {AsFontAwesomeIcon} from '@globalart/react-shared-components';
import {faChevronLeft} from '@fortawesome/free-solid-svg-icons';
import './BackButton.scss';

export default function BackButton({onClick, text}) {
  return (
        <button className="astrum-back-button as-text-link" onClick={onClick}>
          <AsFontAwesomeIcon icon={faChevronLeft} />
          <span>{text}</span>
        </button>
  );
}
