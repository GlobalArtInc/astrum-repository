

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchRubygemsExt() {
  const CRITERIA = UIStrings.SEARCH.RUBYGEMS.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.RUBYGEMS.MENU.text}
      icon={UIStrings.SEARCH.RUBYGEMS.MENU.icon}
      criterias={[
        {
          id: 'assets.attributes.rubygems.platform',
          group: CRITERIA.GROUP,
          config: {
            format: 'rubygems',
            fieldLabel: CRITERIA.FIELD_LABEL.PLATFORM,
            width: 250
          }
        },
        {
          id: 'assets.attributes.rubygems.summary',
          group: CRITERIA.GROUP,
          config: {
            format: 'rubygems',
            fieldLabel: CRITERIA.FIELD_LABEL.SUMMARY,
            width: 250
          }
        },
        {
          id: 'assets.attributes.rubygems.description',
          group: CRITERIA.GROUP,
          config: {
            format: 'rubygems',
            fieldLabel: CRITERIA.FIELD_LABEL.DESCRIPTION,
            width: 250
          }
        }
      ]}
      filter={{
        id: 'rubygems',
        name: 'Rubygems',
        text: UIStrings.SEARCH.RUBYGEMS.MENU.text,
        description: UIStrings.SEARCH.RUBYGEMS.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'rubygems', hidden: true },
          { id: 'name.raw' },
          { id: 'version' },
          { id: 'assets.attributes.rubygems.platform' },
          { id: 'assets.attributes.rubygems.summary' },
          { id: 'assets.attributes.rubygems.description' }
        ]
      }}
    />
  );
}
