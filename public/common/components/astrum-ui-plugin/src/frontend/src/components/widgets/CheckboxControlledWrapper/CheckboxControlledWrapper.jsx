

import React from 'react';

import './CheckboxControlledWrapper.scss';
import {NxCheckbox} from "@globalart/react-shared-components";

/**
 * @since 3.29
 */
export default function CheckboxControlledWrapper({id, children, onChange, ...attrs}) {
  function handleChange(value) {
    if (onChange) {
      onChange(value);
    }
  }

  return <div className='checkbox-controlled-wrapper' id={id}>
    <div className='checkbox-control'>
      <NxCheckbox {...attrs} onChange={handleChange}/>
    </div>
    <div className='checkbox-children'>
      {children}
    </div>
  </div>;
};
