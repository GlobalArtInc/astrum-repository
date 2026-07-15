

import React from "react";

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from "../../../../constants/UIStrings";

export default function SearchConanExt() {
  const CRITERIA = UIStrings.SEARCH.CONAN.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.CONAN.MENU.text}
      icon={UIStrings.SEARCH.CONAN.MENU.icon}
      criterias={[
        {
          id: 'attributes.conan.baseVersion',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.BASE_VERSION,
            width: 250
          }
        },
        {
          id: 'attributes.conan.channel',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.CHANNEL,
            width: 250
          }
        },
        {
          id: 'attributes.conan.revision',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.RECIPE_REVISION,
            width: 250
          }
        },
        {
          id: 'assets.attributes.conan.packageId',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.PACKAGE_ID,
            width: 250
          }
        },
        {
          id: 'assets.attributes.conan.packageRevision',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.PACKAGE_REVISION,
            width: 250
          }
        },
        {
          id: 'attributes.conan.baseVersion.strict',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.BASE_VERSION_STRICT,
            width: 250
          }
        },
        {
          id: 'attributes.conan.revision.latest',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.RECIPE_REVISION_LATEST,
            width: 250
          }
        },
        {
          id: 'attributes.conan.settings.arch',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.ARCH,
            width: 250
          }
        },
        {
          id: 'attributes.conan.settings.os',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.OS,
            width: 250
          }
        },
        {
          id: 'attributes.conan.settings.compiler',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.COMPILER,
            width: 250
          }
        },
        {
          id: 'attributes.conan.settings.compiler.version',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.COMPILER_VERSION,
            width: 250
          }
        },
        {
          id: 'attributes.conan.settings.compiler.runtime',
          group: CRITERIA.GROUP,
          config: {
            format: 'conan',
            fieldLabel: CRITERIA.FIELD_LABEL.COMPILER_RUNTIME,
            width: 250
          }
        }
      ]}
      filter={{
        id: 'conan',
        name: 'Conan',
        text: UIStrings.SEARCH.CONAN.MENU.text,
        description: UIStrings.SEARCH.CONAN.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'conan', hidden: true},
          {id: 'name.raw'},
          {id: 'attributes.conan.baseVersion'},
          {id: 'attributes.conan.baseVersion.strict'},
          {id: 'attributes.conan.channel'},
          {id: 'attributes.conan.revision'},
          {id: 'assets.attributes.conan.packageId'},
          {id: 'assets.attributes.conan.packageRevision'}
        ]
      }}
    />
  );
}
