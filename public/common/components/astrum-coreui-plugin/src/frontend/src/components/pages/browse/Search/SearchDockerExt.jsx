

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchDockerExt() {
  const CRITERIA = UIStrings.SEARCH.DOCKER.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.DOCKER.MENU.text}
      icon={UIStrings.SEARCH.DOCKER.MENU.icon}
      criterias={[
        {
          id: 'attributes.docker.imageName',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.IMAGE_NAME,
            width: 300
          }
        },
        {
          id: 'attributes.docker.imageTag',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.IMAGE_TAG,
          }
        },
        {
          id: 'attributes.docker.layerAncestry',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.LAYER_ID,
            width: 500
          }
        },
        {
          id: 'assets.attributes.docker.content_digest',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.CONTENT_DIGEST,
            width: 500
          }
        },
        {
          id: 'attributes.docker.os',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.OS
          }
        },
        {
          id: 'attributes.docker.architecture',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.ARCHITECTURE
          }
        },
        {
          id: 'attributes.docker.labels',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.LABELS
          }
        },
        {
          id: 'attributes.docker.author',
          group: CRITERIA.GROUP,
          config: {
            format: 'docker',
            fieldLabel: CRITERIA.FIELD_LABEL.AUTHOR
          }
        }
      ]}
      filter={{
        id: 'docker',
        name: 'Docker',
        text: UIStrings.SEARCH.DOCKER.MENU.text,
        description: UIStrings.SEARCH.DOCKER.MENU.description,
        readOnly: true,
        criterias: [
          {id: 'format', value: 'docker', hidden: true},
          {id: 'attributes.docker.imageName'},
          {id: 'attributes.docker.imageTag'},
          {id: 'attributes.docker.layerAncestry'},
          {id: 'assets.attributes.docker.content_digest'},
          {id: 'attributes.docker.os'},
          {id: 'attributes.docker.architecture'},
          {id: 'attributes.docker.labels'},
          {id: 'attributes.docker.author'}
        ]
      }}
    />
  );
}
