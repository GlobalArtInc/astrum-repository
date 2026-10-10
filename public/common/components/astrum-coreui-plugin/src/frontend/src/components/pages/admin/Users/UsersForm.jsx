
import React from 'react';
import {useActor} from '@xstate/react';

import {
  FormUtils,
  ValidationUtils,
  ExtJS,
} from '@globalart/astrum-ui-plugin';
import {
  AsFormGroup,
  AsButton,
  AsCheckbox,
  AsFontAwesomeIcon,
  AsH2,
  AsModal,
  AsStatefulForm,
  AsStatefulTransferList,
  AsTextInput,
  AsTile,
  AsTooltip
} from '@globalart/astrum-react-shared-components';

import {faTrash} from '@fortawesome/free-solid-svg-icons';

import {isAnonymousUser, isExternalUser, isCurrentUser} from "./UsersHelper";

import UIStrings from '../../../../constants/UIStrings';

import ConfirmAdminPasswordForm from './ConfirmAdminPasswordForm';
import ConfirmNewPasswordForm from './ConfirmNewPasswordForm';

import './Users.scss';

const {USERS: {FORM: LABELS, MODAL}} = UIStrings;

export default function UsersForm({service, onDone}) {
  const [current, send] = useActor(service);

  const {
    data: {externalRoles = [], source, roles = []},
    pristineData: {userId},
    allRoles,
    hasDeletePermission,
  } = current.context;

  const isCreate = ValidationUtils.isBlank(userId);
  const isExternal = isExternalUser(source);
  const isEdit = !isCreate;
  const isChangingPassword = current.matches('changingPassword');
  const confirmingAdminPassword = current.matches('changingPassword.confirmAdminPassword');
  const confirmingNewPassword = current.matches('changingPassword.confirmNewPassword');
  const canChangePassword = ExtJS.checkPermission('astrum:*');

  const allRolesList = allRoles?.map((it) => ({id: it.id, displayName: it.name})) || [];

  const cancel = onDone;

  const confirmDelete = () => send({type: 'CONFIRM_DELETE'});

  const changePassword = () => {
    if (canChangePassword) {
      send({type: 'CHANGE_PASSWORD'});
    }
  }
  const cancelChangePassword = () => send({type: 'CANCEL_CHANGE_PASSWORD'});

  const showDeleteButton = isEdit && hasDeletePermission && !isExternal && !isAnonymousUser(userId) && !isCurrentUser(userId);
  const showChangePassword = isEdit && !isExternal && !isAnonymousUser(userId);

  return <>
    <AsStatefulForm
        {...FormUtils.formProps(current, send)}
        onCancel={cancel}
        additionalFooterBtns={showDeleteButton &&
        <AsButton variant="tertiary" onClick={confirmDelete} type="button">
          <AsFontAwesomeIcon icon={faTrash}/>
          <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
        </AsButton>
        }
    >
      <AsTile.Header>
        <AsTile.HeaderTitle>
          <AsH2>{LABELS.SECTIONS.SETUP}</AsH2>
        </AsTile.HeaderTitle>
        {showChangePassword &&
          <AsTile.HeaderActions className="as-users__actions">
            <AsTooltip
                title={!canChangePassword && UIStrings.PERMISSION_ERROR}
                placement="bottom"
            >
              <AsButton
                  onClick={changePassword}
                  variant="tertiary"
                  type="button"
                  className={!canChangePassword ? 'disabled' : ''}
              >
                {MODAL.CHANGE_PASSWORD}
              </AsButton>
            </AsTooltip>
          </AsTile.HeaderActions>
        }
      </AsTile.Header>
      <AsTile.Content>
        <AsFormGroup
            label={LABELS.ID.LABEL}
            sublabel={LABELS.ID.SUB_LABEL}
            isRequired
        >
          <AsTextInput
              {...FormUtils.fieldProps('userId', current)}
              onChange={FormUtils.handleUpdate('userId', send)}
              disabled={isEdit || isExternal}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.FIRST_NAME.LABEL} isRequired>
          <AsTextInput
              {...FormUtils.fieldProps('firstName', current)}
              onChange={FormUtils.handleUpdate('firstName', send)}
              disabled={isExternal}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.LAST_NAME.LABEL} isRequired>
          <AsTextInput
              {...FormUtils.fieldProps('lastName', current)}
              onChange={FormUtils.handleUpdate('lastName', send)}
              disabled={isExternal}
          />
        </AsFormGroup>
        <AsFormGroup
            label={LABELS.EMAIL.LABEL}
            sublabel={LABELS.EMAIL.SUB_LABEL}
            isRequired
        >
          <AsTextInput
              {...FormUtils.fieldProps('emailAddress', current)}
              onChange={FormUtils.handleUpdate('emailAddress', send)}
              disabled={isExternal}
          />
        </AsFormGroup>
        {!isExternal && isCreate && <>
          <AsFormGroup label={LABELS.PASSWORD.LABEL} isRequired>
            <AsTextInput
                type="password"
                autoComplete="new-password"
                {...FormUtils.fieldProps('password', current)}
                onChange={FormUtils.handleUpdate('password', send)}
            />
          </AsFormGroup>
          <AsFormGroup label={LABELS.CONFIRM_PASSWORD.LABEL} isRequired>
            <AsTextInput
                type="password"
                autoComplete="new-password"
                {...FormUtils.fieldProps('passwordConfirm', current)}
                onChange={FormUtils.handleUpdate('passwordConfirm', send)}
            />
          </AsFormGroup>
        </>}
        <AsFormGroup label={LABELS.STATUS.LABEL} isRequired>
          <AsCheckbox
              {...FormUtils.checkboxProps('status', current)}
              onChange={FormUtils.handleUpdate('status', send)}
              disabled={isExternal}
          >
            {LABELS.STATUS.OPTIONS.ACTIVE}
          </AsCheckbox>
        </AsFormGroup>

        <div className="as-form-group">
          <AsH2>{LABELS.SECTIONS.ROLES}</AsH2>
          <AsStatefulTransferList
              id="user-roles-select"
              availableItemsLabel={LABELS.ROLES.AVAILABLE}
              selectedItemsLabel={LABELS.ROLES.GRANTED}
              allItems={allRolesList}
              selectedItems={roles}
              onChange={FormUtils.handleUpdate('roles', send)}
              showMoveAll
          />
        </div>
        {isExternal &&
            <AsFormGroup
                label={LABELS.EXTERNAL_ROLES.LABEL}
                className="form-external-roles"
                isRequired
            >
              <AsTextInput
                  type="textarea"
                  id="externalRoles"
                  name="externalRoles"
                  value={externalRoles.join('\n')}
                  isPristine={true}
                  readOnly
                  disabled
              />
            </AsFormGroup>
        }
      </AsTile.Content>
    </AsStatefulForm>
    {isChangingPassword &&
      <AsModal
        onCancel={cancelChangePassword}
        aria-labelledby="modal-form-header"
        variant="narrow">
          {confirmingAdminPassword &&
            <ConfirmAdminPasswordForm
              actor={current.children.confirmAdminPasswordMachine}
              title={MODAL.CHANGE_PASSWORD}
              text={MODAL.TEXT}
              confirmLabel={MODAL.NEXT}
            />
          }
          {confirmingNewPassword &&
            <ConfirmNewPasswordForm actor={current.children.confirmNewPasswordMachine} />
          }
      </AsModal>}
  </>;
}
