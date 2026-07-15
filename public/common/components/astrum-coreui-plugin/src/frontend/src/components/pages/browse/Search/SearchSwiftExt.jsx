

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

/**
 * Search component for Swift package repositories.
 * Provides format-specific search criteria including name, scope, version,
 * type, package identifier, and manifest filename.
 */
export default function SearchSwiftExt() {
  const CRITERIA = UIStrings.SEARCH.SWIFT.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.SWIFT.MENU.text}
      icon={UIStrings.SEARCH.SWIFT.MENU.icon}
      criterias={[
        {
          id: 'attributes.swift.scope',
          group: CRITERIA.GROUP,
          config: {
            format: 'swift',
            fieldLabel: CRITERIA.FIELD_LABEL.SCOPE,
            width: 250
          }
        },
      ]}
      filter={{
        id: 'swift',
        name: 'swift',
        text: UIStrings.SEARCH.SWIFT.MENU.text,
        description: UIStrings.SEARCH.SWIFT.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'swift', hidden: true },
          { id: 'name.raw' },
          { id: 'attributes.swift.scope' },
          { id: 'version' }
        ]
      }}
    />
  );
}
