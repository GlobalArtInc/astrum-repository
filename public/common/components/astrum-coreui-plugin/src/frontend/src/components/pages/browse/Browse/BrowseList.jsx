
import React, {useEffect, useState} from 'react';

import {ExtJS, ListMachineUtils, copyToClipboard} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsButtonBar,
  AsFilterInput,
  AsFontAwesomeIcon,
  AsTable,
  AsTile
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
      <AsTile>
        <AsTile.Header>
          <AsTile.HeaderActions>
            <AsFilterInput
                    id="filter"
                    onChange={filter}
                    value={filterText}
                    placeholder={FILTER_PLACEHOLDER}/>
          </AsTile.HeaderActions>
        </AsTile.Header>
        <AsTile.Content>
          <AsTable>
            <AsTable.Head>
              <AsTable.Row>
                <AsTable.Cell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</AsTable.Cell>
                <AsTable.Cell onClick={sortByType} isSortable sortDir={typeSortDir}>{COLUMNS.TYPE}</AsTable.Cell>
                <AsTable.Cell onClick={sortByFormat} isSortable sortDir={formatSortDir}>{COLUMNS.FORMAT}</AsTable.Cell>
                <AsTable.Cell onClick={sortByStatus} isSortable sortDir={statusSortDir}>{COLUMNS.STATUS}</AsTable.Cell>
                <AsTable.Cell>{COLUMNS.URL}</AsTable.Cell>
                {showHealthCheckColumn && (
                  <AsTable.Cell className="as-table-cell-centered">
                    {COLUMNS.HEALTH_CHECK}
                  </AsTable.Cell>
                )}
                {showIqPolicyViolationsColumn && (
                  <AsTable.Cell className="as-table-cell-centered">
                    {COLUMNS.IQ_POLICY_VIOLATIONS}
                  </AsTable.Cell>
                )}
                <AsTable.Cell chevron />
              </AsTable.Row>
            </AsTable.Head>
            <AsTable.Body isLoading={isLoading} error={error} emptyMessage={EMPTY_MESSAGE}>
              {data.map(({name, type, format, status, url}) => (
                <AsTable.Row key={name} onClick={() => onEdit(name)} isClickable>
                  <AsTable.Cell>{name}</AsTable.Cell>
                  <AsTable.Cell>{type}</AsTable.Cell>
                  <AsTable.Cell>{format}</AsTable.Cell>
                  <AsTable.Cell>
                    <RepositoryStatus status={status} />
                  </AsTable.Cell>
                  <AsTable.Cell>
                    <AsButtonBar>
                      <AsButton
                        variant="icon-only"
                        onClick={(e) => copyUrl(e, url)}
                        title={COPY_URL_TITLE}
                        aria-label={COPY_URL_TITLE}
                      >
                        <AsFontAwesomeIcon icon={faCopy} />
                      </AsButton>
                    </AsButtonBar>
                  </AsTable.Cell>
                  {showHealthCheckColumn && (
                    <AsTable.Cell className="as-table-cell-centered">
                      <HealthCheckCell name={name} openModal={() => {}}/>
                    </AsTable.Cell>
                  )}
                  {showIqPolicyViolationsColumn && (
                    <AsTable.Cell className="as-table-cell-centered">
                      <IqPolicyViolationsCell name={name}/>
                    </AsTable.Cell>
                  )}
                  <AsTable.Cell chevron />
                </AsTable.Row>
              ))}
            </AsTable.Body>
          </AsTable>
        </AsTile.Content>
      </AsTile>
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
