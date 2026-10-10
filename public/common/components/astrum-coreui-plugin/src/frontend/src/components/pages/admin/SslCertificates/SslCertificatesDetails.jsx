
import React from 'react';

import {
  AsReadOnly,
  AsH2,
  AsGrid,
  AsDivider,
  AsWarningAlert,
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
    <AsReadOnly>
      <ReadOnlyField label={LABELS.FINGERPRINT.LABEL} value={fingerprint}/>
      <ReadOnlyField label={LABELS.VALID_UNTIL.LABEL} value={expiredDate}/>
      <ReadOnlyField label={LABELS.ISSUED_ON.LABEL} value={issuedDate}/>
    </AsReadOnly>
    <AsDivider />
    <AsGrid.Row>
      <AsGrid.Column>
        <AsH2>{LABELS.SECTIONS.SUBJECT}</AsH2>
        <AsReadOnly>
          <ReadOnlyField label={LABELS.COMMON_NAME.LABEL} value={subjectCommonName}/>
          <ReadOnlyField label={LABELS.ORGANIZATION.LABEL} value={subjectOrganization}/>
          <ReadOnlyField label={LABELS.UNIT.LABEL} value={subjectOrganizationalUnit}/>
        </AsReadOnly>
      </AsGrid.Column>
      <AsGrid.Column>
        <AsH2>{LABELS.SECTIONS.ISSUER}</AsH2>
        <AsReadOnly>
          <ReadOnlyField label={LABELS.COMMON_NAME.LABEL} value={issuerCommonName}/>
          <ReadOnlyField label={LABELS.ORGANIZATION.LABEL} value={issuerOrganization}/>
          <ReadOnlyField label={LABELS.UNIT.LABEL} value={issuerOrganizationalUnit}/>
        </AsReadOnly>
      </AsGrid.Column>
    </AsGrid.Row>
    <AsWarningAlert className="ssl-certificate-alert">{LABELS.WARNING}</AsWarningAlert>
  </>;
}
