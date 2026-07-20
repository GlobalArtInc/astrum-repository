
import React from 'react';
import {useMachine} from '@xstate/react';

import {ListMachineUtils, Page} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxFilterInput,
  NxTable,
  NxTableBody,
  NxTableCell,
  NxTableHead,
  NxTableRow,
  NxTooltip
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import {faIdCardAlt} from '@fortawesome/free-solid-svg-icons';

import Machine from './SslCertificatesListMachine';
import UIStrings from '../../../../constants/UIStrings';
import {canCreateCertificate} from './SslCertificatesHelper';

import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';
import { useRouter } from '@uirouter/react';

const ADMIN = ROUTE_NAMES.ADMIN;

const {
  LIST: {COLUMNS},
  LIST: LABELS
} = UIStrings.SSL_CERTIFICATES;

export default function SslCertificatesList() {
  const router = useRouter();
  const onEdit = (itemId) => router.stateService.go(ADMIN.SECURITY.SSLCERTIFICATES.EDIT, {itemId});
  const onCreate = () => router.stateService.go(ADMIN.SECURITY.SSLCERTIFICATES.CREATE);
  const [current, send] = useMachine(Machine, {devTools: true});
  const isLoading = current.matches('loading');
  const {data, error, filter: filterText} = current.context;

  const subjectCommonNameSortDir = ListMachineUtils.getSortDirection(
    'subjectCommonName',
    current.context
  );
  const subjectOrganizationSortDir = ListMachineUtils.getSortDirection(
    'subjectOrganization',
    current.context
  );
  const issuerOrganizationSortDir = ListMachineUtils.getSortDirection(
    'issuerOrganization',
    current.context
  );
  const fingerprintSortDir = ListMachineUtils.getSortDirection('fingerprint', current.context);

  const sortBySubjectCommonName = () => send({type: 'SORT_BY_SUBJECT_COMMON_NAME'});
  const sortBySubjectOrganization = () => send({type: 'SORT_BY_SUBJECT_ORGANIZATION'});
  const sortByIssuerOrganization = () => send({type: 'SORT_BY_ISSUER_ORGANIZATION'});
  const sortByFingerprint = () => send({type: 'SORT_BY_FINGERPRINT'});

  const filter = (value) => send({type: 'FILTER', filter: value});

  const canCreate = canCreateCertificate();

  return (
    <Page className="as-ssl-certificates">
      <PageHeader>
        <PageTitle
          icon={faIdCardAlt}
          text={UIStrings.SSL_CERTIFICATES.MENU.text}
          description={UIStrings.SSL_CERTIFICATES.MENU.description}
        />
        <PageActions>
          <NxTooltip title={!canCreate && UIStrings.PERMISSION_ERROR} placement="bottom">
            <NxButton
              type="button"
              variant="primary"
              className={!canCreate && 'disabled'}
              onClick={() => canCreate && onCreate()}
            >
              {LABELS.CREATE_BUTTON}
            </NxButton>
          </NxTooltip>
        </PageActions>
      </PageHeader>
      <ContentBody className="as-ssl-certificates-list">
        <Section>
          <SectionToolbar>
            <div className="as-spacer" />
            <NxFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.FILTER}
            />
          </SectionToolbar>
          <NxTable>
            <NxTableHead>
              <NxTableRow>
                <NxTableCell
                  onClick={sortBySubjectCommonName}
                  isSortable
                  sortDir={subjectCommonNameSortDir}
                >
                  {COLUMNS.NAME}
                </NxTableCell>
                <NxTableCell
                  onClick={sortBySubjectOrganization}
                  isSortable
                  sortDir={subjectOrganizationSortDir}
                >
                  {COLUMNS.ISSUED_TO}
                </NxTableCell>
                <NxTableCell
                  onClick={sortByIssuerOrganization}
                  isSortable
                  sortDir={issuerOrganizationSortDir}
                >
                  {COLUMNS.ISSUED_BY}
                </NxTableCell>
                <NxTableCell onClick={sortByFingerprint} isSortable sortDir={fingerprintSortDir}>
                  {COLUMNS.FINGERPRINT}
                </NxTableCell>
                <NxTableCell chevron />
              </NxTableRow>
            </NxTableHead>
            <NxTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
              {data.map(
                ({id, subjectCommonName, subjectOrganization, issuerOrganization, fingerprint}) => (
                  <NxTableRow key={id} onClick={() => onEdit(encodeURIComponent(id))} isClickable>
                    <NxTableCell>{subjectCommonName}</NxTableCell>
                    <NxTableCell>{subjectOrganization}</NxTableCell>
                    <NxTableCell>{issuerOrganization}</NxTableCell>
                    <NxTableCell>{fingerprint}</NxTableCell>
                    <NxTableCell chevron />
                  </NxTableRow>
                )
              )}
            </NxTableBody>
          </NxTable>
        </Section>
        <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT} />
      </ContentBody>
    </Page>
  );
}
