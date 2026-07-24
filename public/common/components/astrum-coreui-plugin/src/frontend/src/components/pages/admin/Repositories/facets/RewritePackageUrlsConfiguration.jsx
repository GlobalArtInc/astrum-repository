
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxCheckbox, NxFieldset} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function RewritePackageUrlsConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {format} = currentParent.context.data;

  return (
    <NxFieldset label={EDITOR.REWRITE_URLS_LABEL} className="as-form-group-rewrite-urls">
      <NxCheckbox
        {...FormUtils.checkboxProps(`${format}.rewritePackageUrls`, currentParent)}
        onChange={FormUtils.handleUpdate(`${format}.rewritePackageUrls`, sendParent)}
      >
        {EDITOR.REWRITE_URLS_DESCR}
      </NxCheckbox>
    </NxFieldset>
  );
}
