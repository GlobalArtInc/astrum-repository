
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsFormSelect} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function RepodataDepthConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {format} = currentParent.context.data;

  const repodataDepthOptions = [0, 1, 2, 3, 4, 5];

  return (
    <AsFormGroup
      label={EDITOR.REPODATA_DEPTH_LABEL}
      className="as-form-group-repodata-depth"
      sublabel={EDITOR.REPODATA_DEPTH_SUBLABEL}
      isRequired
    >
      <AsFormSelect
        {...FormUtils.selectProps(`${format}.repodataDepth`, currentParent)}
        onChange={FormUtils.handleUpdate(`${format}.repodataDepth`, sendParent)}
      >
        {repodataDepthOptions?.map((val) => (
          <option key={val} value={val}>
            {val}
          </option>
        ))}
      </AsFormSelect>
    </AsFormGroup>
  );
}
