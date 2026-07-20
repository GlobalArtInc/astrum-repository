
import React from 'react';

import {
  NxReadOnly,
  NxH2,
  NxGrid,
  NxDivider,
  NxWarningAlert,
} from '@globalart/react-shared-components';
import {
  DateUtils,
  ReadOnlyField,
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {
  SSL_CERTIFICATES: {FORM: LABELS},
} = UIStrings;

export default function SslCertificatesDetails({data}) {
  const {
    issuedOn,
    expiresOn,
    subjectCommonName,
    subjectOrganization,
    subjectOrganizationalUnit,
    issuerCommonName,
    issuerOrganization,
    issuerOrganizationalUnit,
    fingerprint,
  } = data;

  const issuedDate = issuedOn ? DateUtils.timestampToString(issuedOn) : '';
  const expiredDate = expiresOn ? DateUtils.timestampToString(expiresOn) : '';

  return <>
    <NxReadOnly>
      <ReadOnlyField label={LABELS.FINGERPRINT.LABEL} value={fingerprint}/>
      <ReadOnlyField label={LABELS.VALID_UNTIL.LABEL} value={expiredDate}/>
      <ReadOnlyField label={LABELS.ISSUED_ON.LABEL} value={issuedDate}/>
    </NxReadOnly>
    <NxDivider />
    <NxGrid.Row>
      <NxGrid.Column>
        <NxH2>{LABELS.SECTIONS.SUBJECT}</NxH2>
        <NxReadOnly>
          <ReadOnlyField label={LABELS.COMMON_NAME.LABEL} value={subjectCommonName}/>
          <ReadOnlyField label={LABELS.ORGANIZATION.LABEL} value={subjectOrganization}/>
          <ReadOnlyField label={LABELS.UNIT.LABEL} value={subjectOrganizationalUnit}/>
        </NxReadOnly>
      </NxGrid.Column>
      <NxGrid.Column>
        <NxH2>{LABELS.SECTIONS.ISSUER}</NxH2>
        <NxReadOnly>
          <ReadOnlyField label={LABELS.COMMON_NAME.LABEL} value={issuerCommonName}/>
          <ReadOnlyField label={LABELS.ORGANIZATION.LABEL} value={issuerOrganization}/>
          <ReadOnlyField label={LABELS.UNIT.LABEL} value={issuerOrganizationalUnit}/>
        </NxReadOnly>
      </NxGrid.Column>
    </NxGrid.Row>
    <NxWarningAlert className="ssl-certificate-alert">{LABELS.WARNING}</NxWarningAlert>
  </>;
}
