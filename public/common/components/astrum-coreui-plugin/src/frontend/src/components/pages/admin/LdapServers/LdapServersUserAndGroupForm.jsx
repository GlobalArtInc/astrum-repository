
import React from 'react';
import {useActor} from '@xstate/react';
import {
  AsTile,
  AsFormGroup,
  AsFormSelect,
  AsH2,
  AsTextInput,
  AsFieldset,
  AsCheckbox,
  AsStatefulForm,
  AsButton,
} from '@globalart/astrum-react-shared-components';
import {FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../constants/UIStrings';
import LdapServersModalPassword from './LdapServersModalPassword';
import {isDynamicGroup, isStaticGroup} from './LdapServersHelper';
import LdapVerifyUserMappingModal from './LdapVerifyUserMapping/LdapVerifyUserMappingModal';
import LdapVerifyLoginModal from './LdapVerifyLogin/LdapVerifyLoginModal';

const {
  LDAP_SERVERS: {FORM: LABELS},
} = UIStrings;

export default function LdapServerUserAndGroupForm({actor, onDone}) {
  const [state, send] = useActor(actor);
  const {
    templates,
    template,
    data: {ldapGroupsAsRoles, groupType},
    data,
    validationErrors,
  } = state.context;

  const isTemplateSelected = ValidationUtils.notBlank(template);

  const askingPassword = state.matches('askingPassword');
  const showingVerifyLoginModal = state.matches('showingVerifyLoginModal');
  const showingVerifyUserMappingModal = state.matches('showingVerifyUserMappingModal');

  const isInvalid = FormUtils.isInvalid(validationErrors);

  const updateTemplate = (value) =>
    send({type: 'UPDATE_TEMPLATE', value});
  const setSystemPassword = (value) => send({type: 'SET_PASSWORD', name: 'authPassword', value})
  const verifyLogin = () => send({type: 'VERIFY_LOGIN'});
  const verifyUserMapping = () => send({type: 'VERIFY_USER_MAPPING'});
  const cancel = () => send({type: 'CANCEL'});

  return (
    <AsTile.Content>
      <AsH2>{LABELS.CONFIGURATION}</AsH2>
      <AsStatefulForm 
        {...FormUtils.formProps(state, send)} 
        onCancel={onDone}
        additionalFooterBtns={
          <>
            <AsButton
              className={isInvalid ? 'disabled' : ''}
              onClick={verifyUserMapping}
              type="button"
            >
              {LABELS.VERIFY_USER_MAPPING_BUTTON}
            </AsButton>
            <AsButton
              className={isInvalid ? 'disabled' : ''}
              onClick={verifyLogin}
              type="button"
            >
              {LABELS.VERIFY_LOGIN_BUTTON}
            </AsButton>
          </>
        }
      >
        <AsFormGroup label={LABELS.TEMPLATE.LABEL}>
          <AsFormSelect onChange={updateTemplate} value={template}>
            <option value="" disabled={isTemplateSelected} />
            {templates.map(({name}) => (
              <option value={name} key={name}>
                {name}
              </option>
            ))}
          </AsFormSelect>
        </AsFormGroup>
        <AsFormGroup
          label={LABELS.USER_RELATIVE_DN.LABEL}
          sublabel={LABELS.USER_RELATIVE_DN.SUB_LABEL}
        >
          <AsTextInput
            {...FormUtils.fieldProps('userBaseDn', state)}
            onChange={FormUtils.handleUpdate('userBaseDn', send)}
            className="as-text-input--long"
          />
        </AsFormGroup>
        <AsFieldset label={LABELS.USER_SUBTREE.LABEL}>
          <AsCheckbox
            {...FormUtils.checkboxProps('userSubtree', state)}
            onChange={FormUtils.handleUpdate('userSubtree', send)}
          >
            {LABELS.USER_SUBTREE.SUB_LABEL}
          </AsCheckbox>
        </AsFieldset>
        <AsFormGroup
          label={LABELS.OBJECT_CLASS.LABEL}
          sublabel={LABELS.OBJECT_CLASS.SUB_LABEL}
          isRequired
        >
          <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('userObjectClass', state)}
            onChange={FormUtils.handleUpdate('userObjectClass', send)}
          />
        </AsFormGroup>
        <AsFormGroup
          label={LABELS.USER_FILTER.LABEL}
          sublabel={LABELS.USER_FILTER.SUB_LABEL}
        >
          <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('userLdapFilter', state)}
            onChange={FormUtils.handleUpdate('userLdapFilter', send)}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.USER_ID_ATTRIBUTE} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('userIdAttribute', state)}
            onChange={FormUtils.handleUpdate('userIdAttribute', send)}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.REAL_NAME_ATTRIBUTE} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('userRealNameAttribute', state)}
            onChange={FormUtils.handleUpdate('userRealNameAttribute', send)}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.EMAIL_ATTRIBUTE} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('userEmailAddressAttribute', state)}
            onChange={FormUtils.handleUpdate('userEmailAddressAttribute', send)}
          />
        </AsFormGroup>
        <AsFormGroup
          label={LABELS.PASSWORD_ATTRIBUTE.LABEL}
          sublabel={LABELS.PASSWORD_ATTRIBUTE.SUB_LABEL}
        >
          <AsTextInput
            {...FormUtils.fieldProps('userPasswordAttribute', state)}
            onChange={FormUtils.handleUpdate('userPasswordAttribute', send)}
          />
        </AsFormGroup>
        <AsFieldset label={LABELS.MAP_LDAP.LABEL}>
          <AsCheckbox
            {...FormUtils.checkboxProps('ldapGroupsAsRoles', state)}
            onChange={FormUtils.handleUpdate('ldapGroupsAsRoles', send)}
          >
            {LABELS.MAP_LDAP.SUB_LABEL}
          </AsCheckbox>
        </AsFieldset>
        {ldapGroupsAsRoles && (
          <>
            <AsFormGroup label={LABELS.GROUP_TYPE.LABEL} isRequired>
              <AsFormSelect
                {...FormUtils.fieldProps('groupType', state)}
                onChange={FormUtils.handleUpdate('groupType', send)}
                validatable
              >
                {Object.values(LABELS.GROUP_TYPE.OPTIONS).map(({id, label}) => (
                  <option key={id} value={id}>
                    {label}
                  </option>
                ))}
              </AsFormSelect>
            </AsFormGroup>
            {isDynamicGroup(groupType) && (
              <AsFormGroup
                label={LABELS.GROUP_MEMBER_OF_ATTRIBUTE.LABEL}
                sublabel={LABELS.GROUP_MEMBER_OF_ATTRIBUTE.SUB_LABEL}
                isRequired
              >
                <AsTextInput
                  {...FormUtils.fieldProps('userMemberOfAttribute', state)}
                  onChange={FormUtils.handleUpdate(
                    'userMemberOfAttribute',
                    send
                  )}
                />
              </AsFormGroup>
            )}
            {isStaticGroup(groupType) && (
              <>
                <AsFormGroup
                  label={LABELS.GROUP_DN.LABEL}
                  sublabel={LABELS.GROUP_DN.SUB_LABEL}
                >
                  <AsTextInput
                    {...FormUtils.fieldProps('groupBaseDn', state)}
                    onChange={FormUtils.handleUpdate('groupBaseDn', send)}
                  />
                </AsFormGroup>
                <AsFieldset label={LABELS.GROUP_SUBTREE.LABEL}>
                  <AsCheckbox
                    {...FormUtils.checkboxProps('groupSubtree', state)}
                    onChange={FormUtils.handleUpdate('groupSubtree', send)}
                  >
                    {LABELS.GROUP_SUBTREE.SUB_LABEL}
                  </AsCheckbox>
                </AsFieldset>
                <AsFormGroup
                  label={LABELS.GROUP_OBJECT_CLASS.LABEL}
                  sublabel={LABELS.GROUP_OBJECT_CLASS.SUB_LABEL}
                  isRequired
                >
                  <AsTextInput
                    {...FormUtils.fieldProps('groupObjectClass', state)}
                    onChange={FormUtils.handleUpdate('groupObjectClass', send)}
                  />
                </AsFormGroup>
                <AsFormGroup label={LABELS.GROUP_ID_ATTRIBUTE.LABEL} isRequired>
                  <AsTextInput
                    {...FormUtils.fieldProps('groupIdAttribute', state)}
                    onChange={FormUtils.handleUpdate('groupIdAttribute', send)}
                  />
                </AsFormGroup>
                <AsFormGroup
                  label={LABELS.GROUP_MEMBER_ATTRIBUTE.LABEL}
                  sublabel={LABELS.GROUP_MEMBER_ATTRIBUTE.SUB_LABEL}
                  isRequired
                >
                  <AsTextInput
                    {...FormUtils.fieldProps('groupMemberAttribute', state)}
                    onChange={FormUtils.handleUpdate(
                      'groupMemberAttribute',
                      send
                    )}
                  />
                </AsFormGroup>
                <AsFormGroup
                  label={LABELS.GROUP_MEMBER_FORMAT.LABEL}
                  sublabel={LABELS.GROUP_MEMBER_FORMAT.SUB_LABEL}
                  isRequired
                >
                  <AsTextInput
                    {...FormUtils.fieldProps('groupMemberFormat', state)}
                    onChange={FormUtils.handleUpdate('groupMemberFormat', send)}
                    className="as-text-input--long"
                  />
                </AsFormGroup>
              </>
            )}
          </>
        )}
      </AsStatefulForm>
      {askingPassword && (
        <LdapServersModalPassword
          onCancel={cancel}
          onSubmit={({value}) => setSystemPassword(value)}
        />
      )}
      {showingVerifyLoginModal && (
        <LdapVerifyLoginModal ldapConfig={data} onCancel={cancel} />
      )}
      {showingVerifyUserMappingModal && (
        <LdapVerifyUserMappingModal ldapConfig={data} onCancel={cancel} />
      )}
    </AsTile.Content>
  );
}
