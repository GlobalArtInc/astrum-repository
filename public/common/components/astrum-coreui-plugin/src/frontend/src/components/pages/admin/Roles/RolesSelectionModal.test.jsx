

import React from 'react';

import RolesSelectionModal from "./RolesSelectionModal";
import { render, within, screen } from '@testing-library/react';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import userEvent from '@testing-library/user-event';

const ROLES = [
    {
        description: 'Administrator Role',
        id: 'as-admin',
        name: 'as-admin',
        privileges: ['as-all'],
        roles: [],
        source: 'default'
    },
    {
        name: "as-analytics-all",
        description: "All permissions for Analytics",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-apikey-all",
        description: "All permissions for APIKey",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-blobstores-all",
        description: "All permissions for Blobstores",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-blobstores-create",
        description: "Create permission for Blobstores",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-blobstores-update",
        description: "Update permission for Blobstores",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-bundles-all",
        description: "All permissions for Bundles",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-bundles-read",
        description: "Read permission for Bundles",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-capabilities-all",
        description: "All permissions for Capabilities",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-capabilities-create",
        description: "Create permission for Capabilities",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-capabilities-delete",
        description: "Delete permission for Capabilities",
        privileges: ['as-healthcheck-read'],
        roles: [],
    },
    {
        name: "as-capabilities-read",
        description: "Read permission for Capabilities",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-capabilities-update",
        description: "Update permission for Capabilities",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-component-upload",
        description: "Upload component permission",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-crowd-all",
        description: "All permissions for Crowd",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-crowd-read",
        description: "Read permission for Crowd",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-crowd-update",
        description: "Update permission for Crowd",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-datastores-all",
        description: "All permissions for Datastores",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-datastores-create",
        description: "Create permission for Datastores",
        privileges: ['as-all'],
        roles: [],
    },
    {
        name: "as-datastores-delete",
        description: "Delete permission for Datastores",
        privileges: ['as-all'],
        roles: [],
    },
];

const selectors = {
    filter: () => screen.queryByPlaceholderText('Filter'),
    modal: () => screen.getByRole('dialog'),
};

describe('RolesSelectionModal', function () {

    function renderModal(roles) {
        return render(<RolesSelectionModal title={"Roles"} allRoles={roles} onModalClose={() => { }} selectedRoles={[]} />);
    }

    it('sorts the selected above the rest of the roles alphabetically when sorting select ascending', async function () {
        const { modal } = selectors;
        renderModal(ROLES);

        const tableRow = (index) => modal().querySelectorAll('tbody tr')[index];
        expect(tableRow(0).cells[1]).toHaveTextContent('as-admin');
        expect(tableRow(0).cells[2]).toHaveTextContent('Administrator Role');
        expect(tableRow(1).cells[1]).toHaveTextContent('as-analytics-all');
        expect(tableRow(2).cells[1]).toHaveTextContent('as-apikey-all');

        const tableRowHeader = (index) => modal().querySelectorAll('thead tr')[index];
        expect(tableRowHeader(1).cells[2].textContent).toBe('0 Selected');

        const checkboxes = (index) => within(modal()).queryAllByRole('checkbox')[index];
        userEvent.click(checkboxes(2));

        userEvent.click(tableRowHeader(0).cells[0]);

        expect(tableRowHeader(1).cells[2].textContent).toBe('1 Selected');
        expect(tableRow(0).cells[1]).toHaveTextContent('as-apikey-all');

        userEvent.click(checkboxes(0));
        userEvent.click(tableRowHeader(0).cells[2]);

        expect(tableRowHeader(1).cells[2].textContent).toBe('0 Selected');
        expect(tableRow(0).cells[1]).toHaveTextContent('as-admin');
        expect(tableRow(2).cells[1]).toHaveTextContent('as-apikey-all');
    });

    it('filters out roles based on the text the user inputs', async function () {
        const { filter, modal } = selectors;
        renderModal(ROLES);

        expect(modal()).toBeInTheDocument();

        const tableRow = (index) => modal().querySelectorAll('tbody tr')[index];
        expect(tableRow(0).cells[1]).toHaveTextContent('as-admin');
        expect(tableRow(1).cells[1]).toHaveTextContent('as-analytics-all');
        expect(tableRow(2).cells[1]).toHaveTextContent('as-apikey-all');

        await TestUtils.changeField(filter, 'ana');
        expect(tableRow(0).cells[1]).toHaveTextContent('as-analytics-all');

        await TestUtils.changeField(filter, 'z');
        expect(modal().querySelectorAll('tbody tr[id]').length).toBe(0);

        await TestUtils.changeField(filter, '');
        expect(tableRow(0).cells[1]).toHaveTextContent('as-admin');
    });

    it('changes to page 1 when filtering', async function () {
        const { filter, modal } = selectors;
        renderModal(ROLES);

        const navigation = within(modal()).getByRole('navigation', { name: 'pagination' });
        const page2 = within(navigation).getByText('2');
        userEvent.click(page2);
        expect(page2).toHaveAttribute('aria-current', 'page');

        await TestUtils.changeField(filter, 'n');
        const page1 = within(navigation).getByText('1');
        expect(page1).toHaveAttribute('aria-current', 'page');
    });

    it('pagination display 10 roles at a time', async function () {
        const { modal } = selectors;
        renderModal(ROLES);
        expect(modal()).toBeInTheDocument();

        const navigation = within(modal()).getByRole('navigation', { name: 'pagination' });
        const pages = (index) => within(navigation).getAllByRole('button')[index];

        const tableRow = (index) => modal().querySelectorAll('tbody tr')[index];
        expect(tableRow(0).cells[1]).toHaveTextContent('as-admin');

        expect(pages(0)).toHaveAttribute('aria-current', 'page');
        expect(pages(0)).toHaveTextContent('1');

        const page1 = within(navigation).getByText('1');
        expect(page1).toHaveAttribute('aria-current', 'page');

        const page2 = within(navigation).getByText('2');
        userEvent.click(page2);

        expect(page2).toHaveAttribute('aria-current', 'page');
        expect(tableRow(0).cells[1]).toHaveTextContent('as-capabilities-delete');
    });
})
