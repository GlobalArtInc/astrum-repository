

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchPypiExt() {
  const CRITERIA = UIStrings.SEARCH.PYPI.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.PYPI.MENU.text}
      icon={UIStrings.SEARCH.PYPI.MENU.icon}
      criterias={[
        {
          id: 'assets.attributes.pypi.classifiers',
          group: CRITERIA.GROUP,
          config: {
            format: 'pypi',
            fieldLabel: CRITERIA.FIELD_LABEL.CLASSIFIERS,
            width: 250
          }
        },
        {
          id: 'assets.attributes.pypi.description',
          group: CRITERIA.GROUP,
          config: {
            format: 'pypi',
            fieldLabel: CRITERIA.FIELD_LABEL.DESCRIPTION,
            width: 250
          }
        },
        {
          id: 'assets.attributes.pypi.keywords',
          group: CRITERIA.GROUP,
          config: {
            format: 'pypi',
            fieldLabel: CRITERIA.FIELD_LABEL.KEYWORDS,
            width: 250
          }
        },
        {
          id: 'assets.attributes.pypi.summary',
          group: CRITERIA.GROUP,
          config: {
            format: 'pypi',
            fieldLabel: CRITERIA.FIELD_LABEL.SUMMARY,
            width: 250
          }
        }
      ]}
      filter={{
        id: 'pypi',
        name: 'pypi',
        text: UIStrings.SEARCH.PYPI.MENU.text,
        description: UIStrings.SEARCH.PYPI.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'pypi', hidden: true },
          { id: 'assets.attributes.pypi.classifiers' },
          { id: 'assets.attributes.pypi.description' },
          { id: 'assets.attributes.pypi.keywords' },
          { id: 'assets.attributes.pypi.summary' }
        ]
      }}
    />
  );
}
