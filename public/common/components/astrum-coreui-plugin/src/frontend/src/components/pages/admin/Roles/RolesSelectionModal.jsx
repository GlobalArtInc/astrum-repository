
import React from 'react';

import {
  NxModal,
  NxH2,
  NxButton,
  NxCheckbox,
  NxPagination,
  NxTable,
  NxTableHead,
  NxTableRow,
  NxTableCell,
  NxTableBody,
  NxFilterInput,
  NxCounter,
  NxFooter
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

  const counterClasses = classNames('nx-pull-right', {
    'nx-counter--active': tempSelectedRoles.length > 0
  });

  return (<>
    {createPortal(
      <NxModal variant="wide" onCancel={onModalClose}>
        <header className="nx-modal-header">
          <NxH2 className="nx-h2" id="modal-header-text">{title} Selection</NxH2>
        </header>
        <div className="nx-modal__filter-box">
          <NxFilterInput className="nx-modal__filter"
            placeholder="Filter"
            value={filterText}
            onChange={filter} />
          {SELECTION_MODAL.WILDCARD_TEXT}
        </div>
        <div className="nx-modal-content">
          <NxTable className="modal-table">
            <NxTableHead>
              <NxTableRow>
                <NxTableCell className="select-column" onClick={sortBySelect} isSortable sortDir={selectSortDir}>Select</NxTableCell>
                <NxTableCell className="name-column" onClick={sortByName} isSortable sortDir={nameSortDir}>Name</NxTableCell>
                <NxTableCell onClick={sortByDescription} isSortable sortDir={descriptionSortDir}>Description</NxTableCell>
              </NxTableRow>
              <NxTableRow className="counter-row">
                <NxTableCell></NxTableCell>
                <NxTableCell></NxTableCell>
                <NxTableCell className="counter-cell">
                  <NxCounter className={counterClasses}>{tempSelectedRoles.length} Selected</NxCounter>
                </NxTableCell>
              </NxTableRow>
            </NxTableHead>

            <NxTableBody emptyMessage="No roles available">
              {filteredData.slice(lowerBound, upperBound).map((role) => (
                  <NxTableRow key={role.id} id={role.id}>
                    <NxTableCell className="select-column">
                      <NxCheckbox
                        onChange={() => onRoleSelected(role)}
                        isChecked={isRoleSelected(role.id)}>
                      </NxCheckbox>
                    </NxTableCell>
                    <NxTableCell>{role.name}</NxTableCell>
                    <NxTableCell>{role.description}</NxTableCell>
                  </NxTableRow>))}
            </NxTableBody>
          </NxTable>

          <div className="modal-pagination">
            <NxPagination
              onChange={changePage}
              pageCount={pages > 0 ? pages : 1}
              currentPage={offsetPage}
            />
          </div>
        </div>

        <NxFooter className="nx-footer-modal">
          <div className="nx-btn-bar">
            <NxButton onClick={onModalClose}>Cancel</NxButton>
            <NxButton variant="primary" onClick={confirmModal}>Confirm</NxButton>
          </div>
        </NxFooter>
      </NxModal>,
      document.body
    )}</>);
}
