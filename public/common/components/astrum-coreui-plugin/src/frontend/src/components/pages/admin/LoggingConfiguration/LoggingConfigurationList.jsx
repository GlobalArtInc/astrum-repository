
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {faRedo, faScroll} from '@fortawesome/free-solid-svg-icons';

import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFilterInput,
  AsFontAwesomeIcon,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
} from '@globalart/astrum-react-shared-components';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar,
  Page
} from '@globalart/astrum-ui-plugin';

import LoggingConfigurationListMachine from './LoggingConfigurationListMachine';

import UIStrings from '../../../../constants/UIStrings';
import { useRouter } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';

export default function LoggingConfigurationList() {
  const router = useRouter();
  const onEdit = useCallback(itemId => router.stateService.go(ROUTE_NAMES.ADMIN.SUPPORT.LOGGING.EDIT, { itemId }));
  const onCreate = useCallback(() => router.stateService.go(ROUTE_NAMES.ADMIN.SUPPORT.LOGGING.CREATE));

  const [current, send] = useMachine(LoggingConfigurationListMachine, {devTools: true});
  const isLoading = current.matches('loading');
  const data = current.context.data;
  const filterText = current.context.filter;
  const error = current.context.error;

  const nameSortDir = ListMachineUtils.getSortDirection('name', current.context);
  const levelSortDir = ListMachineUtils.getSortDirection('level', current.context);

  function filter(value) {
    send({type: 'FILTER', filter: value});
  }

  function reset() {
    send({type: 'RESET'});
  }

  return <Page className="as-logging-configuration">
    <PageHeader>

      <PageTitle
          icon={UIStrings.LOGGING.MENU.icon}
          text={UIStrings.LOGGING.MENU.text}
          description={UIStrings.LOGGING.MENU.description}
      />
      <PageActions>
        <AsButton variant="tertiary" onClick={reset}>
          <AsFontAwesomeIcon icon={faRedo}/>
          <span>{UIStrings.LOGGING.RESET_ALL_BUTTON}</span>
        </AsButton>
        <AsButton variant="primary" onClick={onCreate}>
          <span>{UIStrings.LOGGING.CREATE_BUTTON}</span>
        </AsButton>
      </PageActions>
    </PageHeader>
    <ContentBody>
      <Section className="as-logging-configuration-list">
        <SectionToolbar>
          <div className="as-spacer" />
          <AsFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.LOGGING.FILTER_PLACEHOLDER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={() => send({type: 'SORT_BY_NAME'})} isSortable sortDir={nameSortDir}>
                {UIStrings.LOGGING.NAME_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_LEVEL'})} isSortable sortDir={levelSortDir}>
                {UIStrings.LOGGING.LEVEL_LABEL}
              </AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error}>
            {data.map(({name, level}) => (
                <AsTableRow key={name} onClick={() => onEdit(name)} isClickable>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{level}</AsTableCell>
                  <AsTableCell chevron/>
                </AsTableRow>
            ))}
          </AsTableBody>
        </AsTable>
      </Section>
    </ContentBody>
  </Page>;
}
