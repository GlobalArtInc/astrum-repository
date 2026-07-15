

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchNpmExt() {
  const CRITERIA = UIStrings.SEARCH.NPM.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.NPM.MENU.text}
      icon={UIStrings.SEARCH.NPM.MENU.icon}
      criterias={[
        {
          id: 'group',
          group: CRITERIA.GROUP,
          config: {
            format: 'npm',
            fieldLabel: CRITERIA.FIELD_LABEL.SCOPE,
            width: 250
          }
        },
        {
          id: 'assets.attributes.npm.author',
          group: CRITERIA.GROUP,
          config: {
            format: 'npm',
            fieldLabel: CRITERIA.FIELD_LABEL.AUTHOR,
            width: 250
          }
        },
        {
          id: 'assets.attributes.npm.description',
          group: CRITERIA.GROUP,
          config: {
            format: 'npm',
            fieldLabel: CRITERIA.FIELD_LABEL.DESCRIPTION,
            width: 250
          }
        },
        {
          id: 'assets.attributes.npm.keywords',
          group: CRITERIA.GROUP,
          config: {
            format: 'npm',
            fieldLabel: CRITERIA.FIELD_LABEL.KEYWORDS,
            width: 250
          }
        },
        {
          id: 'assets.attributes.npm.license',
          group: CRITERIA.GROUP,
          config: {
            format: 'npm',
            fieldLabel: CRITERIA.FIELD_LABEL.LICENSE,
            width: 250
          }
        }
      ]}
      filter={{
        id: 'npm',
        name: 'npm',
        text: UIStrings.SEARCH.NPM.MENU.text,
        description: UIStrings.SEARCH.NPM.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'npm', hidden: true },
          { id: 'group' },
          { id: 'name.raw' },
          { id: 'version' },
          { id: 'assets.attributes.npm.author' },
          { id: 'assets.attributes.npm.description' },
          { id: 'assets.attributes.npm.keywords' },
          { id: 'assets.attributes.npm.license' }
        ]
      }}
    />
  );
}
