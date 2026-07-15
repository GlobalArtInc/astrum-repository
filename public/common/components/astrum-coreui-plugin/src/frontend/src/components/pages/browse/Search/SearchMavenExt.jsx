

import React from "react";

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from "../../../../constants/UIStrings";

export default function SearchMavenExt() {
  const CRITERIA = UIStrings.SEARCH.MAVEN.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.MAVEN.MENU.text}
      icon={UIStrings.SEARCH.MAVEN.MENU.icon}
      criterias={[
        {
          id: 'attributes.maven2.groupId',
          group: CRITERIA.GROUP,
          config: {
            format: 'maven2',
            fieldLabel: CRITERIA.FIELD_LABEL.GROUP_ID,
            width: 250
          }
        },
        {
          id: 'attributes.maven2.artifactId',
          group: CRITERIA.GROUP,
          config: {
            format: 'maven2',
            fieldLabel: CRITERIA.FIELD_LABEL.ARTIFACT_ID,
            width: 250
          }
        },
        {
          id: 'attributes.maven2.baseVersion',
          group: CRITERIA.GROUP,
          config: {
            format: 'maven2',
            fieldLabel: CRITERIA.FIELD_LABEL.BASE_VERSION,
            width: 250
          }
        },
        {
          id: 'assets.attributes.maven2.classifier',
          group: CRITERIA.GROUP,
          config: {
            format: 'maven2',
            fieldLabel: CRITERIA.FIELD_LABEL.CLASSIFIER,
          }
        },
        {
          id: 'assets.attributes.maven2.extension',
          group: CRITERIA.GROUP,
          config: {
            format: 'maven2',
            fieldLabel: CRITERIA.FIELD_LABEL.EXTENSION,
          }
        }
      ]}
      filter={{
        id: "maven2",
        name: "Maven",
        text: UIStrings.SEARCH.MAVEN.MENU.text,
        description: UIStrings.SEARCH.MAVEN.MENU.description,
        readOnly: true,
        criterias: [
          { id: "format", value: "maven2", hidden: true },
          { id: "attributes.maven2.groupId" },
          { id: "attributes.maven2.artifactId" },
          { id: "version" },
          { id: "attributes.maven2.baseVersion" },
          { id: "assets.attributes.maven2.classifier" },
          { id: "assets.attributes.maven2.extension" },
        ],
      }}
    />
  );
}
