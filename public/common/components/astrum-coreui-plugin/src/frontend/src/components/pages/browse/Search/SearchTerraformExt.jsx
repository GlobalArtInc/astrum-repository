

import React from 'react';

import SearchFeatureExt from './SearchFeatureExt';
import UIStrings from '../../../../constants/UIStrings';

export default function SearchTerraformExt() {
    const CRITERIA = UIStrings.SEARCH.TERRAFORM.CRITERIA;

  return (
    <SearchFeatureExt
      title={UIStrings.SEARCH.TERRAFORM.MENU.text}
      icon={UIStrings.SEARCH.TERRAFORM.MENU.icon}
      criterias={[
          {
              id: 'assets.attributes.terraform.provider',
              group: CRITERIA.GROUP,
              config: {
                  format: 'terraform',
                  fieldLabel: CRITERIA.FIELD_LABEL.PROVIDER,
                  width: 250
              }
          },
          {
              id: 'assets.attributes.terraform.namespace',
              group: CRITERIA.GROUP,
              config: {
                  format: 'terraform',
                  fieldLabel: CRITERIA.FIELD_LABEL.NAMESPACE,
                  width: 250
              }
          },
          {
              id: 'assets.attributes.terraform.name',
              group: CRITERIA.GROUP,
              config: {
                  format: 'terraform',
                  fieldLabel: CRITERIA.FIELD_LABEL.NAME,
                  width: 250
              }
          }
      ]}
      filter={{
        id: 'terraform',
        name: 'Terraform',
        text: UIStrings.SEARCH.TERRAFORM.MENU.text,
        description: UIStrings.SEARCH.TERRAFORM.MENU.description,
        readOnly: true,
        criterias: [
          { id: 'format', value: 'terraform', hidden: true },
            { id: 'assets.attributes.terraform.provider' },
            { id: 'assets.attributes.terraform.namespace' },
            { id: 'assets.attributes.terraform.name' }
        ]
      }}
    />
  );
}
