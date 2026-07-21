
import React from 'react';

import {ReadOnlyField, FormUtils} from '@globalart/astrum-ui-plugin';

import {NxInfoAlert} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {
  USER_TOKEN_CONFIGURATION: {USER_TOKENS_CHECKBOX, USER_TOKEN_EXPIRY, EXPIRATION_CHECKBOX, REPOSITORY_AUTHENTICATION_CHECKBOX},
  SETTINGS: {
    READ_ONLY: {WARNING}
  }
} = UIStrings;

export default function UserTokensReadOnly({data}) {
  const {enabled, expirationEnabled, expirationDays, protectContent} = data;

  return (
    <>
      <NxInfoAlert>{WARNING}</NxInfoAlert>
      <ReadOnlyField
        label={USER_TOKENS_CHECKBOX.LABEL}
        value={FormUtils.readOnlyCheckboxValueLabel(enabled)}
      />
      <ReadOnlyField
        label={REPOSITORY_AUTHENTICATION_CHECKBOX.LABEL}
        value={FormUtils.readOnlyCheckboxValueLabel(protectContent)}
      />
      <ReadOnlyField
          label={EXPIRATION_CHECKBOX.LABEL}
          value={FormUtils.readOnlyCheckboxValueLabel(expirationEnabled)}
      />
      <ReadOnlyField
        label={USER_TOKEN_EXPIRY.LABEL}
        value={`${expirationDays} Day${expirationDays > 1 ? 's' : ''}`}
      />
    </>
  );
}
