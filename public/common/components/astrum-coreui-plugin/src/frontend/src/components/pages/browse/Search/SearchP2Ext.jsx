

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchP2Ext() {
  const CRITERIA = UIStrings.SEARCH.P2.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.P2.MENU.text}
      icon={UIStrings.SEARCH.P2.MENU.icon}
      criterias={[
        {
          id: 'attributes.p2.pluginName',
          group: CRITERIA.GROUP,
          config: {
            format: 'p2',
            fieldLabel: CRITERIA.FIELD_LABEL.PLUGIN_NAME,
            width: 250
          }
        }
      ]}
      filter={{
        id: 'p2',
        name: 'p2',
        text: UIStrings.SEARCH.P2.MENU.text,
        description: UIStrings.SEARCH.P2.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'p2', hidden: true},
          {id: 'attributes.p2.pluginName'},
          {id: 'name.raw'}
        ]
      }}
    />
  );
}
