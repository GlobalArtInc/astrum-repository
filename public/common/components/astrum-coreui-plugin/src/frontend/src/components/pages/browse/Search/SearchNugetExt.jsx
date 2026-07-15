

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchNugetExt() {
  const CRITERIA = UIStrings.SEARCH.NUGET.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.NUGET.MENU.text}
      icon={UIStrings.SEARCH.NUGET.MENU.icon}
      criterias={[
        {
          id: 'attributes.nuget.id',
          group: CRITERIA.GROUP,
          config: {
            format: 'nuget',
            fieldLabel: CRITERIA.FIELD_LABEL.ID,
            width: 300
          }
        },
        {
          id: 'assets.attributes.nuget.tags',
          group: CRITERIA.GROUP,
          config: {
            format: 'nuget',
            fieldLabel: CRITERIA.FIELD_LABEL.TAGS,
            width: 300
          }
        }
      ]}
      filter={{
        id: 'nuget',
        name: 'NuGet',
        text: UIStrings.SEARCH.NUGET.MENU.text,
        description: UIStrings.SEARCH.NUGET.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'nuget', hidden: true },
          { id: 'attributes.nuget.id' },
          { id: 'assets.attributes.nuget.tags' }
        ]
      }}
    />
  );
}
