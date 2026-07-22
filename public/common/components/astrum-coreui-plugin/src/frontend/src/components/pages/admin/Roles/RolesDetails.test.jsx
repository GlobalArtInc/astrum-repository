
import React from 'react';
import Axios from 'axios';
import {render, screen, within, waitFor, act, waitForElementToBeRemoved} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {when} from 'jest-when';

import {ExtJS} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import UIStrings from '../../../../constants/UIStrings';
import RolesDetails from './RolesDetails';
import {TYPES, URL} from './RolesHelper';
import {getRouter} from "../../../../routerConfig/routerConfig";
import {UIRouter, useCurrentStateAndParams} from "@uirouter/react";
import {ROUTE_NAMES} from "../../../../routerConfig/routeNames/routeNames";
const ADMIN = ROUTE_NAMES.ADMIN;

const { 
  ROLES: { 
    FORM: LABELS,
    FORM: { EXTERNAL_TYPE: { LDAP: { MORE_CHARACTERS, NO_RESULTS } } }
  },
  SETTINGS 
} = UIStrings;
const {rolesUrl, privilegesUrl, sourcesApi, getRolesUrl, defaultRolesUrl, singleRoleUrl, getLdapRolesUrl} = URL;

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    requestConfirmation: jest.fn(),
    checkPermission: jest.fn(),
    showErrorMessage: jest.fn(),
    showSuccessMessage: jest.fn(),
    state: jest.fn().mockReturnValue({
      getValue: jest.fn()
    })
  },
}));

const testRoleId = 'RoleId';
const testRoleName = 'Test Role Name';
const testRoleDescription = 'Test Role Description';

const ROLE = {
  id: testRoleId,
  name: testRoleName,
  description: testRoleDescription,
  privileges: ['as-all', 'as-blobstores-all', 'testRoleId'],
  roles: ['as-admin', 'TestRole', 'replication-role'],
};

const PRIVILEGES = [{
  description: 'All permissions',
  name: 'as-all',
  pattern: 'astrum:*',
  readOnly: true,
  type: 'wildcard',
}, {
  actions: ['ALL'],
  description: 'All permissions for Blobstores',
  domain: 'blobstores',
  name: 'as-blobstores-all',
  readOnly: true,
  type: 'application',
},
{
  description: 'All permissions for testing',
  domain: 'test',
  name: 'testRoleId',
  readOnly: true,
  type: 'application',
}];

const ROLES = {
  'as-admin': {
    description: 'Administrator Role',
    id: 'as-admin',
    name: 'as-admin',
    privileges: ['as-all'],
    roles: [],
    source: 'default'
  },
  'TestRole': {
    description: 'Test role',
    id: 'TestRole',
    name: 'Test Role name',
    privileges: ['as-healthcheck-read'],
    roles: [],
    source: 'default',
  },
  'replication-role': {
    description: 'Replication',
    id: 'replication-role',
    name: 'Replication role',
    privileges: ['as-replication-update'],
    roles: [],
    source: 'default',
  }
};

const CROWD_ROLES = [{
  description: 'crowd-administrators',
  id: 'crowd-administrators',
  name: 'crowd-administrators',
  privileges: [],
  roles: [],
  source: 'Crowd',
}];

const SOURCE_TYPES = {
  SAML: {id: 'SAML', name: 'SAML'},
  Crowd: {id: 'Crowd', name: 'Crowd'},
  LDAP: {id: 'LDAP', name: 'LDAP'},
};

const SOURCE_TYPES_RESP = {
  action: "coreui_ProprietaryRepositories",
  method: "readPossibleRepos",
  tid: 1,
  type: "rpc",
  result: {
    data: Object.values(SOURCE_TYPES),
    success: true,
  }
};

const selectors = {
  ...TestUtils.selectors,
  ...TestUtils.formSelectors,
  type: () => screen.queryByLabelText(LABELS.TYPE.LABEL),
  id: () => screen.queryByLabelText(LABELS.ID.LABEL),
  externalRoleType: () => screen.queryByLabelText(LABELS.EXTERNAL_TYPE.LABEL),
  mappedRole: () => screen.queryByLabelText(LABELS.MAPPED_ROLE.LABEL),
  name: () => screen.queryByLabelText(LABELS.NAME.LABEL),
  description: () => screen.queryByLabelText(LABELS.DESCRIPTION.LABEL),
  privileges: () => screen.queryByRole('group', {name: "Applied Privileges"}),
  roles: () => screen.queryByRole('group', {name: "Applied Roles"}),
  readOnly: {
    id: () => screen.getByText(LABELS.ID.LABEL).nextSibling,
    name: () => screen.getByText(LABELS.NAME.LABEL).nextSibling,
    description: () => screen.getByText(LABELS.DESCRIPTION.LABEL).nextSibling,
    privileges: () => screen.queryAllByRole('list')[0],
    roles: () => screen.queryAllByRole('list')[1],
  },
  roleModalButton: () => screen.getByRole('button', {name: 'Modify Applied Roles'}),
  privilegeModalButton: () => screen.getByRole('button', {name: 'Modify Applied Privileges'}),
  selectionModal: {
    modal: () => screen.queryByRole('dialog'),
    cancel: () => within(selectors.selectionModal.modal()).getByRole('button', {name: 'Cancel'}),
    confirmButton: () => within(selectors.selectionModal.modal()).getByRole('button', {name: 'Confirm'}),
    filter: () => screen.queryAllByPlaceholderText('Filter')[1],
  },
  cancelButton: () => screen.getByText(SETTINGS.CANCEL_BUTTON_LABEL),
  saveButton: () => screen.getByText(SETTINGS.SAVE_BUTTON_LABEL),
  deleteButton: () => screen.getByText(SETTINGS.DELETE_BUTTON_LABEL),
};

const clickOnCheckboxes = (checkboxes) => checkboxes.forEach((it) => userEvent.click(it));

let stateServiceGoMock;

jest.mock('@uirouter/react', () => ({
  ...jest.requireActual('@uirouter/react'),
  useCurrentStateAndParams: jest.fn(),
  useRouter: () =>({
    stateService: {
      go: stateServiceGoMock,
    }
  })
}));

describe('RolesDetails', function() {
  const CONFIRM = Promise.resolve();
  let router;

  function renderEditView(itemId) {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: ADMIN.SECURITY.ROLES.EDIT},
      params: {itemId}
    });

    return render(
        <UIRouter router={router}>
          <RolesDetails/>
        </UIRouter>
    );
  }

  function renderCreateView() {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: ADMIN.SECURITY.ROLES.CREATE},
      params: {}
    });

    return render(
        <UIRouter router={router}>
          <RolesDetails/>
        </UIRouter>
    );
  }

  beforeEach(() => {
    useCurrentStateAndParams.mockReset()
    router = getRouter();
    stateServiceGoMock = jest.fn();
    ExtJS.state = jest.fn().mockReturnValue({
      getValue: jest.fn(),
    });
    when(ExtJS.state().getValue).calledWith('astrum.react.roles.modal.enabled').mockReturnValue(true);
    when(ExtJS.state().getValue).calledWith('astrum.react.privileges.modal.enabled').mockReturnValue(true);
    when(Axios.get).calledWith(defaultRolesUrl).mockResolvedValue({data: Object.values(ROLES)});
    when(Axios.get).calledWith(privilegesUrl).mockResolvedValue({data: PRIVILEGES});
    when(Axios.get).calledWith(singleRoleUrl(testRoleId)).mockResolvedValue({data: {...ROLE, readOnly: false}});
    when(Axios.post).calledWith('service/extdirect', expect.objectContaining(sourcesApi))
        .mockResolvedValue({data: SOURCE_TYPES_RESP});
    ExtJS.checkPermission.mockReturnValue(true);
  });

  it('redirects to 404 page if itemId is not provided for edit route', function() {
    renderEditView('');
    expect(stateServiceGoMock).toBeCalledWith(ROUTE_NAMES.MISSING_ROUTE);
  });

  it('renders the resolved data', async function() {
    const {id, name, queryLoadingMask, description, privileges, roles, saveButton} = selectors;
    renderEditView(testRoleId);
    await waitForElementToBeRemoved(queryLoadingMask());

    expect(id()).toHaveValue(testRoleId);
    expect(id()).toBeDisabled();
    expect(name()).toHaveValue(ROLE.name);
    expect(description()).toHaveValue(ROLE.description);

    ROLE.privileges.forEach(it => {
      expect(privileges()).toHaveTextContent(it);
    });
    ROLE.roles.forEach(it => {
      expect(roles()).toHaveTextContent(ROLES[it].name);
    });

    userEvent.click(saveButton());
    expect(selectors.queryFormError(TestUtils.NO_CHANGES_MESSAGE)).toBeInTheDocument();
  });

  it('renders load error message', async function() {
    const message = 'Load error message!';
    const {queryLoadingMask} = selectors;

    Axios.get.mockReturnValue(Promise.reject({message}));

    renderEditView(testRoleId);
    await waitForElementToBeRemoved(queryLoadingMask());

    expect(screen.getByRole('alert')).toHaveTextContent(message);
  });

  it('requires the ID and Name fields when creating a new internal role', async function() {
    const {type, id, name, queryLoadingMask, description, privileges, roles, saveButton} = selectors;
    renderCreateView();
    await waitForElementToBeRemoved(queryLoadingMask());

    expect(id()).not.toBeInTheDocument();
    expect(name()).not.toBeInTheDocument();
    expect(description()).not.toBeInTheDocument();
    expect(roles()).not.toBeInTheDocument();
    expect(privileges()).not.toBeInTheDocument();

    userEvent.selectOptions(type(), TYPES.INTERNAL);
    expect(name()).toBeInTheDocument();

    userEvent.click(saveButton());
    expect(selectors.queryFormError(TestUtils.NO_CHANGES_MESSAGE)).toBeInTheDocument();

    userEvent.clear(id());
    await TestUtils.changeField(id, testRoleId);
    expect(selectors.queryFormError(TestUtils.VALIDATION_ERRORS_MESSAGE)).toBeInTheDocument();

    await TestUtils.changeField(name, testRoleName);
    expect(selectors.queryFormError()).not.toBeInTheDocument();
  });

  it('fires onDone when cancelled', async function() {
    const {queryLoadingMask, cancelButton} = selectors;
    renderCreateView();
    await waitForElementToBeRemoved(queryLoadingMask());

    userEvent.click(cancelButton());

    // expect onDone is called
    expect(stateServiceGoMock).toBeCalledWith(ADMIN.SECURITY.ROLES.LIST);
  });

  it('requests confirmation when delete is requested', async function() {
    const {queryLoadingMask, deleteButton} = selectors;
    Axios.delete.mockReturnValue(Promise.resolve(null));

    renderEditView(testRoleId);
    await waitForElementToBeRemoved(queryLoadingMask());

    ExtJS.requestConfirmation.mockReturnValue(CONFIRM);
    userEvent.click(deleteButton());

    await waitFor(() => expect(Axios.delete).toBeCalledWith(singleRoleUrl(testRoleId)));
    expect(ExtJS.showSuccessMessage).toBeCalled();

    // expect onDone is called
    expect(stateServiceGoMock).toBeCalledWith(ADMIN.SECURITY.ROLES.LIST);
  });

  it('creates internal role', async function() {
    const {type, id, name, queryLoadingMask, description, saveButton, roleModalButton, privilegeModalButton,
      selectionModal: {modal, confirmButton}} = selectors;

    when(Axios.post).calledWith(rolesUrl, ROLE).mockResolvedValue({data: {}});

    renderCreateView();
    await waitForElementToBeRemoved(queryLoadingMask());

    userEvent.selectOptions(type(), TYPES.INTERNAL);
    await TestUtils.changeField(id, testRoleId);
    await TestUtils.changeField(name, testRoleName);
    await TestUtils.changeField(description, testRoleDescription);

    userEvent.click(privilegeModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    userEvent.click(roleModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    expect(saveButton()).not.toHaveClass('disabled');
    userEvent.click(saveButton());

    await waitFor(() => expect(Axios.post).toHaveBeenCalledWith(rolesUrl, ROLE));
    expect(NX.Messages.success).toHaveBeenCalledWith(UIStrings.SAVE_SUCCESS);
  });

  it('creates a ldap external role', async function() {
    const {type, name, queryLoadingMask, description, saveButton, externalRoleType, mappedRole, roleModalButton,
      privilegeModalButton, selectionModal: {modal, confirmButton}} = selectors;
    const ldapType = SOURCE_TYPES.LDAP.id;
    const externalRole = {...ROLE, id: 'testLDAPRoleId'};
    const combobox = () => container.querySelector('.nx-combobox__alert');

    when(Axios.post).calledWith(rolesUrl, externalRole).mockResolvedValue({data: {}});
    when(ExtJS.state().getValue).calledWith('astrum.ldap.mapped.role.query.character.limit').mockReturnValue(3);

    const {container} = renderCreateView();
    await waitForElementToBeRemoved(queryLoadingMask());

    userEvent.selectOptions(type(), TYPES.EXTERNAL);
    userEvent.selectOptions(externalRoleType(), 'LDAP');

    await TestUtils.changeField(mappedRole, 't');
    expect(combobox()).toHaveTextContent(MORE_CHARACTERS(3));

    await TestUtils.changeField(mappedRole, 'testLDAPRoleId');
    expect(combobox()).toHaveTextContent(NO_RESULTS);

    await waitFor(() => expect(Axios.get).toHaveBeenCalledWith(getLdapRolesUrl('testLDAPRoleId', ldapType)));

    await TestUtils.changeField(name, testRoleName);
    await TestUtils.changeField(description, testRoleDescription);

    userEvent.click(privilegeModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    userEvent.click(roleModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    expect(saveButton()).not.toHaveClass('disabled');
    userEvent.click(saveButton());

    await waitFor(() => expect(Axios.post).toHaveBeenCalledWith(rolesUrl, externalRole));
    expect(NX.Messages.success).toHaveBeenCalledWith(UIStrings.SAVE_SUCCESS);
  });

  it('creates a crowd external role', async function() {
    const {type, name, queryLoadingMask, description, saveButton, externalRoleType, mappedRole, roleModalButton,
      privilegeModalButton, selectionModal: {modal, confirmButton}} = selectors;
    const crowdType = SOURCE_TYPES.Crowd.id;
    const testCrowdRoleId = CROWD_ROLES[0].id;
    const externalRole = {...ROLE, id: testCrowdRoleId};

    when(Axios.post).calledWith(rolesUrl, externalRole).mockResolvedValue({data: {}});

    renderCreateView();
    await waitForElementToBeRemoved(queryLoadingMask());

    userEvent.selectOptions(type(), TYPES.EXTERNAL);
    userEvent.selectOptions(externalRoleType(), crowdType);

    await TestUtils.changeField(mappedRole, testCrowdRoleId);
    await TestUtils.changeField(name, testRoleName);
    await TestUtils.changeField(description, testRoleDescription);

    userEvent.click(privilegeModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    userEvent.click(roleModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    expect(saveButton()).not.toHaveClass('disabled');
    userEvent.click(saveButton());

    await waitFor(() => expect(Axios.post).toHaveBeenCalledWith(rolesUrl, externalRole));
    expect(NX.Messages.success).toHaveBeenCalledWith(UIStrings.SAVE_SUCCESS);
  });

  it('updates', async function() {
    const {name, description, queryLoadingMask, saveButton, roleModalButton,privilegeModalButton,
      selectionModal: {modal, confirmButton}} = selectors;
    const data = {
      id: 'RoleId',
      name: 'Updated name',
      description: 'Updated description',
      privileges: ['as-all', 'as-blobstores-all', 'testRoleId'],
      roles: ['as-admin', 'TestRole', 'replication-role'],
    };

    Axios.put.mockReturnValue(Promise.resolve());

    renderEditView(testRoleId);
    await waitForElementToBeRemoved(queryLoadingMask());

    await TestUtils.changeField(name, data.name);
    await TestUtils.changeField(description, data.description);

    userEvent.click(privilegeModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    userEvent.click(roleModalButton());
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    clickOnCheckboxes(within(modal()).getAllByRole('checkbox'));
    await act(async () => userEvent.click(confirmButton()));

    expect(saveButton()).not.toHaveClass('disabled');
    userEvent.click(saveButton());

    await waitFor(() => expect(Axios.put).toHaveBeenCalledWith(singleRoleUrl(testRoleId), {
      id: testRoleId,
      readOnly: false,
      ...data,
    }));
    expect(NX.Messages.success).toHaveBeenCalledWith(UIStrings.SAVE_SUCCESS);
  });

  it('shows save API errors', async function() {
    const message = "Use a unique roleId";
    const {type, id, name, queryLoadingMask, description, saveButton} = selectors;

    when(Axios.post).calledWith(rolesUrl, expect.objectContaining({name: testRoleName})).mockRejectedValue({
      response: {
        data: message,
      }
    });

    renderCreateView();
    await waitForElementToBeRemoved(queryLoadingMask());

    userEvent.selectOptions(type(), TYPES.INTERNAL);
    await TestUtils.changeField(id, testRoleId);
    await TestUtils.changeField(name, testRoleName);
    await TestUtils.changeField(description, testRoleDescription);

    userEvent.click(saveButton());
    await waitForElementToBeRemoved(selectors.querySavingMask());

    expect(screen.getByText(new RegExp(message))).toBeInTheDocument();
  });

  describe('Read Only Mode', function() {
    const shouldSeeDetailsInReadOnlyMode = () => {
      const {readOnly: {id, name, description, privileges, roles}} = selectors;

      expect(id()).toHaveTextContent(testRoleId);
      expect(name()).toHaveTextContent(testRoleName);
      expect(description()).toHaveTextContent(testRoleDescription);

      ROLE.privileges.forEach(it => {
        expect(privileges()).toHaveTextContent(it);
      });
      ROLE.roles.forEach(it => {
        expect(roles()).toHaveTextContent(ROLES[it].name);
      });
    };

    it('renders default role in Read Only Mode', async () => {
      const {queryLoadingMask} = selectors;
      const warning = () => screen.getByText(LABELS.DEFAULT_ROLE_WARNING);

      when(Axios.get).calledWith(singleRoleUrl(testRoleId)).mockResolvedValue({
        data: {...ROLE, readOnly: true}
      });

      renderEditView(testRoleId);
      await waitForElementToBeRemoved(queryLoadingMask());

      expect(warning()).toBeInTheDocument();
      shouldSeeDetailsInReadOnlyMode();
    });

    it('renders role details without edit permissions', async () => {
      const {queryLoadingMask} = selectors;
      const warning = () => screen.getByText(SETTINGS.READ_ONLY.WARNING);

      when(ExtJS.checkPermission).calledWith('astrum:roles:update').mockReturnValue(false);

      renderEditView(testRoleId);
      await waitForElementToBeRemoved(queryLoadingMask());

      expect(warning()).toBeInTheDocument();
      shouldSeeDetailsInReadOnlyMode();
    });
  });

  describe('Roles Selection Modal', function () {
    it('opens the modal when button is clicked', async function () {
      const { queryLoadingMask, roleModalButton, selectionModal: { modal, cancel } } = selectors;

      renderEditView(testRoleId);
      await waitForElementToBeRemoved(queryLoadingMask());

      expect(modal()).not.toBeInTheDocument();

      userEvent.click(roleModalButton());
      expect(modal()).toBeInTheDocument();

      userEvent.click(cancel());
      expect(modal()).not.toBeInTheDocument();
    });

    it('add the role to the transfer list once the modal is confirmed', async function () {
      const {
        queryLoadingMask,
        type,
        id,
        name,
        description,
        roleModalButton,
        roles,
        selectionModal: { modal, confirmButton }
      } = selectors;

      renderCreateView();
      await waitForElementToBeRemoved(queryLoadingMask());

      userEvent.selectOptions(type(), TYPES.INTERNAL);
      await TestUtils.changeField(id, testRoleId);
      await TestUtils.changeField(name, testRoleName);
      await TestUtils.changeField(description, testRoleDescription);

      expect(modal()).not.toBeInTheDocument();
      userEvent.click(roleModalButton());
      expect(modal()).toBeInTheDocument();

      expect(roles()).toHaveTextContent('0 Items Available');

      const tableRow = (index) => modal().querySelectorAll('tbody tr')[index];
      expect(tableRow(0).cells[1]).toHaveTextContent('as-admin');
      expect(modal().querySelectorAll('thead tr')[1].cells[2].textContent).toBe('0 Selected');

      userEvent.click(within(modal()).getAllByRole('checkbox')[0]);
      expect(modal().querySelectorAll('thead tr')[1].cells[2].textContent).toBe('1 Selected');

      await act(async () => userEvent.click(confirmButton()));
      expect(roles()).toHaveTextContent('1 Item Available');
    });
  });

  describe('Privileges Selection Modal', function () {
    it('opens the modal when button is clicked', async function () {
      const { queryLoadingMask, privilegeModalButton, selectionModal: { modal, cancel } } = selectors;

      renderEditView(testRoleId);
      await waitForElementToBeRemoved(queryLoadingMask());

      expect(modal()).not.toBeInTheDocument();

      userEvent.click(privilegeModalButton());
      expect(modal()).toBeInTheDocument();

      userEvent.click(cancel());
      expect(modal()).not.toBeInTheDocument();
    });

    it('can have the same name as the role', async function () {
      const { queryLoadingMask, privilegeModalButton, selectionModal: { modal, cancel, filter } } = selectors;

      renderEditView(testRoleId);
      await waitForElementToBeRemoved(queryLoadingMask());

      expect(modal()).not.toBeInTheDocument();

      userEvent.click(privilegeModalButton());
      expect(modal()).toBeInTheDocument();

      const tableRow = (index) => modal().querySelectorAll('tbody tr')[index];
      expect(tableRow(2).cells[1]).toHaveTextContent('testRoleId');

      userEvent.click(cancel());
      expect(modal()).not.toBeInTheDocument();
    });

    it('add the role to the transfer list once the modal is confirmed', async function () {
      const {
        queryLoadingMask,
        type,
        id,
        name,
        description,
        privilegeModalButton,
        privileges,
        selectionModal: { modal, confirmButton }
      } = selectors;

      renderCreateView();
      await waitForElementToBeRemoved(queryLoadingMask());

      userEvent.selectOptions(type(), TYPES.INTERNAL);
      await TestUtils.changeField(id, testRoleId);
      await TestUtils.changeField(name, testRoleName);
      await TestUtils.changeField(description, testRoleDescription);

      expect(modal()).not.toBeInTheDocument();
      userEvent.click(privilegeModalButton());
      expect(modal()).toBeInTheDocument();

      expect(privileges()).toHaveTextContent('0 Items Available');

      const tableRow = (index) => modal().querySelectorAll('tbody tr')[index];
      expect(tableRow(0).cells[1]).toHaveTextContent('as-all');
      expect(modal().querySelectorAll('thead tr')[1].cells[2].textContent).toBe('0 Selected');

      userEvent.click(within(modal()).getAllByRole('checkbox')[0]);
      expect(modal().querySelectorAll('thead tr')[1].cells[2].textContent).toBe('1 Selected');

      await act(async () => userEvent.click(confirmButton()));
      expect(privileges()).toHaveTextContent('1 Item Available');
    });
  });
});
