
import React, {useState, useEffect} from 'react';

import {ExtJS, ListMachineUtils, copyToClipboard} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsButtonBar,
  AsFilterInput,
  AsFontAwesomeIcon,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsTooltip
} from '@globalart/react-shared-components';
import {faCopy, faDatabase} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import {useRepositoriesService} from './RepositoriesContextProvider';

import UIStrings from '../../../../constants/UIStrings';

import {
  isIqServerEnabled,
  canReadFirewallStatus,
  canUpdateHealthCheck,
  hasFirewall
} from './IQServerColumns/IQServerHelpers';

import RepositoryStatus from './RepositoryStatus';
import HealthCheckCell from './IQServerColumns/HealthCheckCell';
import AnalyzeConfirmationModal from './IQServerColumns/AnalyzeConfirmationModal';
import IqPolicyViolationsCell from './IQServerColumns/IqPolicyViolationsCell';

const {REPOSITORIES} = UIStrings;
const {COLUMNS} = REPOSITORIES.LIST;

export default function RepositoriesList({onCreate, onEdit, copyUrl = doCopyUrl}) {
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

  const isLoading = state.matches('loading');
  const {data, error, filter: filterText} = state.context;

  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const typeSortDir = ListMachineUtils.getSortDirection('type', state.context);
  const formatSortDir = ListMachineUtils.getSortDirection('format', state.context);
  const statusSortDir = ListMachineUtils.getSortDirection('status', state.context);
  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByType = () => send({type: 'SORT_BY_TYPE'});
  const sortByFormat = () => send({type: 'SORT_BY_FORMAT'});
  const sortByStatus = () => send({type: 'SORT_BY_STATUS'});

  const filter = (value) => send({type: 'FILTER', filter: value});

  const canCreate = ExtJS.checkPermission('astrum:repository-admin:*:*:add');

  // Show Health Check column only if user has permission AND NOT (IQ enabled AND Firewall enabled)
  // Health Check should show when: IQ is disabled OR IQ doesn't have Firewall
  // clmState is used to trigger re-renders when IQ/Firewall config changes via 'clmchanged' event
  // We call helper functions directly for the logic (so tests work with mocked helpers)
  const showHealthCheckColumn = canUpdateHealthCheck() && !(isIqServerEnabled() && hasFirewall());
  const showIqPolicyViolationsColumn = isIqServerEnabled() && canReadFirewallStatus();

  function create() {
    if (canCreate) {
      onCreate();
    }
  }

  const [healthCheckModalState, setHealthCheckModalState] = useState({
    isOpen: false,
    repoName: ''
  });
  const openHealthCheckModal = (repoName) =>
    setHealthCheckModalState({
      isOpen: true,
      repoName
    });
  const closeHealthCheckModal = () =>
    setHealthCheckModalState({
      isOpen: false,
      repoName: ''
    });

  return (
    <div className="as-repositories">
      <PageHeader>
        <PageTitle icon={faDatabase} {...REPOSITORIES.MENU} />
        <PageActions>
          <AsTooltip title={!canCreate && UIStrings.PERMISSION_ERROR} placement="bottom">
            <AsButton
              type="button"
              variant="primary"
              className={!canCreate && 'disabled'}
              onClick={create}
            >
              {REPOSITORIES.LIST.CREATE_BUTTON}
            </AsButton>
          </AsTooltip>
        </PageActions>
      </PageHeader>
      <ContentBody className="as-repositories-list">
        <Section>
          <SectionToolbar>
            <div className="as-spacer" />
            <AsFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={REPOSITORIES.LIST.FILTER_PLACEHOLDER}
            />
          </SectionToolbar>
          <AsTable>
            <AsTableHead>
              <AsTableRow>
                <AsTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>
                  {COLUMNS.NAME}
                </AsTableCell>
                <AsTableCell onClick={sortByType} isSortable sortDir={typeSortDir}>
                  {COLUMNS.TYPE}
                </AsTableCell>
                <AsTableCell onClick={sortByFormat} isSortable sortDir={formatSortDir}>
                  {COLUMNS.FORMAT}
                </AsTableCell>
                <AsTableCell onClick={sortByStatus} isSortable sortDir={statusSortDir}>
                  {COLUMNS.STATUS}
                </AsTableCell>
                <AsTableCell className="as-table-cell-centered">{COLUMNS.URL}</AsTableCell>
                {showHealthCheckColumn && (
                  <AsTableCell className="as-table-cell-centered">
                    {COLUMNS.HEALTH_CHECK}
                  </AsTableCell>
                )}
                {showIqPolicyViolationsColumn && (
                  <AsTableCell className="as-table-cell-centered">{COLUMNS.IQ}</AsTableCell>
                )}
                <AsTableCell chevron />
              </AsTableRow>
            </AsTableHead>
            <AsTableBody
              isLoading={isLoading}
              error={error}
              emptyMessage={REPOSITORIES.LIST.EMPTY_LIST}
            >
              {data.map(({name, type, format, url, status}) => (
                <AsTableRow key={name} onClick={() => onEdit(name)} isClickable>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{type}</AsTableCell>
                  <AsTableCell>{format}</AsTableCell>
                  <AsTableCell>
                    <RepositoryStatus status={status} />
                  </AsTableCell>
                  <AsTableCell className="as-table-cell-centered">
                    <AsButtonBar>
                      <AsButton
                        variant="icon-only"
                        onClick={(e) => copyUrl(e, url)}
                        title={REPOSITORIES.LIST.COPY_URL_TITLE}
                      >
                        <AsFontAwesomeIcon icon={faCopy} />
                      </AsButton>
                    </AsButtonBar>
                  </AsTableCell>
                  {showHealthCheckColumn && (
                    <AsTableCell className="as-table-cell-centered">
                      <HealthCheckCell name={name} openModal={() => openHealthCheckModal(name)} />
                    </AsTableCell>
                  )}
                  {showIqPolicyViolationsColumn && (
                    <AsTableCell className="as-table-cell-centered">
                      <IqPolicyViolationsCell name={name} />
                    </AsTableCell>
                  )}
                  <AsTableCell chevron />
                </AsTableRow>
              ))}
            </AsTableBody>
          </AsTable>
        </Section>

        <HelpTile header={REPOSITORIES.LIST.HELP.TITLE} body={REPOSITORIES.LIST.HELP.TEXT} />
      </ContentBody>
      {healthCheckModalState.isOpen && (
        <AnalyzeConfirmationModal
          close={closeHealthCheckModal}
          name={healthCheckModalState.repoName}
        />
      )}
    </div>
  );
}

async function doCopyUrl(event, url) {
  event.stopPropagation();
  const success = await copyToClipboard(url);
  if (success) {
    ExtJS.showSuccessMessage(REPOSITORIES.LIST.URL_COPIED_MESSAGE);
  } else {
    ExtJS.showErrorMessage(REPOSITORIES.LIST.URL_COPY_ERROR_MESSAGE);
  }
}
