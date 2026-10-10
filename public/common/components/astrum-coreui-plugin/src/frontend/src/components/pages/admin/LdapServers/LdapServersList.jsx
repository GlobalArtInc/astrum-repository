
import React from 'react';
import {useMachine} from '@xstate/react';
import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsTooltip,
  AsTile,
  AsTransferListHalf,
  AsModal,
  AsH2,
  AsFooter,
  AsButtonBar,
  AsFontAwesomeIcon,
  AsPageTitle,
} from '@globalart/react-shared-components';
import {faSortNumericUp, faTrashAlt} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle,
  SectionToolbar
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import {canDelete, canUpdate, canCreate} from './LdapServersHelper';

import {faBook} from '@fortawesome/free-solid-svg-icons';

import Machine from './LdapServersListMachine';
import UIStrings from '../../../../constants/UIStrings';

const {
  LDAP_SERVERS: {LIST: LABELS, MENU},
  SETTINGS,
} = UIStrings;
const {COLUMNS} = LABELS;

import './LdapServersList.scss';

export default function LdapServersList({onCreate, onEdit}) {
  const [state, send] = useMachine(Machine, {
    context: {transferListData: [], filterTransferList: '', modal: false},
    devTools: true,
  });
  const isLoading = state.matches('loading');
  const {
    data,
    modal,
    transferListData,
    filterTransferList,
    error,
    filter: filterText,
  } = state.context;
  const hasCreatePermission = canCreate();

  const orderSortDir = ListMachineUtils.getSortDirection(
    'order',
    state.context
  );
  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const urlSortDir = ListMachineUtils.getSortDirection('url', state.context);

  const sortByOrder = () => send({type: 'SORT_BY_ORDER'});
  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByUrl = () => send({type: 'SORT_BY_URL'});

  const filter = (value) => send({type: 'FILTER', filter: value});

  const create = () => {
    if (hasCreatePermission) {
      onCreate();
    }
  };

  const change = () => send({type: 'TOGGLE_ORDER_MODAL', value: true});
  const clear = () => send({type: 'CLEAR_CACHE'});

  const onReorder = (id, direction) => send({type: 'REORDER', id, direction});
  const onCancelOrder = () => send({type: 'TOGGLE_ORDER_MODAL', value: false});
  const onSaveOrder = () => send({type: 'SAVE_ORDER'});
  const filterOrder = (value) =>
    send({
      type: 'FILTER_ORDER_LIST',
      value,
    });

  const transferListItems = transferListData.map(({order, name}) => ({
    id: order,
    displayName: name,
  }));

  return (
    <div className="as-ldap-servers">
      <PageHeader>
        <PageTitle
          icon={faBook}
          text={MENU.text}
          description={MENU.description}
        />
        <PageActions>
          <AsTooltip
            title={!hasCreatePermission && UIStrings.PERMISSION_ERROR}
            placement="bottom"
          >
            <AsButton
              type="button"
              variant="primary"
              className={!hasCreatePermission && 'disabled'}
              onClick={create}
            >
              {LABELS.BUTTONS.CREATE}
            </AsButton>
          </AsTooltip>
        </PageActions>
      </PageHeader>
      <ContentBody className="as-ldap-servers-list">
        <AsTile>
          <AsPageTitle>
            <AsH2>{LABELS.LABEL}</AsH2>
            <AsButtonBar>
              {canUpdate() && (
                <AsButton type="button" onClick={change} variant="tertiary">
                  <AsFontAwesomeIcon icon={faSortNumericUp} />
                  <span>{LABELS.BUTTONS.CHANGE_ORDER}</span>
                </AsButton>
              )}
              {canDelete() && (
                <AsButton type="button" onClick={clear} variant="tertiary">
                  <AsFontAwesomeIcon icon={faTrashAlt} />
                  <span>{LABELS.BUTTONS.CLEAR_CACHE}</span>
                </AsButton>
              )}
            </AsButtonBar>
          </AsPageTitle>
          <SectionToolbar>
            <div className="as-spacer" />
            <AsFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.FILTER}
            />
          </SectionToolbar>
          <AsTable>
            <AsTableHead>
              <AsTableRow>
                <AsTableCell
                  onClick={sortByOrder}
                  isSortable
                  sortDir={orderSortDir}
                >
                  {COLUMNS.ORDER}
                </AsTableCell>
                <AsTableCell
                  onClick={sortByName}
                  isSortable
                  sortDir={nameSortDir}
                >
                  {COLUMNS.NAME}
                </AsTableCell>
                <AsTableCell
                  onClick={sortByUrl}
                  isSortable
                  sortDir={urlSortDir}
                >
                  {COLUMNS.URL}
                </AsTableCell>
                <AsTableCell chevron />
              </AsTableRow>
            </AsTableHead>
            <AsTableBody
              isLoading={isLoading}
              error={error}
              emptyMessage={LABELS.EMPTY_LIST}
            >
              {data.map(({id, order, name, url}) => (
                <AsTableRow
                  key={id}
                  onClick={() => onEdit(encodeURIComponent(name))}
                  isClickable
                >
                  <AsTableCell>{order}</AsTableCell>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{url}</AsTableCell>
                  <AsTableCell chevron />
                </AsTableRow>
              ))}
            </AsTableBody>
          </AsTable>
          {modal && (
            <AsModal
              onCancel={() => {}}
              aria-labelledby="modal-header-text"
              variant="narrow"
            >
              <AsModal.Header>
                <AsH2 id="modal-header-text">{LABELS.MODAL.LABEL}</AsH2>
              </AsModal.Header>
              <AsModal.Content>
                <AsTransferListHalf
                  onReorderItem={onReorder}
                  label={LABELS.MODAL.SUB_LABEL}
                  filterValue={filterTransferList}
                  onFilterChange={filterOrder}
                  items={transferListItems}
                  allowReordering
                  isSelected={false}
                  showMoveAll={false}
                  onItemChange={() => {}}
                  footerContent={LABELS.MODAL.FOOTER(transferListItems.length)}
                />
              </AsModal.Content>
              <AsFooter>
                <AsButtonBar>
                  <AsButton onClick={onCancelOrder}>
                    {SETTINGS.CANCEL_BUTTON_LABEL}
                  </AsButton>
                  <AsButton onClick={onSaveOrder} variant="primary">
                    {SETTINGS.SAVE_BUTTON_LABEL}
                  </AsButton>
                </AsButtonBar>
              </AsFooter>
            </AsModal>
          )}
        </AsTile>
        <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT} />
      </ContentBody>
    </div>
  );
}
