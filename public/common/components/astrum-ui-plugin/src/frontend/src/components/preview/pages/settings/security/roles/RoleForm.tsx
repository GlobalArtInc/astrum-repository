

import React, { useMemo, useEffect, useState, useCallback, useRef } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Trash2, Loader2, AlertCircle } from 'lucide-react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsTextArea,
  SettingsCombobox,
  SettingsTransferList,
  SettingsButton,
  SettingsSelect,
} from '../../../../shared/form';
import { useRolesApi } from './useRolesApi';
import { useRolesForm } from './useRolesForm';
import {
  RoleFormData,
  RoleFormProps,
  RolesFormContext,
  ASTRUM_SOURCE,
  PRIVILEGE_GROUP_LABELS,
  LDAP_EMPTY_SEARCH_MESSAGE,
  isExternalRole,
  isReadOnlyRole,
} from './types';

import './RoleForm.scss';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

/**
 * RoleForm - Form for creating and editing roles
 * Uses XState for state management via useRolesForm hook
 */
export function RoleForm({
  role,
  isCreate,
  onCancel,
  onDelete,
  onComplete,
  loading = false,
  error,
  hideActions = false,
  onSubmitRef,
  onValidationChange,
  wizardStep = 0,
}: RoleFormProps & {
  hideActions?: boolean;
  onSubmitRef?: React.MutableRefObject<(() => void) | null>;
  onValidationChange?: (isValid: boolean) => void;
  wizardStep?: number;
}) {
  const { createRole, updateRole, searchRoles } = useRolesApi();
  const [ldapRoleOptions, setLdapRoleOptions] = useState<Array<{ value: string; label: string }>>([]);
  const [ldapSearchLoading, setLdapSearchLoading] = useState(false);
  const [ldapEmptyMessage, setLdapEmptyMessage] = useState<string | null>(null);
  const ldapSearchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const { form, role: loadedRole } = useRolesForm({
    roleId: isCreate ? undefined : role?.id,
    role: role || undefined,
    onCancel,
    onComplete,
    createRole,
    updateRole,
  });

  // P0: Require at least one privilege or contained role on final step
  const formData = form.data as RoleFormData;
  const hasPrivilegeOrRole = (formData.privileges?.length ?? 0) > 0 || (formData.roles?.length ?? 0) > 0;

  // Calculate validation state with useMemo for cleaner dependency tracking
  const isStepValid = useMemo(() => {
    const isIdValid = !!form.data.id?.trim() && !form.validationErrors?.id;
    const isNameValid = !!form.data.name?.trim() && !form.validationErrors?.name;
    const hasValidExternalSource = formData.roleType !== 'external' || !!formData.externalSource;

    if (wizardStep === 0) return true; // Step 0: Role type selection (always valid)
    if (wizardStep === 1) return isIdValid && isNameValid && hasValidExternalSource;
    if (wizardStep === 3) return hasPrivilegeOrRole;
    return true;
  }, [form.data.id, form.data.name, form.validationErrors, formData.roleType, formData.externalSource, wizardStep, hasPrivilegeOrRole]);

  // Notify parent of validation status
  useEffect(() => {
    onValidationChange?.(isStepValid);
  }, [onValidationChange, isStepValid]);

  const submitDisabled = !hasPrivilegeOrRole;
  const [hasAttemptedAdvance, setHasAttemptedAdvance] = useState(false);

  // Expose submit function to parent via ref (for WizardForm integration)
  useEffect(() => {
    if (onSubmitRef) {
      onSubmitRef.current = () => {
        if (wizardStep === 3) setHasAttemptedAdvance(true);
        if (submitDisabled && wizardStep === 3) return;
        if (!form.isLoading && !form.isSaving) {
          form.submit();
        }
      };
      return () => { onSubmitRef.current = null; };
    }
  }, [onSubmitRef, form, wizardStep, submitDisabled]);

  // Reset attempt flag only when returning to Type Selection (step 0) - keeps warning visible if user goes Back from step 3 to step 2
  useEffect(() => {
    if (wizardStep === 0) setHasAttemptedAdvance(false);
  }, [wizardStep]);

  // Cleanup LDAP search timer on unmount
  useEffect(() => {
    return () => {
      if (ldapSearchTimerRef.current) {
        clearTimeout(ldapSearchTimerRef.current);
      }
    };
  }, []);

  // Use the role from props if provided, otherwise from form state
  const currentRole = role || loadedRole;
  const isExternal = currentRole ? isExternalRole(currentRole.source) : false;
  const isReadOnly = currentRole ? isReadOnlyRole(currentRole) : false;

  // Access extended context for reference data
  const context = form.state.context as RolesFormContext;
  const allPrivileges = context.allPrivileges || [];
  const allRoles = context.allRoles || [];
  const allSources = context.allSources || [];

  const _sourceOptions = useMemo(() => {
    const options = allSources.map(s => ({ value: s.id, label: s.name }));
    if (!options.some(o => o.value === ASTRUM_SOURCE)) {
      options.unshift({ value: ASTRUM_SOURCE, label: ASTRUM_SOURCE });
    }
    return options;
  }, [allSources]);

  // Convert privileges to combobox option format
  const privilegeOptions = useMemo(() => {
    return allPrivileges.map((priv) => ({
      value: priv.id,
      label: priv.name,
      description: priv.description,
    }));
  }, [allPrivileges]);

  // Group privileges by prefix for better organization in combobox
  const groupPrivilege = useCallback((option: { value: string; label: string }) => {
    const prefixes = Object.keys(PRIVILEGE_GROUP_LABELS);
    for (const prefix of prefixes) {
      if (option.value.startsWith(prefix)) return PRIVILEGE_GROUP_LABELS[prefix];
    }
    return RolesStrings.FORM.OTHER;
  }, []);

  const showPrivilegeWarning = submitDisabled && hasAttemptedAdvance && wizardStep === 3;

  const handlePrivilegesChange = (values: string[]) => {
    form.send({ type: 'UPDATE', name: 'privileges', value: values });
  };

  // Convert roles to transfer list format
  const availableRoles = useMemo(() => {
    return allRoles
      .filter((r) => r.id !== role?.id && r.id !== formData.id)
      .map((r) => ({
        id: r.id,
        name: r.name,
      }));
  }, [allRoles, role?.id, formData.id]);

  const selectedRoles = useMemo(() => {
    return allRoles
      .filter((r) => formData.roles?.includes(r.id))
      .map((r) => ({
        id: r.id,
        name: r.name,
      }));
  }, [allRoles, formData.roles]);

  const handleRolesChange = (newSelected: Array<{ id: string; name: string }>) => {
    form.send({ type: 'UPDATE', name: 'roles', value: newSelected.map((r) => r.id) });
  };

  // Handle LDAP role search with debounce
  const handleLdapSearch = useCallback(async (query: string) => {
    if (formData.externalSource !== 'LDAP') {
      setLdapRoleOptions([]);
      setLdapEmptyMessage(null);
      return;
    }

    if (query.length < 3) {
      setLdapRoleOptions([]);
      setLdapEmptyMessage(RolesStrings.FORM.ENTER_3_MORE_CHARACTERS_SEARCH);
      return;
    }

    // Clear any pending search
    if (ldapSearchTimerRef.current) {
      clearTimeout(ldapSearchTimerRef.current);
    }

    // Debounce the search by 300ms
    ldapSearchTimerRef.current = setTimeout(async () => {
      setLdapSearchLoading(true);
      setLdapEmptyMessage(null);

      try {
        const results = await searchRoles(formData.externalSource, query);
        const options = results.map(r => ({ value: r.id, label: r.name }));
        setLdapRoleOptions(options);
        if (options.length === 0) {
          setLdapEmptyMessage(LDAP_EMPTY_SEARCH_MESSAGE);
        }
      } catch (err) {
        console.error('Failed to search LDAP roles:', err);
        setLdapRoleOptions([]);
        setLdapEmptyMessage(RolesStrings.FORM.FAILED_SEARCH_LDAP_ROLES);
      } finally {
        setLdapSearchLoading(false);
      }
    }, 300);
  }, [formData.externalSource, searchRoles]);

  // Show loading state while data loads
  if (form.isLoading) {
    return (
      <Flex align="center" justify="center" className="role-form__loading">
        <Loader2 size={24} className="role-form__spinner" />
        <Text size="2">{CommonStrings.MESSAGES.LOADING_FORM}</Text>
      </Flex>
    );
  }

  const roleTypeOptions = [
    { value: 'astrum', label: RolesStrings.FORM.ASTRUM_ROLE, description: RolesStrings.FORM.STANDARD_ROLE_MANAGED_ASTRUM },
    { value: 'external', label: RolesStrings.FORM.EXTERNAL_ROLE_MAPPING, description: RolesStrings.FORM.MAP_ROLES_LDAP_SAML_CROWD },
  ];

  const formContent = (
    <>
      {/* Step 0: Role Type — create mode only. Edit mode skips this step at the
          wizard level (RolesPage never passes wizardStep=0 when !isCreate) because
          the role type is immutable once persisted; showing a read-only step there
          adds no value. Props are passed individually (not via form.field spread)
          so the explicit value={... ?? 'astrum'} default is unambiguous. */}
      {wizardStep === 0 && isCreate && (
        <SettingsFormSection title={RolesStrings.FORM.ROLE_TYPE} defaultOpen>
          <Text size="2" className="role-form__section-help">
            {RolesStrings.FORM.CHOOSE_TYPE_ROLE_CREATE}
          </Text>
          <SettingsSelect
            name="roleType"
            onChange={(value) => form.send({ type: 'UPDATE', name: 'roleType', value })}
            label={CommonStrings.LABELS.TYPE}
            options={roleTypeOptions}
            helpText={RolesStrings.FORM.SELECT_WHETHER_STANDARD_ASTRUM_ROLE}
            value={formData.roleType ?? 'astrum'}
          />
        </SettingsFormSection>
      )}

      {/* Step 1: Role Setup Section */}
      {wizardStep === 1 && (
        <SettingsFormSection title={RolesStrings.FORM.ROLE_SETUP} defaultOpen>
          {/* External Source is FIRST for external roles */}
          {formData.roleType === 'external' && (
            <>
              <SettingsSelect
                {...form.field('externalSource')}
                label={RolesStrings.FORM.EXTERNAL_SOURCE}
                options={[
                  { value: 'LDAP', label: 'LDAP' },
                  { value: 'SAML', label: 'SAML' },
                  { value: 'Crowd', label: 'Crowd' },
                ]}
                helpText={RolesStrings.FORM.SELECT_EXTERNAL_AUTHENTICATION_SOURCE_LDAP}
                disabled={!isCreate || isReadOnly}
                required
              />

              {formData.externalSource && (
                <SettingsCombobox
                  name="mappedRole"
                  value={formData.mappedRole || ''}
                  onChange={(value) => {
                    form.send({ type: 'UPDATE', name: 'mappedRole', value });
                  }}
                  onInputChange={handleLdapSearch}
                  options={ldapRoleOptions}
                  placeholder={formData.externalSource === 'LDAP' ? RolesStrings.FORM.SEARCH_LDAP_ROLES : RolesStrings.FORM.ENTER_ROLE_NAME(formData.externalSource)}
                  label={RolesStrings.FORM.MAPPED_ROLE}
                  helpText={formData.externalSource === 'LDAP'
                    ? RolesStrings.FORM.ENTER_SEARCH_ROLE_LDAP_TYPE
                    : RolesStrings.FORM.ENTER_EXACT_ROLE_NAME(formData.externalSource)}
                  allowCustomValue
                  loading={ldapSearchLoading}
                  disabled={isReadOnly}
                  hideEmptyMessage={formData.externalSource !== 'LDAP'}
                  emptyMessage={ldapEmptyMessage}
                />
              )}
            </>
          )}

          <SettingsTextInput
            {...form.field('id')}
            label={RolesStrings.FORM.ID}
            placeholder="as-custom-role"
            helpText={RolesStrings.FORM.UNIQUE_IDENTIFIER_ROLE}
            required
            disabled={!isCreate || isExternal}
          />

          <SettingsTextInput
            {...form.field('name')}
            label={CommonStrings.LABELS.NAME}
            placeholder={RolesStrings.FORM.MY_CUSTOM_ROLE_PLACEHOLDER}
            helpText={RolesStrings.FORM.HUMAN_READABLE_NAME_ROLE}
            required
            disabled={isReadOnly}
          />

          <SettingsTextArea
            {...form.field('description')}
            label={CommonStrings.LABELS.DESCRIPTION}
            placeholder={RolesStrings.FORM.GRANTS_READ_ACCESS_ALL_MAVEN_PLACEHOLDER}
            helpText={RolesStrings.FORM.OPTIONAL_DESCRIPTION_ROLES_PURPOSE}
            rows={3}
            disabled={isReadOnly}
          />
        </SettingsFormSection>
      )}

      {/* Step 2: Privileges Section */}
      {wizardStep === 2 && (
        <SettingsFormSection title={RolesStrings.FORM.PRIVILEGES} defaultOpen>
          <Text size="2" className="role-form__section-help">
            {RolesStrings.FORM.SEARCH_SELECT_PRIVILEGES_GRANT_RESULTS}
          </Text>
          <SettingsCombobox
            name="privileges"
            multiple
            selectedValues={formData.privileges || []}
            onMultiChange={handlePrivilegesChange}
            options={privilegeOptions}
            groupBy={groupPrivilege}
            placeholder={RolesStrings.FORM.SEARCH_PRIVILEGES_NAME_PLACEHOLDER}
            helpText={plural(RolesStrings.FORM.PRIVILEGES_GRANTED, formData.privileges?.length || 0)}
            disabled={isReadOnly}
          />
          {hasAttemptedAdvance && form.validationErrors?.privileges && (
            <Text size="1" className="role-form__error">{form.validationErrors.privileges}</Text>
          )}
        </SettingsFormSection>
      )}

      {/* Step 3: Contained Roles Section */}
      {wizardStep === 3 && (
        <>
          <SettingsFormSection title={RolesStrings.FORM.CONTAINED_ROLES} defaultOpen>
            <Text size="2" className="role-form__section-help">
              {RolesStrings.FORM.SELECT_OTHER_ROLES_INCLUDE_THEIR}
            </Text>
            <SettingsTransferList
              name="roles"
              testId="role-form-roles"
              availableItems={availableRoles}
              selectedItems={selectedRoles}
              onChange={handleRolesChange}
              availableLabel={RolesStrings.FORM.AVAILABLE}
              selectedLabel={RolesStrings.FORM.CONTAINED}
              getItemId={(item) => item.id}
              getItemLabel={(item) => item.name}
              disabled={isReadOnly}
            />
          </SettingsFormSection>

          {/* Validation error: no privileges or roles selected */}
          {showPrivilegeWarning && (
            <Flex align="center" gap="2" className="role-form__privilege-warning" data-testid="privilege-warning" role="alert">
              <AlertCircle size={14} />
              <Text size="2" weight="medium">
                {form.validationErrors?.privileges ?? RolesStrings.FORM.SELECT_LEAST_ONE_PRIVILEGE_CONTAINED}
              </Text>
            </Flex>
          )}
        </>
      )}
    </>
  );

  if (hideActions) {
    return <Box className="role-form">{formContent}</Box>;
  }

  return (
    <Box className="role-form">
      <SettingsForm
        testId="role-form"
        onSubmit={() => {
          if (wizardStep === 2) setHasAttemptedAdvance(true);
          if (submitDisabled && wizardStep === 2) return;
          form.submit();
        }}
        onCancel={onCancel}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        loading={form.isSaving || loading}
        pristine={form.isPristine}
        error={error || form.saveError || undefined}
        submitLabel={isCreate ? CommonStrings.ACTIONS.CREATE : CommonStrings.ACTIONS.SAVE}
        submitDisabled={submitDisabled}
        footerExtra={
          !isCreate && onDelete && !isReadOnly ? (
            <SettingsButton
              testId="form-delete"
              variant="danger"
              onClick={onDelete}
              disabled={form.isSaving || loading}
              icon={Trash2}
            >
              {RolesStrings.FORM.DELETE_ROLE}
            </SettingsButton>
          ) : undefined
        }
      >
        {formContent}
      </SettingsForm>
    </Box>
  );
}


export default RoleForm;
