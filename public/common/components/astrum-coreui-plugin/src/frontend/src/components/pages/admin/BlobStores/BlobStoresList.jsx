
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  ExtJS,
  HumanReadableUtils,
  ListMachineUtils,
  Permissions
} from '@globalart/astrum-ui-plugin';
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
  faCheckCircle,
  faExclamationCircle,
  faServer,
} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  Page,
  PageActions,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import BlobStoresListMachine from './BlobStoresListMachine';
import UIStrings from '../../../../constants/UIStrings';
import {useRouter} from "@uirouter/react";
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';

const {BLOB_STORES} = UIStrings;
const {COLUMNS} = BLOB_STORES.LIST;

const ADMIN = ROUTE_NAMES.ADMIN;

export default function BlobStoresList() {
  const router = useRouter();
  const onEdit = (type, name) => router.stateService.go(ADMIN.REPOSITORY.BLOBSTORES.EDIT, {type, name});
  const onCreate = () => router.stateService.go(ADMIN.REPOSITORY.BLOBSTORES.CREATE);
  const [current, send] = useMachine(BlobStoresListMachine, {devTools: true});
  const isCalculating = ExtJS.state().getValue('astrum.datastore.blobstore.metrics.calculating') || false;
  const isLoading = current.matches('loading');
  const {data, error, filter: filterText} = current.context;
  const hasUser = ExtJS.useUser() ?? false;
  const canCreate = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.BLOB_STORES.CREATE), [hasUser]);

  const nameSortDir = ListMachineUtils.getSortDirection('name', current.context);
  const pathSortDir = ListMachineUtils.getSortDirection('path', current.context);
  const typeSortDir = ListMachineUtils.getSortDirection('typeName', current.context);
  const stateSortDir = ListMachineUtils.getSortDirection('available', current.context);
  const countSortDir = ListMachineUtils.getSortDirection('blobCount', current.context);
  const sizeSortDir = ListMachineUtils.getSortDirection('totalSizeInBytes', current.context);
  const spaceSortDir = ListMachineUtils.getSortDirection('availableSpaceInBytes', current.context);
  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByPath = () => send({type: 'SORT_BY_PATH'});
  const sortByType = () => send({type: 'SORT_BY_TYPE_NAME'});
  const sortByState = () => send({type: 'SORT_BY_AVAILABLE'});
  const sortByCount = () => send({type: 'SORT_BY_BLOB_COUNT'});
  const sortBySize = () => send({type: 'SORT_BY_TOTAL_SIZE_IN_BYTES'});
  const sortBySpace = () => send({type: 'SORT_BY_AVAILABLE_SPACE_IN_BYTES'});

  const filter = (value) => send({type: 'FILTER', filter: value});

  return <Page className="as-blob-stores">
    <PageHeader>
      <PageTitle icon={faServer} {...BLOB_STORES.MENU} />
      <PageActions>
        <AsButton variant="primary" onClick={onCreate} disabled={!canCreate} data-analytics-id="as-blobstore-create">{BLOB_STORES.LIST.CREATE_BUTTON}</AsButton>
      </PageActions>
    </PageHeader>
    <ContentBody className="as-blob-stores-list">
      <Section>
        <SectionToolbar>
          <div className="as-spacer"/>
          <AsFilterInput
              id="filter"
              inputAttributes={{
                'aria-label': 'Filter blob stores by name',
                'data-analytics-id': 'as-blobstore-filter'
              }}
              onChange={filter}
              value={filterText}
              placeholder={BLOB_STORES.LIST.FILTER_PLACEHOLDER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</AsTableCell>
              <AsTableCell onClick={sortByPath} isSortable sortDir={pathSortDir}>{COLUMNS.PATH}</AsTableCell>
              <AsTableCell onClick={sortByType} isSortable sortDir={typeSortDir}>{COLUMNS.TYPE}</AsTableCell>
              <AsTableCell onClick={sortByState} isSortable sortDir={stateSortDir}>{COLUMNS.STATE}</AsTableCell>
              <AsTableCell onClick={sortByCount} isSortable sortDir={countSortDir} isNumeric>
                {COLUMNS.COUNT}
              </AsTableCell>
              <AsTableCell onClick={sortBySize} isSortable sortDir={sizeSortDir} isNumeric>{COLUMNS.SIZE}</AsTableCell>
              <AsTableCell onClick={sortBySpace} isSortable sortDir={spaceSortDir} isNumeric>
                {COLUMNS.SPACE}
              </AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={BLOB_STORES.LIST.EMPTY_LIST}>
            {data.map(
                ({name, path, typeId, typeName, available, unavailable, blobCount, totalSizeInBytes, availableSpaceInBytes, unlimited}) => (
                    <AsTableRow isClickable key={name} onClick={() => onEdit(`${encodeURIComponent(typeId)}`, `${encodeURIComponent(name)}`)}>
                      <AsTableCell>{name}</AsTableCell>
                      <AsTableCell className="blob-store-path">{path}</AsTableCell>
                      <AsTableCell>{typeName}</AsTableCell>
                      <AsTableCell>
                        <AsFontAwesomeIcon className={available ? 'available' : 'unavailable'}
                                           icon={available ? faCheckCircle : faExclamationCircle}/>
                        <span>{available ? BLOB_STORES.LIST.AVAILABLE : BLOB_STORES.LIST.UNAVAILABLE}</span>
                      </AsTableCell>
                      <AsTableCell isNumeric>
                        {unavailable
                          ? BLOB_STORES.LIST.UNKNOWN
                          : isCalculating
                            ? BLOB_STORES.LIST.CALCULATING
                            : blobCount
                        }
                      </AsTableCell>
                      <AsTableCell isNumeric>
                        {unavailable
                          ? BLOB_STORES.LIST.UNKNOWN
                          : isCalculating
                            ? BLOB_STORES.LIST.CALCULATING
                            : `${HumanReadableUtils.bytesToString(totalSizeInBytes)} (${totalSizeInBytes})`
                        }
                      </AsTableCell>
                      <AsTableCell isNumeric>
                        {
                          (unavailable && BLOB_STORES.LIST.UNKNOWN) ||
                          (unlimited && BLOB_STORES.LIST.UNLIMITED) ||
                          HumanReadableUtils.bytesToString(availableSpaceInBytes)
                        }
                      </AsTableCell>
                        <AsTableCell chevron/>
                    </AsTableRow>
                ))}
          </AsTableBody>
        </AsTable>
      </Section>

      <HelpTile header={BLOB_STORES.LIST.HELP.TITLE} body={BLOB_STORES.LIST.HELP.TEXT}/>
    </ContentBody>
  </Page>;
}
