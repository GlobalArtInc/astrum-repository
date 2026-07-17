
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
  NxTile,
  NxTable,
  NxFontAwesomeIcon,
} from '@globalart/react-shared-components';

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
        <NxTile>
          <UpgradeAlert/>
          <NxTable>
            <NxTable.Head>
              <NxTable.Row>
                <NxTable.Cell hasIcon />
                <NxTable.Cell
                  onClick={() => send({type: 'SORT_BY_NAME'})}
                  isSortable
                  sortDir={nameSortDir}
                >
                  {METRIC_HEALTH.NAME_HEADER}
                </NxTable.Cell>
                <NxTable.Cell
                  onClick={() => send({type: 'SORT_BY_ERROR'})}
                  isSortable
                  sortDir={errorSortDir}
                >
                  {METRIC_HEALTH.ERROR_HEADER}
                </NxTable.Cell>
                <NxTable.Cell
                  onClick={() => send({type: 'SORT_BY_MESSAGE'})}
                  isSortable
                  sortDir={messageSortDir}
                >
                  {METRIC_HEALTH.MESSAGE_HEADER}
                </NxTable.Cell>
                <NxTable.Cell chevron />
              </NxTable.Row>
            </NxTable.Head>
            <NxTable.Body
              isLoading={isLoading}
              error={error}
              emptyMessage={METRIC_HEALTH.EMPTY_NODE_LIST}
            >
              {data.map(({name, error, message, nodeId}) => (
                <NxTable.Row
                  key={nodeId}
                  onClick={() => onEdit(encodeURIComponent(nodeId))}
                  isClickable
                >
                  <NxTable.Cell hasIcon>
                    <NxFontAwesomeIcon
                      color={isEmpty(error) ? 'green' : 'red'}
                      icon={
                        isEmpty(error) ? faCheckCircle : faExclamationCircle
                      }
                    />
                  </NxTable.Cell>
                  <NxTable.Cell>{name}</NxTable.Cell>
                  <NxTable.Cell>{error}</NxTable.Cell>
                  <NxTable.Cell>
                    <p
                      dangerouslySetInnerHTML={{
                        __html: DOMPurify.sanitize(message),
                      }}
                    />
                  </NxTable.Cell>
                  <NxTable.Cell chevron />
                </NxTable.Row>
              ))}
            </NxTable.Body>
          </NxTable>
        </NxTile>
        <HelpTile
          header={METRIC_HEALTH.HELP.LABEL}
          body={METRIC_HEALTH.HELP.SUB_LABEL}
        />
      </ContentBody>
    </div>
  );
}
