

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchYumExt() {
  const CRITERIA = UIStrings.SEARCH.YUM.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.YUM.MENU.text}
      icon={UIStrings.SEARCH.YUM.MENU.icon}
      criterias={[
        {
          id: 'assets.attributes.yum.architecture',
          group: CRITERIA.GROUP,
          config: {
            format: 'yum',
            fieldLabel: CRITERIA.FIELD_LABEL.ARCHITECTURE,
            width: 250
          }
        },
        {
          id: 'attributes.yum.name',
          group: CRITERIA.GROUP,
          config: {
            format: 'yum',
            fieldLabel: CRITERIA.FIELD_LABEL.NAME,
            width: 250
          }
        }
      ]}
      filter={{
        id: 'yum',
        name: 'Yum',
        text: UIStrings.SEARCH.YUM.MENU.text,
        description: UIStrings.SEARCH.YUM.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'yum', hidden: true },
          { id: 'attributes.yum.name' },
          { id: 'version' },
          { id: 'assets.attributes.yum.architecture' }
        ]
      }}
    />
  );
}
