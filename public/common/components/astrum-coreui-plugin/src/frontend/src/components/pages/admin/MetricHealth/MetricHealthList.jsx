
import React from 'react';
import DOMPurify from 'dompurify';
import {useMachine} from '@xstate/react';
import {isEmpty} from 'ramda';
import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  faMedkit,
  faExclamationCircle,
  faCheckCircle,
} from '@fortawesome/free-solid-svg-icons';
import {
  AsTile,
  AsTable,
  AsFontAwesomeIcon,
} from '@globalart/astrum-react-shared-components';

import {
  ContentBody,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

import MetricHealthListMachine from './MetricHealthListMachine';

import UpgradeAlert from './UpgradeAlert';

const {METRIC_HEALTH} = UIStrings;

export default function MetricHealthList({onEdit}) {
  const [state, send] = useMachine(MetricHealthListMachine, {devTools: true});
  const {data, error} = state.context;
  const isLoading = state.matches('loading');

  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const errorSortDir = ListMachineUtils.getSortDirection(
    'error',
    state.context
  );
  const messageSortDir = ListMachineUtils.getSortDirection(
    'message',
    state.context
  );

  return (
    <div className="as-metric-health">
      <PageHeader>
        <PageTitle
          icon={faMedkit}
          text={METRIC_HEALTH.MENU.text}
          description={METRIC_HEALTH.MENU.description}
        />
      </PageHeader>
      <ContentBody className="as-metric-health-list">
        <AsTile>
          <UpgradeAlert/>
          <AsTable>
            <AsTable.Head>
              <AsTable.Row>
                <AsTable.Cell hasIcon />
                <AsTable.Cell
                  onClick={() => send({type: 'SORT_BY_NAME'})}
                  isSortable
                  sortDir={nameSortDir}
                >
                  {METRIC_HEALTH.NAME_HEADER}
                </AsTable.Cell>
                <AsTable.Cell
                  onClick={() => send({type: 'SORT_BY_ERROR'})}
                  isSortable
                  sortDir={errorSortDir}
                >
                  {METRIC_HEALTH.ERROR_HEADER}
                </AsTable.Cell>
                <AsTable.Cell
                  onClick={() => send({type: 'SORT_BY_MESSAGE'})}
                  isSortable
                  sortDir={messageSortDir}
                >
                  {METRIC_HEALTH.MESSAGE_HEADER}
                </AsTable.Cell>
                <AsTable.Cell chevron />
              </AsTable.Row>
            </AsTable.Head>
            <AsTable.Body
              isLoading={isLoading}
              error={error}
              emptyMessage={METRIC_HEALTH.EMPTY_NODE_LIST}
            >
              {data.map(({name, error, message, nodeId}) => (
                <AsTable.Row
                  key={nodeId}
                  onClick={() => onEdit(encodeURIComponent(nodeId))}
                  isClickable
                >
                  <AsTable.Cell hasIcon>
                    <AsFontAwesomeIcon
                      color={isEmpty(error) ? 'green' : 'red'}
                      icon={
                        isEmpty(error) ? faCheckCircle : faExclamationCircle
                      }
                    />
                  </AsTable.Cell>
                  <AsTable.Cell>{name}</AsTable.Cell>
                  <AsTable.Cell>{error}</AsTable.Cell>
                  <AsTable.Cell>
                    <p
                      dangerouslySetInnerHTML={{
                        __html: DOMPurify.sanitize(message),
                      }}
                    />
                  </AsTable.Cell>
                  <AsTable.Cell chevron />
                </AsTable.Row>
              ))}
            </AsTable.Body>
          </AsTable>
        </AsTile>
        <HelpTile
          header={METRIC_HEALTH.HELP.LABEL}
          body={METRIC_HEALTH.HELP.SUB_LABEL}
        />
      </ContentBody>
    </div>
  );
}
