
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  AsTile,
} from '@globalart/react-shared-components';
import {ExtJS, ValidationUtils} from '@globalart/astrum-ui-plugin';

import {
  ContentBody,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import Machine from './UsersFormMachine';
import UsersForm from './UsersForm';
import UsersReadOnly from './UsersReadOnly';
import UsersToken from './UsersToken';

import UIStrings from '../../../../constants/UIStrings';
import {parseIdParameter, fullName, sourceLabel} from './UsersHelper';

const {USERS: {FORM: LABELS}} = UIStrings;

export default function UsersDetails({itemId, onDone}) {
  const hasDeletePermission = ExtJS.checkPermission('astrum:users:delete');
  const canEdit = ExtJS.checkPermission('astrum:users:update');

  const {source, userId} = parseIdParameter(itemId);

  const [current, , service] = useMachine(Machine, {
    context: {
      pristineData: {source, userId},
      hasDeletePermission,
    },
    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone,
    },
    guards: {
      canDelete: () => hasDeletePermission,
    },
    devTools: true,
  });

  const {data: {firstName, lastName}, pristineData: {userId: id}} = current.context;

  const isEdit = ValidationUtils.notBlank(id);
  const showReadOnly = isEdit && !canEdit;
  const isPro = ExtJS.isProEdition();

  return <div className="as-user">
    <PageHeader>
      <PageTitle
          text={isEdit ? LABELS.EDIT_TILE(fullName({firstName, lastName})) : LABELS.CREATE_TITLE}
          description={LABELS.EDIT_DESCRIPTION(sourceLabel(source))}
      />
    </PageHeader>
    <ContentBody className="as-user-form">
      <AsTile>
        {showReadOnly
            ? <UsersReadOnly service={service} onDone={onDone}/>
            : <UsersForm service={service} onDone={onDone}/>
        }
      </AsTile>
      {isPro && isEdit &&
        <AsTile>
          <UsersToken service={service} />
        </AsTile>
      }
    </ContentBody>
  </div>;
}
