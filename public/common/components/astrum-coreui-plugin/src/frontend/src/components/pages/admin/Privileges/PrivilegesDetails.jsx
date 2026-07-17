
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {
  NxTile,
} from '@globalart/react-shared-components';
import {ExtJS, ValidationUtils} from '@globalart/astrum-ui-plugin';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Page
} from '@globalart/astrum-ui-plugin';

import Machine from './PrivilegesFormMachine';
import PrivilegesForm from './PrivilegesForm';
import PrivilegesReadOnly from './PrivilegesReadOnly';

import UIStrings from '../../../../constants/UIStrings';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';

const {PRIVILEGES: {FORM: LABELS}} = UIStrings;

export default function PrivilegesDetails() {
  const router = useRouter();
  const onDone = useCallback(() => router.stateService.go(ROUTE_NAMES.ADMIN.SECURITY.PRIVILEGES.LIST));
  const { params } = useCurrentStateAndParams();
  const itemId = params?.itemId;

  const hasDeletePermissions = ExtJS.checkPermission('astrum:privileges:delete');
  const hasEditPermissions = ExtJS.checkPermission('astrum:privileges:update');

  const [current, , service] = useMachine(Machine, {
    context: {
      pristineData: {
        name: itemId,
      }
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

  const {data: {readOnly}, pristineData} = current.context;

  const canEdit = hasEditPermissions && !readOnly;
  const isEdit = ValidationUtils.notBlank(itemId);
  const showReadOnly = isEdit && !canEdit;

  return <Page className="as-privileges">
    <PageHeader>
      <PageTitle
          text={isEdit ? LABELS.EDIT_TILE(pristineData.name || '') : LABELS.CREATE_TITLE}
          description={isEdit ? LABELS.EDIT_DESCRIPTION : null}
      />
    </PageHeader>
    <ContentBody className="as-privileges-form">
      <NxTile>
        {showReadOnly
            ? <PrivilegesReadOnly service={service} onDone={onDone}/>
            : <PrivilegesForm itemId={itemId} service={service} onDone={onDone}/>
        }
      </NxTile>
    </ContentBody>
  </Page>;
}
