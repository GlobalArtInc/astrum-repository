
import React from 'react';
import {useMachine} from '@xstate/react';
import DOMPurify from 'dompurify';

import {
  faCheckCircle,
  faExclamationCircle,
  faMedkit,
} from '@fortawesome/free-solid-svg-icons';

import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  AsFontAwesomeIcon,
  AsTable,
  AsTile,
  AsH2,
  AsBackButton,
} from '@globalart/astrum-react-shared-components';

import {
  ContentBody,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import MetricHealthMachine from './MetricHealthMachine';

import UIStrings from '../../../../constants/UIStrings';
import {isClustered} from './MetricHealthHelper';

const {METRIC_HEALTH} = UIStrings;

export default function MetricHealthDetails({itemId, onDone}) {
  const [state, send] = useMachine(MetricHealthMachine, {
    devTools: true,
    context: {itemId},
  });
  const isLoading = state.matches('loading');
  const {data, error, name} = state.context;

  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const messageSortDir = ListMachineUtils.getSortDirection(
    'message',
    state.context
  );
  const errorSortDir = ListMachineUtils.getSortDirection(
    'error',
    state.context
  );

  return (
    <div>
      <div>
        <PageTitle
          icon={faMedkit}
          text={METRIC_HEALTH.MENU.text}
          description={METRIC_HEALTH.MENU.detailsDescription}
        />
        {isClustered() && (
          <AsBackButton
            text={METRIC_HEALTH.BACK_BUTTON}
            href="#admin/support/status"
          />
        )}
      </div>
      <ContentBody className="as-metric-health">
        <AsTile>
          {isClustered() && <AsH2>{name}</AsH2>}
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
                  onClick={() => send({type: 'SORT_BY_MESSAGE'})}
                  isSortable
                  sortDir={messageSortDir}
                >
                  {METRIC_HEALTH.MESSAGE_HEADER}
                </AsTable.Cell>
                <AsTable.Cell
                  onClick={() => send({type: 'SORT_BY_ERROR'})}
                  isSortable
                  sortDir={errorSortDir}
                >
                  {METRIC_HEALTH.ERROR_HEADER}
                </AsTable.Cell>
              </AsTable.Row>
            </AsTable.Head>
            <AsTable.Body
              isLoading={isLoading}
              error={error}
              emptyMessage={METRIC_HEALTH.EMPTY_NODE}
            >
              {data.map((metric) => (
                <AsTable.Row key={metric.name}>
                  <AsTable.Cell hasIcon>
                    <AsFontAwesomeIcon
                      color={metric.healthy ? 'green' : 'red'}
                      icon={
                        metric.healthy ? faCheckCircle : faExclamationCircle
                      }
                    />
                  </AsTable.Cell>
                  <AsTable.Cell>{metric.name}</AsTable.Cell>
                  <AsTable.Cell>
                    {/* SECURITY FIX: Sanitize HTML to prevent XSS attacks */}
                    <span dangerouslySetInnerHTML={{__html: DOMPurify.sanitize(metric.message)}} />
                  </AsTable.Cell>
                  <AsTable.Cell>
                    {metric.error ? metric.error.message : ''}
                  </AsTable.Cell>
                </AsTable.Row>
              ))}
            </AsTable.Body>
          </AsTable>
        </AsTile>
      </ContentBody>
    </div>
  );
}
