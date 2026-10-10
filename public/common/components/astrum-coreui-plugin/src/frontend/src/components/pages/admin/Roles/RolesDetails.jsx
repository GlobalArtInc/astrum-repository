
import React, {useCallback, useEffect} from 'react';
import {useMachine} from '@xstate/react';

import {
  AsTile,
} from '@globalart/react-shared-components';
import {
  ContentBody,
  ExtJS,
  PageHeader,
  PageTitle,
  ValidationUtils,
  Page,
} from '@globalart/astrum-ui-plugin';

import Machine from './RolesFormMachine';
import RolesForm from './RolesForm';
import RolesReadOnly from './RolesReadOnly';

import UIStrings from '../../../../constants/UIStrings';
import {useCurrentStateAndParams, useRouter} from "@uirouter/react";
import {ROUTE_NAMES} from "../../../../routerConfig/routeNames/routeNames";

const {ROLES: {FORM: LABELS}} = UIStrings;
const ADMIN = ROUTE_NAMES.ADMIN;

export default function RolesDetails() {
  const router = useRouter();
  const {state: routerState, params} = useCurrentStateAndParams();
  const onDone = useCallback(() => router.stateService.go(ADMIN.SECURITY.ROLES.LIST), [router]);
  const itemId = params?.itemId;
  const isCreate = routerState.name === ADMIN.SECURITY.ROLES.CREATE;

  useEffect(() => {
    // we should not render edit form if itemId is not provided
    if (!isCreate && !itemId) {
      router.stateService.go(ROUTE_NAMES.MISSING_ROUTE)
    }

  }, [itemId]);

  const hasDeletePermissions = ExtJS.checkPermission('astrum:roles:delete');
  const hasEditPermissions = ExtJS.checkPermission('astrum:roles:update');
  const ldapQueryCharacterLimit = ExtJS.state().getValue('astrum.ldap.mapped.role.query.character.limit');

  const [current, , service] = useMachine(Machine, {
    context: {
      externalRolesRef: null,
      pristineData: {
        id: decodeURIComponent(itemId),
      },
      ldapQueryCharacterLimit: ldapQueryCharacterLimit,
      isCreate,
    },
    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone,
    },
    guards: {
      canDelete: () => hasDeletePermissions,
    },
    devTools: true,
  });

  const {data: {readOnly}, pristineData: {id, name}} = current.context;

  const canEdit = hasEditPermissions && !readOnly;
  const showReadOnly = !isCreate && !canEdit;

  return <Page className="as-roles">
    <PageHeader>
      <PageTitle
          text={isCreate ? LABELS.CREATE_TITLE : LABELS.EDIT_TILE(name || '')}
          description={isCreate ? null : LABELS.EDIT_DESCRIPTION}
      />
    </PageHeader>
    <ContentBody className="as-roles-form">
      <AsTile>
        <AsTile.Content>
          {showReadOnly
              ? <RolesReadOnly service={service} onDone={onDone}/>
              : <RolesForm
                  roleId={id}
                  service={service}
                  onDone={onDone}
                  isCreate={isCreate}
              />
          }
        </AsTile.Content>
      </AsTile>
    </ContentBody>
  </Page>;
}
