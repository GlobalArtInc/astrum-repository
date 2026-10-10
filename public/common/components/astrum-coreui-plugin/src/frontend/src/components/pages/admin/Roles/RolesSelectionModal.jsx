
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
} from '@globalart/react-shared-components';

import { ListMachineUtils } from '@globalart/astrum-ui-plugin';

import RoleSelectionMachine from './RoleSelectionMachine';
import { useMachine } from '@xstate/react';
import { createPortal } from 'react-dom';
import classNames from 'classnames';
import UIStrings from '../../../../constants/UIStrings';

const {ROLES: {SELECTION_MODAL}} = UIStrings;

export default function RolesSelectionModal({
    title,
    allRoles,
    onModalClose,
    selectedRoles,
    saveModal 
  }) {
  const [state, send] = useMachine(RoleSelectionMachine, {
    context: {
      offsetPage: 0,
      data: allRoles.map(item => ({
        ...item,
        isSelected: selectedRoles.includes(item.id)
      })),
      selectedRoles: selectedRoles,
      tempSelectedRoles: selectedRoles,
      filteredData: allRoles,
      numberOfRoles: allRoles.length,
    },
    devTools: true
  });

  const { offsetPage, tempSelectedRoles, numberOfRoles, filteredData, filter: filterText } = state.context;

  const onRoleSelected = (role) => send({ type: 'SELECT_ROLE', role: role });
  const isRoleSelected = (role) => { return tempSelectedRoles.includes(role) };
  const confirmModal = () => {
    onModalClose();
    send({ type: 'ON_CONFIRM' });
    saveModal(tempSelectedRoles);
  }

  const filter = (value) => send({ type: 'FILTER', filter: value });

  const rowsPerPage = 10;
  const pages = Math.ceil(numberOfRoles / rowsPerPage);
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
    'as-counter--active': tempSelectedRoles.length > 0
  });

  return (<>
    {createPortal(
      <AsModal variant="wide" onCancel={onModalClose}>
        <header className="as-modal-header">
          <AsH2 className="as-h2" id="modal-header-text">{title} Selection</AsH2>
        </header>
        <div className="as-modal__filter-box">
          <AsFilterInput className="as-modal__filter"
            placeholder="Filter"
            value={filterText}
            onChange={filter} />
          {SELECTION_MODAL.WILDCARD_TEXT}
        </div>
        <div className="as-modal-content">
          <AsTable className="modal-table">
            <AsTableHead>
              <AsTableRow>
                <AsTableCell className="select-column" onClick={sortBySelect} isSortable sortDir={selectSortDir}>Select</AsTableCell>
                <AsTableCell className="name-column" onClick={sortByName} isSortable sortDir={nameSortDir}>Name</AsTableCell>
                <AsTableCell onClick={sortByDescription} isSortable sortDir={descriptionSortDir}>Description</AsTableCell>
              </AsTableRow>
              <AsTableRow className="counter-row">
                <AsTableCell></AsTableCell>
                <AsTableCell></AsTableCell>
                <AsTableCell className="counter-cell">
                  <AsCounter className={counterClasses}>{tempSelectedRoles.length} Selected</AsCounter>
                </AsTableCell>
              </AsTableRow>
            </AsTableHead>

            <AsTableBody emptyMessage="No roles available">
              {filteredData.slice(lowerBound, upperBound).map((role) => (
                  <AsTableRow key={role.id} id={role.id}>
                    <AsTableCell className="select-column">
                      <AsCheckbox
                        onChange={() => onRoleSelected(role)}
                        isChecked={isRoleSelected(role.id)}>
                      </AsCheckbox>
                    </AsTableCell>
                    <AsTableCell>{role.name}</AsTableCell>
                    <AsTableCell>{role.description}</AsTableCell>
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
            <AsButton onClick={onModalClose}>Cancel</AsButton>
            <AsButton variant="primary" onClick={confirmModal}>Confirm</AsButton>
          </div>
        </AsFooter>
      </AsModal>,
      document.body
    )}</>);
}
