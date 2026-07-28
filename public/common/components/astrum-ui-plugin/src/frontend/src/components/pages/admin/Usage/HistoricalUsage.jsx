
import React from 'react';
import {useMachine} from '@xstate/react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faInfoCircle } from '@fortawesome/free-solid-svg-icons';
import { UsageInsightsChart } from './UsageInsightsChart'

import {
  NxTable,
  NxTableHead,
  NxTableRow,
  NxTableCell,
  NxTableBody,
  NxTooltip,
  NxH2, NxP,
  NxInfoAlert,
  NxWarningAlert,
  NxErrorAlert,
  useToggle
} from '@globalart/react-shared-components';

import HistoricalUsageMachine from './HistoricalUsageMachine';
import './Usage.scss';
import UIStrings from "../../../../constants/UIStrings";
import ExtJS from "../../../../interface/ExtJS";

export default function HistoricalUsage({columns}) {
  const [state] = useMachine(HistoricalUsageMachine, {
      devTools: true
    });
  const {data, loadError, isPermissionError} = state.context;
  const [isOpen, dismiss] = useToggle(true);
  const isCloud = ExtJS.useState(() => ExtJS.state().getValue('isCloud', false));

  return (<>
        <NxH2>{UIStrings.HISTORICAL_USAGE.TITLE}</NxH2>
        {isCloud && <NxInfoAlert>{UIStrings.HISTORICAL_USAGE.USAGE_DATA_UPDATE_FREQUENCY}</NxInfoAlert>}
        {/*
          IMPORTANT: Order matters here!
          isPermissionError must be checked before loadError to ensure 403 errors
          display the permission message, not the generic error message.
        */}
        {isPermissionError ? (
          <NxWarningAlert>{UIStrings.HISTORICAL_USAGE.PERMISSION_ERROR}</NxWarningAlert>
        ) : loadError ? (
          <NxErrorAlert>
            {loadError.response?.data?.message || loadError.message || UIStrings.HISTORICAL_USAGE.LOAD_ERROR}
          </NxErrorAlert>
        ) : (
          <>
            <NxP>{UIStrings.HISTORICAL_USAGE.DESCRIPTION}</NxP>
            {isCloud && isOpen && <NxInfoAlert onClose={dismiss}>{UIStrings.HISTORICAL_USAGE.USAGE_DATA_STORAGE_EXPLANATION}</NxInfoAlert>}
            {isCloud && <UsageInsightsChart />}

            <NxTable className="historical-usage-table">
              <NxTableHead>
                <NxTableRow>
                  {columns.map(column => (
                      <NxTableCell key={column.key}>
                        {column.Header()}
                        {column.tooltip && column.showTooltip !== false && (
                            <NxTooltip title={column.tooltip}>
                              <span><FontAwesomeIcon icon={faInfoCircle} /></span>
                            </NxTooltip>
                        )}
                      </NxTableCell>
                  ))}
                </NxTableRow>
              </NxTableHead>
              <NxTableBody>
                {data && data.map((item, index) => (
                    <NxTableRow key={index}>
                      {columns.map(column => (
                          <NxTableCell key={column.key}>
                            {column.Cell(item)}
                          </NxTableCell>
                      ))}
                    </NxTableRow>
                ))}
              </NxTableBody>
            </NxTable>
          </>
        )}
      </>
  );
}
