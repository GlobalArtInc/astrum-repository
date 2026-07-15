

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchAnsiblegalaxyExt() {
    const CRITERIA = UIStrings.SEARCH.ANSIBLEGALAXY.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.ANSIBLEGALAXY.MENU.text}
      icon={UIStrings.SEARCH.ANSIBLEGALAXY.MENU.icon}
      criterias={[
          {
              id: 'assets.attributes.ansiblegalaxy.namespace',
              group: CRITERIA.GROUP,
              config: {
                  format: 'ansiblegalaxy',
                  fieldLabel: CRITERIA.FIELD_LABEL.NAMESPACE,
                  width: 250
              }
          },
          {
              id: 'assets.attributes.ansiblegalaxy.name',
              group: CRITERIA.GROUP,
              config: {
                  format: 'ansiblegalaxy',
                  fieldLabel: CRITERIA.FIELD_LABEL.NAME,
                  width: 250
              }
          },
          {
              id: 'assets.attributes.ansiblegalaxy.version',
              group: CRITERIA.GROUP,
              config: {
                  format: 'ansiblegalaxy',
                  fieldLabel: CRITERIA.FIELD_LABEL.VERSION,
                  width: 250
              }
          }
      ]}
      filter={{
        id: 'ansiblegalaxy',
        name: 'Ansible Galaxy',
        text: UIStrings.SEARCH.ANSIBLEGALAXY.MENU.text,
        description: UIStrings.SEARCH.ANSIBLEGALAXY.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'ansiblegalaxy', hidden: true },
            { id: 'assets.attributes.ansiblegalaxy.namespace' },
            { id: 'assets.attributes.ansiblegalaxy.name' },
            { id: 'assets.attributes.ansiblegalaxy.version' }
        ]
      }}
    />
  );
}
