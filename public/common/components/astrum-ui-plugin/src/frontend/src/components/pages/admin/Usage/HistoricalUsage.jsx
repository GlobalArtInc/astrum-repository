
import React from 'react';
import {useMachine} from '@xstate/react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faInfoCircle } from '@fortawesome/free-solid-svg-icons';
import { UsageInsightsChart } from './UsageInsightsChart'

import {
  AsTable,
  AsTableHead,
  AsTableRow,
  AsTableCell,
  AsTableBody,
  AsTooltip,
  AsH2, AsP,
  AsInfoAlert,
  AsWarningAlert,
  AsErrorAlert,
  useToggle
} from '@globalart/astrum-react-shared-components';

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
        <AsH2>{UIStrings.HISTORICAL_USAGE.TITLE}</AsH2>
        {isCloud && <AsInfoAlert>{UIStrings.HISTORICAL_USAGE.USAGE_DATA_UPDATE_FREQUENCY}</AsInfoAlert>}
        {/*
          IMPORTANT: Order matters here!
          isPermissionError must be checked before loadError to ensure 403 errors
          display the permission message, not the generic error message.
        */}
        {isPermissionError ? (
          <AsWarningAlert>{UIStrings.HISTORICAL_USAGE.PERMISSION_ERROR}</AsWarningAlert>
        ) : loadError ? (
          <AsErrorAlert>
            {loadError.response?.data?.message || loadError.message || UIStrings.HISTORICAL_USAGE.LOAD_ERROR}
          </AsErrorAlert>
        ) : (
          <>
            <AsP>{UIStrings.HISTORICAL_USAGE.DESCRIPTION}</AsP>
            {isCloud && isOpen && <AsInfoAlert onClose={dismiss}>{UIStrings.HISTORICAL_USAGE.USAGE_DATA_STORAGE_EXPLANATION}</AsInfoAlert>}
            {isCloud && <UsageInsightsChart />}

            <AsTable className="historical-usage-table">
              <AsTableHead>
                <AsTableRow>
                  {columns.map(column => (
                      <AsTableCell key={column.key}>
                        {column.Header()}
                        {column.tooltip && column.showTooltip !== false && (
                            <AsTooltip title={column.tooltip}>
                              <span><FontAwesomeIcon icon={faInfoCircle} /></span>
                            </AsTooltip>
                        )}
                      </AsTableCell>
                  ))}
                </AsTableRow>
              </AsTableHead>
              <AsTableBody>
                {data && data.map((item, index) => (
                    <AsTableRow key={index}>
                      {columns.map(column => (
                          <AsTableCell key={column.key}>
                            {column.Cell(item)}
                          </AsTableCell>
                      ))}
                    </AsTableRow>
                ))}
              </AsTableBody>
            </AsTable>
          </>
        )}
      </>
  );
}
