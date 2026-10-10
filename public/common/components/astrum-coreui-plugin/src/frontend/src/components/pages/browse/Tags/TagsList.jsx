
import React, { useState } from 'react';
import { useMachine } from '@xstate/react';

import { ListMachineUtils, APIConstants } from '@globalart/astrum-ui-plugin';

import {
  AsH2,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsTile,
  AsPagination,
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import TagsListMachine from './TagsListMachine';
import { faTags } from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../constants/UIStrings';

const {TAGS} = UIStrings;
const {COLUMNS} = TAGS.LIST;

export default function TagsList({onEdit}) {
  const [state, send] = useMachine(TagsListMachine, {devTools: true});
  const {data, error, filter: filterText} = state.context;
  const isLoading = state.matches('loading');

  const idSortDir = ListMachineUtils.getSortDirection('id', state.context);
  const firstCreatedSortDir = ListMachineUtils.getSortDirection('firstCreatedTime', state.context);
  const lastUpdatedSortDir = ListMachineUtils.getSortDirection('lastUpdatedTime', state.context);
  const sortById = () => send({type: 'SORT_BY_ID'});
  const sortByFirstCreated = () => send({type: 'SORT_BY_FIRST_CREATED_TIME'});
  const sortByLastUpdated = () => send({type: 'SORT_BY_LAST_UPDATED_TIME'});

  const filter = value => {
    setCurrentPage(0);
    send({ type: 'FILTER', filter: value });
  };

  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = APIConstants.EXT.SMALL_PAGE_SIZE;
  const pageCount = Math.ceil(data.length / pageSize);
  const changePage = page => setCurrentPage(page);

  const paginatedData = data.slice(currentPage * pageSize, (currentPage + 1) * pageSize);

  return (
    <div className='as-tags'>
      <PageHeader>
        <PageTitle icon={faTags} {...TAGS.MENU} />
      </PageHeader>
      <ContentBody className='as-tags-list'>
        <AsTile>
          <AsTile.Header>
            <AsTile.HeaderTitle>
              <AsH2>{TAGS.LIST.CAPTION}</AsH2>
            </AsTile.HeaderTitle>
            <AsTile.HeaderActions>
              <AsFilterInput
                id='filter'
                onChange={filter}
                value={filterText}
                placeholder={TAGS.LIST.FILTER_PLACEHOLDER}
              />
            </AsTile.HeaderActions>
          </AsTile.Header>
          <AsTile.Content>
            <AsTable>
              <AsTableHead>
                <AsTableRow>
                  <AsTableCell onClick={sortById} isSortable sortDir={idSortDir}>
                    {COLUMNS.NAME}
                  </AsTableCell>
                  <AsTableCell onClick={sortByFirstCreated} isSortable sortDir={firstCreatedSortDir}>
                    {COLUMNS.FIRST_CREATED}
                  </AsTableCell>
                  <AsTableCell onClick={sortByLastUpdated} isSortable sortDir={lastUpdatedSortDir}>
                    {COLUMNS.LAST_UPDATED}
                  </AsTableCell>
                  <AsTableCell chevron />
                </AsTableRow>
              </AsTableHead>
              <AsTableBody isLoading={isLoading} error={error} emptyMessage={TAGS.EMPTY_MESSAGE}>
                {paginatedData.map(({ id, firstCreatedTime, lastUpdatedTime }) => (
                  <AsTableRow key={id} isClickable onClick={() => onEdit(id)}>
                    <AsTableCell>{id}</AsTableCell>
                    <AsTableCell>{new Date(firstCreatedTime).toLocaleString()}</AsTableCell>
                    <AsTableCell>{new Date(lastUpdatedTime).toLocaleString()}</AsTableCell>
                    <AsTableCell chevron />
                  </AsTableRow>
                ))}
              </AsTableBody>
            </AsTable>
            {pageCount > 1 && !isLoading && (
              <div className='astrum-pagination'>
                <AsPagination onChange={changePage} pageCount={pageCount} currentPage={currentPage} />
              </div>
            )}
          </AsTile.Content>
        </AsTile>
        <HelpTile header={TAGS.HELP_MESSAGE.TITLE} body={TAGS.HELP_MESSAGE.TEXT} />
      </ContentBody>
    </div>
  );
}
