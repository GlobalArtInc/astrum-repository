
import React, {useEffect, useState} from 'react';

import {ExtJS, ListMachineUtils, copyToClipboard} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxButtonBar,
  NxFilterInput,
  NxFontAwesomeIcon,
  NxTable,
  NxTile
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import {
  isIqServerEnabled,
  canReadFirewallStatus,
  canUpdateHealthCheck,
  hasFirewall
} from '../../admin/Repositories/IQServerColumns/IQServerHelpers';

import HealthCheckCell from '../../admin/Repositories/IQServerColumns/HealthCheckCell';
import IqPolicyViolationsCell from '../../admin/Repositories/IQServerColumns/IqPolicyViolationsCell';
import RepositoryStatus from '../../admin/Repositories/RepositoryStatus';
import {faCopy, faDatabase} from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../constants/UIStrings';
import './Browse.scss';

import { useRepositoriesService } from '../../admin/Repositories/RepositoriesContextProvider';

const {BROWSE} = UIStrings;
const {
  COLUMNS,
  FILTER_PLACEHOLDER,
  EMPTY_MESSAGE,
  COPY_URL_TITLE,
  URL_COPIED_MESSAGE,
  URL_COPY_ERROR_MESSAGE
} = BROWSE.LIST;

export default function BrowseList({onEdit, copyUrl = doCopyUrl}) {
  const [state, send] = useRepositoriesService();
  // clmState exists solely to trigger re-renders when IQ/Firewall config changes.
  // The actual state value is not used directly; we call helper functions instead.
  const [clmState, setClmState] = useState(ExtJS.state().getValue('clm', {}));

  useEffect(() => {
    send({type: 'LOAD'});
  }, []);

  // Listen for clm state changes to re-render when IQ/Firewall config changes
  useEffect(() => {
    const state = ExtJS.state();
    // Only set up listeners if ExtJS.state() supports events (not in test environment)
    if (state && typeof state.on === 'function') {
      const handleClmChange = () => {
        setClmState(state.getValue('clm', {}));
      };

      state.on('clmchanged', handleClmChange);

      return () => {
        state.un('clmchanged', handleClmChange);
      };
    }
  }, []);

  const {data, error, filter: filterText} = state.context;
  const isLoading = state.matches('loading');

  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const typeSortDir = ListMachineUtils.getSortDirection('type', state.context);
  const formatSortDir = ListMachineUtils.getSortDirection('format', state.context);
  const statusSortDir = ListMachineUtils.getSortDirection('status', state.context);
  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByType = () => send({type: 'SORT_BY_TYPE'});
  const sortByFormat = () => send({type: 'SORT_BY_FORMAT'});
  const sortByStatus = () => send({type: 'SORT_BY_STATUS'});

  const filter = (value) => send({type: 'FILTER', filter: value});

  // Show Health Check column only if user has permission AND NOT (IQ enabled AND Firewall enabled)
  // Health Check should show when: IQ is disabled OR IQ doesn't have Firewall
  // clmState is used to trigger re-renders when IQ/Firewall config changes via 'clmchanged' event
  // We call helper functions directly for the logic (so tests work with mocked helpers)
  const showHealthCheckColumn = canUpdateHealthCheck() && !(isIqServerEnabled() && hasFirewall());
  const showIqPolicyViolationsColumn = isIqServerEnabled() && canReadFirewallStatus();

  return <div className="as-browse">
    <PageHeader>
      <PageTitle icon={faDatabase} {...BROWSE.MENU}/>
    </PageHeader>
    <ContentBody className="as-browse-list">
      <NxTile>
        <NxTile.Header>
          <NxTile.HeaderActions>
            <NxFilterInput
                    id="filter"
                    onChange={filter}
                    value={filterText}
                    placeholder={FILTER_PLACEHOLDER}/>
          </NxTile.HeaderActions>
        </NxTile.Header>
        <NxTile.Content>
          <NxTable>
            <NxTable.Head>
              <NxTable.Row>
                <NxTable.Cell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</NxTable.Cell>
                <NxTable.Cell onClick={sortByType} isSortable sortDir={typeSortDir}>{COLUMNS.TYPE}</NxTable.Cell>
                <NxTable.Cell onClick={sortByFormat} isSortable sortDir={formatSortDir}>{COLUMNS.FORMAT}</NxTable.Cell>
                <NxTable.Cell onClick={sortByStatus} isSortable sortDir={statusSortDir}>{COLUMNS.STATUS}</NxTable.Cell>
                <NxTable.Cell>{COLUMNS.URL}</NxTable.Cell>
                {showHealthCheckColumn && (
                  <NxTable.Cell className="as-table-cell-centered">
                    {COLUMNS.HEALTH_CHECK}
                  </NxTable.Cell>
                )}
                {showIqPolicyViolationsColumn && (
                  <NxTable.Cell className="as-table-cell-centered">
                    {COLUMNS.IQ_POLICY_VIOLATIONS}
                  </NxTable.Cell>
                )}
                <NxTable.Cell chevron />
              </NxTable.Row>
            </NxTable.Head>
            <NxTable.Body isLoading={isLoading} error={error} emptyMessage={EMPTY_MESSAGE}>
              {data.map(({name, type, format, status, url}) => (
                <NxTable.Row key={name} onClick={() => onEdit(name)} isClickable>
                  <NxTable.Cell>{name}</NxTable.Cell>
                  <NxTable.Cell>{type}</NxTable.Cell>
                  <NxTable.Cell>{format}</NxTable.Cell>
                  <NxTable.Cell>
                    <RepositoryStatus status={status} />
                  </NxTable.Cell>
                  <NxTable.Cell>
                    <NxButtonBar>
                      <NxButton
                        variant="icon-only"
                        onClick={(e) => copyUrl(e, url)}
                        title={COPY_URL_TITLE}
                        aria-label={COPY_URL_TITLE}
                      >
                        <NxFontAwesomeIcon icon={faCopy} />
                      </NxButton>
                    </NxButtonBar>
                  </NxTable.Cell>
                  {showHealthCheckColumn && (
                    <NxTable.Cell className="as-table-cell-centered">
                      <HealthCheckCell name={name} openModal={() => {}}/>
                    </NxTable.Cell>
                  )}
                  {showIqPolicyViolationsColumn && (
                    <NxTable.Cell className="as-table-cell-centered">
                      <IqPolicyViolationsCell name={name}/>
                    </NxTable.Cell>
                  )}
                  <NxTable.Cell chevron />
                </NxTable.Row>
              ))}
            </NxTable.Body>
          </NxTable>
        </NxTile.Content>
      </NxTile>
    </ContentBody>
  </div>
}

async function doCopyUrl(event, url) {
  event.stopPropagation();
  const success = await copyToClipboard(url);
  if (success) {
    ExtJS.showSuccessMessage(URL_COPIED_MESSAGE);
  } else {
    ExtJS.showErrorMessage(URL_COPY_ERROR_MESSAGE);
  }
}
