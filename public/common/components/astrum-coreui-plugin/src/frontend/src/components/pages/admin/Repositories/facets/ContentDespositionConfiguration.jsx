
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsFormSelect} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

const contentDispositionOptions = [
  {key: 'Inline', value: 'INLINE'},
  {key: 'Attachment', value: 'ATTACHMENT'}
];

export default function ContentDespositionConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  let {format} = currentParent.context.data;
  format = format === 'maven2' ? 'maven' : format;

  return (
    <AsFormGroup
      label={EDITOR.CONTENT_DISPOSITION_LABEL}
      className="as-form-group-content-disposition"
      sublabel={EDITOR.CONTENT_DISPOSITION_SUBLABEL}
      isRequired
    >
      <AsFormSelect
        {...FormUtils.selectProps(`${format}.contentDisposition`, currentParent)}
        onChange={FormUtils.handleUpdate(`${format}.contentDisposition`, sendParent)}
      >
        {contentDispositionOptions?.map(({key, value}) => (
          <option key={key} value={value}>
            {key}
          </option>
        ))}
      </AsFormSelect>
    </AsFormGroup>
  );
}
