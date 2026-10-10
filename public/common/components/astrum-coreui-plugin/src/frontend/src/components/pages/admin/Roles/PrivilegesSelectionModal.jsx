
import React from 'react';

import {
  AsModal,
  AsH2,
  AsButton,
  AsCheckbox,
  AsPagination,
  AsTable,
  AsTableHead,
  AsTableRow,
  AsTableCell,
  AsTableBody,
  AsFilterInput,
  AsCounter,
  AsFooter
} from '@globalart/astrum-react-shared-components';

import PrivilegeSelectionMachine from './PrivilegeSelectionMachine';
import { useMachine } from '@xstate/react';
import { createPortal } from 'react-dom';
import { ListMachineUtils } from '@globalart/astrum-ui-plugin';
import classNames from 'classnames';
import UIStrings from '../../../../constants/UIStrings';

const {ROLES: {SELECTION_MODAL}} = UIStrings;

export default function PrivilegesSelectionModal({
    title,
    allPrivileges,
    onModalClose,
    selectedPrivileges,
    saveModal
  }) {
  const [state, send] = useMachine(PrivilegeSelectionMachine, {
    context: {
      offsetPage: 0,
      data: allPrivileges.map(item => ({
        ...item,
        isSelected: selectedPrivileges.includes(item.name)
      })),
      selectedPrivileges: selectedPrivileges,
      tempSelectedPrivileges: selectedPrivileges,
      filteredData: allPrivileges,
      numberOfPrivileges: allPrivileges.length,
    },
    devTools: true
  });

  const {
    offsetPage,
    tempSelectedPrivileges,
    numberOfPrivileges,
    filteredData,
    filter: filterText,
  } = state.context;

  const onPrivilegeSelected = (privilege) => send({ type: 'SELECT_PRIVILEGE', privilege: privilege });
  const isPrivilegeSelected = (privilege) => { return tempSelectedPrivileges.includes(privilege) };
  const confirmModal = () => {
    onModalClose();
    send({ type: 'ON_CONFIRM' });
    saveModal(tempSelectedPrivileges);
  }

  const filter = (value) => send({ type: 'FILTER', filter: value });

  const rowsPerPage = 10;
  const pages = Math.ceil(numberOfPrivileges / rowsPerPage);
  const changePage = (event) => send({ type: 'CHANGE_PAGE', offsetPage: event });
  const lowerBound = offsetPage * rowsPerPage;
  const upperBound = lowerBound + rowsPerPage;

  const selectSortDir = ListMachineUtils.getSortDirection('select', state.context);
  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const descriptionSortDir = ListMachineUtils.getSortDirection('description', state.context);

  const sortBySelect = () => send({type: 'SORT_BY_SELECT'});
  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByDescription = () => send({type: 'SORT_BY_DESCRIPTION'});

  const counterClasses = classNames('as-pull-right', {
    'as-counter--active': tempSelectedPrivileges.length > 0
  });

  return (<>
    {createPortal(
      <AsModal variant="wide" onCancel={onModalClose}>
        <header className="as-modal-header">
          <AsH2 className="as-h2" id="modal-header-text">{UIStrings.PAGES.SELECTION_MODAL.TITLE(title)}</AsH2>
        </header>
        <div className="as-modal__filter-box">
          <AsFilterInput className="as-modal__filter"
            placeholder={UIStrings.PAGES.SELECTION_MODAL.FILTER}
            value={filterText}
            onChange={filter} 
            id= "modalFilter"/>
          {SELECTION_MODAL.WILDCARD_TEXT}
        </div>
        <div className="as-modal-content">
          <AsTable className="modal-table">
            <AsTableHead>
              <AsTableRow>
                <AsTableCell className="select-column" onClick={sortBySelect} isSortable sortDir={selectSortDir}>{UIStrings.PAGES.SELECTION_MODAL.SELECT}</AsTableCell>
                <AsTableCell className="name-column" onClick={sortByName} isSortable sortDir={nameSortDir}>{UIStrings.PAGES.SELECTION_MODAL.NAME}</AsTableCell>
                <AsTableCell className="desc-column" onClick={sortByDescription} isSortable sortDir={descriptionSortDir}>{UIStrings.PAGES.SELECTION_MODAL.DESCRIPTION}</AsTableCell>
              </AsTableRow>
              <AsTableRow className="counter-row">
                <AsTableCell></AsTableCell>
                <AsTableCell></AsTableCell>
                <AsTableCell className="counter-cell">
                  <AsCounter className={counterClasses}>{UIStrings.PAGES.SELECTION_MODAL.SELECTED(tempSelectedPrivileges.length)}</AsCounter>
                </AsTableCell>
              </AsTableRow>
            </AsTableHead>

            <AsTableBody emptyMessage={UIStrings.PAGES.SELECTION_MODAL.NO_PRIVILEGES}>
              {filteredData.slice(lowerBound, upperBound).map((privilege) => (
                <AsTableRow key={privilege.name} id={privilege.name}>
                  <AsTableCell className="select-column">
                    <AsCheckbox
                      onChange={() => onPrivilegeSelected(privilege)}
                      isChecked={isPrivilegeSelected(privilege.name)}>
                    </AsCheckbox>
                  </AsTableCell>
                  <AsTableCell>{privilege.name}</AsTableCell>
                  <AsTableCell>{privilege.description}</AsTableCell>
                </AsTableRow>))}
            </AsTableBody>
          </AsTable>

          <div className="modal-pagination">
            <AsPagination
              onChange={changePage}
              pageCount={pages > 0 ? pages : 1}
              currentPage={offsetPage}
            />
          </div>
        </div>

        <AsFooter className="as-footer-modal">
          <div className="as-btn-bar">
            <AsButton onClick={onModalClose}>{UIStrings.PAGES.SELECTION_MODAL.CANCEL}</AsButton>
            <AsButton variant="primary" onClick={confirmModal}>{UIStrings.PAGES.SELECTION_MODAL.CONFIRM}</AsButton>
          </div>
        </AsFooter>
      </AsModal>,
      document.body
    )}</>);
}
