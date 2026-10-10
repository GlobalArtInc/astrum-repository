
import React from 'react';
import {useMachine} from '@xstate/react';
import {DateTime} from 'luxon';
import {faScroll} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  HumanReadableUtils,
  ListMachineUtils,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar
} from '@globalart/astrum-ui-plugin';

import {
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsFilterInput
} from '@globalart/astrum-react-shared-components';

import LogsListMachine from './LogsListMachine';

import UIStrings from '../../../../constants/UIStrings';

export default function LogsList({onEdit}) {
  const [current, send] = useMachine(LogsListMachine, {devTools: true});
  const isLoading = current.matches('loading');
  const data = current.context.data;
  const error = current.context.error;
  const filter = current.context.filter;

  const fileNameSortDir = ListMachineUtils.getSortDirection('fileName', current.context);
  const sizeSortDir = ListMachineUtils.getSortDirection('size', current.context);
  const lastModifiedSortDir = ListMachineUtils.getSortDirection('lastModified', current.context);

  return <div className="as-logs">
    <PageHeader>
      {/*icon does not match what's in constants faScroll vs faTerminal */}
      <PageTitle
          icon={faScroll}
          text={UIStrings.LOGS.MENU.text}
          description={UIStrings.LOGS.MENU.description}
      />
    </PageHeader>
    <ContentBody>
      <Section className="as-logs-list">
        <SectionToolbar>
          <AsFilterInput
              className="as-logs-filter"
              id="filter"
              inputAttributes={{'data-analytics-id': 'as-logs-filter'}}
              onChange={(value) => send({type: 'FILTER', filter: value})}
              value={filter}
              placeholder={UIStrings.LOGS.LIST.FILTER_PLACEHOLDER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={() => send({type: 'SORT_BY_FILE_NAME'})} isSortable sortDir={fileNameSortDir}>
                {UIStrings.LOGS.LIST.FILE_NAME_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_SIZE'})} isSortable sortDir={sizeSortDir}>
                {UIStrings.LOGS.LIST.SIZE_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_LAST_MODIFIED'})} isSortable sortDir={lastModifiedSortDir}>
                {UIStrings.LOGS.LIST.LAST_MODIFIED_LABEL}
              </AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error}>
            {/* Encoding mirrors encodeLogFilename in astrum-ui-plugin/types.ts — keep both in sync */}
            {data.map(({fileName, size, lastModified}) => (
                <AsTableRow key={fileName} onClick={() => onEdit(fileName.split('/').map(encodeURIComponent).join('/'))} isClickable>
                  <AsTableCell>{fileName}</AsTableCell>
                  <AsTableCell>{HumanReadableUtils.bytesToString(size)}</AsTableCell>
                  <AsTableCell>
                    {DateTime.fromMillis(lastModified).toLocaleString(DateTime.DATETIME_SHORT_WITH_SECONDS)}
                  </AsTableCell>
                  <AsTableCell chevron/>
                </AsTableRow>
            ))}
          </AsTableBody>
        </AsTable>
      </Section>
    </ContentBody>
  </div>;
}
