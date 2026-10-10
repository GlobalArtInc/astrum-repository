
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsCheckbox, AsFieldset} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function RewritePackageUrlsConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {format} = currentParent.context.data;

  return (
    <AsFieldset label={EDITOR.REWRITE_URLS_LABEL} className="as-form-group-rewrite-urls">
      <AsCheckbox
        {...FormUtils.checkboxProps(`${format}.rewritePackageUrls`, currentParent)}
        onChange={FormUtils.handleUpdate(`${format}.rewritePackageUrls`, sendParent)}
      >
        {EDITOR.REWRITE_URLS_DESCR}
      </AsCheckbox>
    </AsFieldset>
  );
}
