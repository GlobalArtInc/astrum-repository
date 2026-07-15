
import React, { useState } from 'react';
import { useMachine } from '@xstate/react';

import { ListMachineUtils, APIConstants } from '@globalart/astrum-ui-plugin';

import {
  NxH2,
  NxFilterInput,
  NxTable,
  NxTableBody,
  NxTableCell,
  NxTableHead,
  NxTableRow,
  NxTile,
  NxPagination,
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
        <NxTile>
          <NxTile.Header>
            <NxTile.HeaderTitle>
              <NxH2>{TAGS.LIST.CAPTION}</NxH2>
            </NxTile.HeaderTitle>
            <NxTile.HeaderActions>
              <NxFilterInput
                id='filter'
                onChange={filter}
                value={filterText}
                placeholder={TAGS.LIST.FILTER_PLACEHOLDER}
              />
            </NxTile.HeaderActions>
          </NxTile.Header>
          <NxTile.Content>
            <NxTable>
              <NxTableHead>
                <NxTableRow>
                  <NxTableCell onClick={sortById} isSortable sortDir={idSortDir}>
                    {COLUMNS.NAME}
                  </NxTableCell>
                  <NxTableCell onClick={sortByFirstCreated} isSortable sortDir={firstCreatedSortDir}>
                    {COLUMNS.FIRST_CREATED}
                  </NxTableCell>
                  <NxTableCell onClick={sortByLastUpdated} isSortable sortDir={lastUpdatedSortDir}>
                    {COLUMNS.LAST_UPDATED}
                  </NxTableCell>
                  <NxTableCell chevron />
                </NxTableRow>
              </NxTableHead>
              <NxTableBody isLoading={isLoading} error={error} emptyMessage={TAGS.EMPTY_MESSAGE}>
                {paginatedData.map(({ id, firstCreatedTime, lastUpdatedTime }) => (
                  <NxTableRow key={id} isClickable onClick={() => onEdit(id)}>
                    <NxTableCell>{id}</NxTableCell>
                    <NxTableCell>{new Date(firstCreatedTime).toLocaleString()}</NxTableCell>
                    <NxTableCell>{new Date(lastUpdatedTime).toLocaleString()}</NxTableCell>
                    <NxTableCell chevron />
                  </NxTableRow>
                ))}
              </NxTableBody>
            </NxTable>
            {pageCount > 1 && !isLoading && (
              <div className='as-pagination'>
                <NxPagination onChange={changePage} pageCount={pageCount} currentPage={currentPage} />
              </div>
            )}
          </NxTile.Content>
        </NxTile>
        <HelpTile header={TAGS.HELP_MESSAGE.TITLE} body={TAGS.HELP_MESSAGE.TEXT} />
      </ContentBody>
    </div>
  );
}
