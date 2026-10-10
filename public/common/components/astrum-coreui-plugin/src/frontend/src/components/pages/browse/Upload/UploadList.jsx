
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsButtonBar,
  AsFilterInput,
  AsFontAwesomeIcon,
  AsTable,
  AsTile
} from '@globalart/astrum-react-shared-components';

import {faCopy, faUpload} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Page
} from '@globalart/astrum-ui-plugin';

import UploadListMachine from './UploadListMachine';
import UIStrings from '../../../../constants/UIStrings';
import './UploadList.scss';
import { useRouter } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';
import { copyUrl as defaultCopyUrl } from '../BrowseUtils';

const {UPLOAD, FILTER} = UIStrings;
const {COLUMNS, COPY_URL_TITLE} = UPLOAD.LIST;

export default function UploadList({copyUrl = defaultCopyUrl} = {}) {
  const router = useRouter();
  const onEdit = useCallback(itemId => router.stateService.go(ROUTE_NAMES.BROWSE.UPLOAD.EDIT, { itemId }));

  const [state, send] = useMachine(UploadListMachine, {devTools: true});
  const {data, error, filter: filterText} = state.context;
  const isLoading = state.matches('loading');

  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const formatSortDir = ListMachineUtils.getSortDirection('format', state.context);
  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByFormat = () => send({type: 'SORT_BY_FORMAT'});

  const filter = (value) => send({type: 'FILTER', filter: value});

  return <Page className="as-upload">
    <PageHeader>
      <PageTitle icon={faUpload} {...UPLOAD.MENU}/>
    </PageHeader>
    <ContentBody className="as-upload-list">
      <AsTile>
        <AsTile.Header>
            <AsTile.HeaderActions>
              <AsFilterInput
                  id="filter"
                  onChange={filter}
                  value={filterText}
                  placeholder={FILTER}/>
            </AsTile.HeaderActions>
        </AsTile.Header>
        <AsTile.Content>
          <AsTable id="as-upload-list-repo-table">
            <AsTable.Head>
              <AsTable.Row>
                <AsTable.Cell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</AsTable.Cell>
                <AsTable.Cell onClick={sortByFormat} isSortable sortDir={formatSortDir}>{COLUMNS.FORMAT}</AsTable.Cell>
                <AsTable.Cell>{COLUMNS.URL}</AsTable.Cell>
                <AsTable.Cell chevron />
              </AsTable.Row>
            </AsTable.Head>
            <AsTable.Body isLoading={isLoading} error={error} emptyMessage={UPLOAD.EMPTY_MESSAGE}>
              {data.map(
                ({name, format, url}) => (
                  <AsTable.Row key={name} isClickable onClick={() => onEdit(name)}>
                    <AsTable.Cell>{name}</AsTable.Cell>
                    <AsTable.Cell>{format}</AsTable.Cell>
                    <AsTable.Cell>
                      <AsButtonBar>
                        <AsButton
                          variant="icon-only"
                          title={COPY_URL_TITLE}
                          onClick={(e) => copyUrl(e, url)}>
                          <AsFontAwesomeIcon icon={faCopy} />
                        </AsButton>
                      </AsButtonBar>
                    </AsTable.Cell>
                    <AsTable.Cell chevron />
                  </AsTable.Row>
              ))}
            </AsTable.Body>
          </AsTable>
        </AsTile.Content>
      </AsTile>
    </ContentBody>
  </Page>
}
