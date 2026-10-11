

import React, { useMemo, useEffect } from 'react';
import { Box, Flex, Text, } from '@radix-ui/themes';
import { Trash2, Loader2 } from 'lucide-react';
import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsTextArea,
  SettingsSelect,
  SettingsButton,
  SettingsCheckboxGroup,
  SettingsCombobox,
} from '../../../../shared/form';
import { usePrivilegesApi } from './usePrivilegesApi';
import { usePrivilegeForm } from './usePrivilegeForm';
import {
  PrivilegeFormProps,
  PRIVILEGE_TYPES,
  getPrivilegeTypeLabel,
  isReadOnlyPrivilege,
  getActionsForPrivilegeType,
  APPLICATION_DOMAINS,
  ContentSelector,
} from './types';
import { SelectionInsights } from './SelectionInsights';

import './PrivilegeForm.scss';
import PrivilegesStrings from '../../../../constants/pages/settings/security/PrivilegesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * PrivilegeForm - Form for creating and editing privileges
 */
export function PrivilegeForm({
  privilege,
  isCreate,
  typeId,
  onSave,
  onCancel,
  onDelete,
  loading = false,
  error,
  hideActions = false,
  onSubmitRef,
  onValidationChange,
  wizardStep,
}: PrivilegeFormProps & { 
  typeId?: string; 
  hideActions?: boolean; 
  onSubmitRef?: React.MutableRefObject<(() => void) | null>;
  onValidationChange?: (isValid: boolean) => void;
  wizardStep?: number;
}) {
  const { createPrivilege, updatePrivilege } = usePrivilegesApi();

  const { form, privilege: formPrivilege } = usePrivilegeForm({
    privilegeId: isCreate ? undefined : privilege?.id,
    privilege: privilege || undefined,
    typeId,
    onSave,
    onCancel,
    createPrivilege,
    updatePrivilege,
  });

  const isLoading = form.state.matches('loading');
  const isSaving = form.state.matches('saving');
  const context = form.state.context as any;
  const formData = context.data;

  useEffect(() => {
    if (onValidationChange) {
      const isNameValid = !!formData?.name?.trim() && !form.validationErrors?.name;
      if (wizardStep === 1) {
        onValidationChange(isNameValid);
      } else {
        onValidationChange(true);
      }
    }
  }, [formData?.name, form.validationErrors?.name, onValidationChange, wizardStep]);

  if (onSubmitRef) {
    onSubmitRef.current = () => form.send('SUBMIT');
  }

  const currentPrivilege = privilege || formPrivilege;
  const isReadOnly = currentPrivilege ? isReadOnlyPrivilege(currentPrivilege) : false;

  const repositories = context.repositories || [];
  const formats = context.formats || [];
  const contentSelectors: ContentSelector[] = context.contentSelectors || [];
  const scripts = context.scripts || [];

  const selectedRepo = useMemo(() => {
    const repoName = formData?.properties?.repository;
    if (!repoName || repoName === '*') return null;
    return repositories.find((r: any) => r.name === repoName) || null;
  }, [repositories, formData?.properties?.repository]);

  const selectedSelector = useMemo(() => {
    const selectorName = formData?.properties?.contentSelector;
    if (!selectorName) return null;
    return contentSelectors.find((s) => s.name === selectorName) || null;
  }, [contentSelectors, formData?.properties?.contentSelector]);

  const selectedFormat = formData?.properties?.format;

  const _currentPrivilegeType = typeId || formData?.type;

  // Repository options for repository-view and repository-admin privileges
  // These types only accept "*" or a specific repository name (not "*-format")
  const repositoryOptions = useMemo(() => {
    const options: Array<{ value: string; label: string }> = [];

    // Show "(All Repositories)" option with value "*"
    // When format is specified, "*" means "all repositories of that format"
    options.push({ value: '*', label: PrivilegesStrings.FORM.ALL_REPOSITORIES });

    // Add individual repositories (filtered by selected format if applicable)
    const filteredRepos = selectedFormat && selectedFormat !== '*'
      ? repositories.filter((r: any) => r.format === selectedFormat)
      : repositories;
    filteredRepos.forEach((r: any) => {
      options.push({ value: r.name, label: r.name });
    });

    return options;
  }, [repositories, selectedFormat]);

  // Repository options for repository-content-selector privileges
  // This type accepts "*", "*-format", or a specific repository name
  const contentSelectorRepositoryOptions = useMemo(() => {
    const options: Array<{ value: string; label: string }> = [];

    // Only show "* (All Repositories)" if format is "*" or not selected
    if (!selectedFormat || selectedFormat === '*') {
      options.push({ value: '*', label: PrivilegesStrings.FORM.ALL_REPOSITORIES });
    }

    // Add "All {format} Repositories" options - filtered by selected format
    // Uses "*-{format}" format (e.g., "*-maven2") which matches RepositorySelector.ALL_OF_FORMAT_PREFIX
    const uniqueFormats = Array.from(new Set(repositories.map((r: any) => r.format)));
    uniqueFormats.sort().forEach(format => {
      if (format) {
        // Only show this format option if it matches the selected format or no format is selected
        if (!selectedFormat || selectedFormat === '*' || selectedFormat === format) {
          options.push({
            value: `*-${format}`,
            label: PrivilegesStrings.FORM.ALL_FORMAT_REPOSITORIES(format)
          });
        }
      }
    });

    // Add individual repositories (filtered by selected format if applicable)
    const filteredRepos = selectedFormat && selectedFormat !== '*'
      ? repositories.filter((r: any) => r.format === selectedFormat)
      : repositories;
    filteredRepos.forEach((r: any) => {
      options.push({ value: r.name, label: r.name });
    });

    return options;
  }, [repositories, selectedFormat]);

  const formatOptions = useMemo(() => [
    { value: '*', label: '* (All Formats)' },
    ...formats.map((f: string) => ({ value: f, label: f })),
  ], [formats]);

  const contentSelectorOptions = useMemo(() => {
    const filteredSelectors = selectedFormat && selectedFormat !== '*'
      ? contentSelectors.filter((s) => {
          const lowerExpr = s.expression.toLowerCase();
          const regex = /format\s*(?:==|!=|=~|=\^|\^=)\s*["']([^"']+)["']/gi;
          let match;
          let hasRestriction = false;
          let matches = false;
          while ((match = regex.exec(lowerExpr)) !== null) {
            hasRestriction = true;
            if (match[1].toLowerCase() === selectedFormat.toLowerCase() || match[1] === '*') {
              matches = true;
            }
          }
          return !hasRestriction || matches;
        })
      : contentSelectors;
    return filteredSelectors.map((s: ContentSelector) => ({ 
      value: s.name, 
      label: s.name,
      description: s.expression,
    }));
  }, [contentSelectors, selectedFormat]);

  const scriptOptions = useMemo(() =>
    scripts.map((s: any) => ({ value: s.name, label: s.name })),
  [scripts]);

  const handleTypeChange = (value: string) => {
    form.send({ type: 'TYPE_CHANGE', value });
  };

  const renderTypeSpecificFields = () => {
    const type = typeId || formData?.type;
    switch (type) {
      case PRIVILEGE_TYPES.WILDCARD:
        return (
          <Box>
            <SettingsTextInput
              {...form.field('properties.pattern')}
              label={PrivilegesStrings.FORM.PRIVILEGE_STRING}
              required
              disabled={isReadOnly}
            />
            <Text as="p" size="1" mt="2" style={{ color: 'var(--gray-11)' }}>
              {PrivilegesStrings.FORM.SHIRO_NOTE(
                <a
                  href="https://help.globalart.dev/en/privileges.html"
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{ color: 'var(--accent-11)', textDecoration: 'underline' }}
                  onMouseDown={(e) => {
                    // Open link before blur event fires to prevent validation from blocking navigation
                    e.preventDefault();
                    window.open('https://help.globalart.dev/en/privileges.html', '_blank', 'noopener,noreferrer');
                  }}
                >
                  {PrivilegesStrings.FORM.DOCUMENTATION_LINK}
                </a>
              )}
            </Text>
          </Box>
        );
      case PRIVILEGE_TYPES.APPLICATION:
        return (
          <Box>
            <SettingsCombobox
              {...form.field('properties.domain')}
              label={PrivilegesStrings.FORM.DOMAIN}
              options={APPLICATION_DOMAINS}
              helpText={PrivilegesStrings.FORM.APPLICATION_DOMAIN_PRIVILEGE_APPLIES_SELECT}
              placeholder={PrivilegesStrings.FORM.SELECT_TYPE_DOMAIN_PLACEHOLDER}
              required
              disabled={isReadOnly}
              allowCustom
            />
            <SettingsCheckboxGroup
              {...form.checkboxGroup('properties.actions')}
              label={CommonStrings.LABELS.ACTIONS}
              options={getActionsForPrivilegeType(type)}
              helpText={PrivilegesStrings.FORM.SPECIFIC_ACTIONS_ALLOWED_WITHIN_DOMAIN}
              required
              disabled={isReadOnly}
              layout="horizontal"
            />
          </Box>
        );
      case PRIVILEGE_TYPES.REPOSITORY_VIEW:
      case PRIVILEGE_TYPES.REPOSITORY_ADMIN:
        return (
          <Box>
            <SettingsCombobox
              {...form.field('properties.format')}
              label={PrivilegesStrings.FORM.REPOSITORY_FORMAT}
              options={formatOptions}
              helpText={PrivilegesStrings.FORM.REPOSITORY_FORMAT_PRIVILEGE_APPLIES_USE}
              required
              disabled={isReadOnly}
              allowCustom
              placeholder={PrivilegesStrings.FORM.SELECT_FORMAT_PLACEHOLDER}
            />
            <SettingsCombobox
              {...form.field('properties.repository')}
              label={PrivilegesStrings.FORM.REPOSITORY}
              options={repositoryOptions}
              helpText={PrivilegesStrings.FORM.SPECIFIC_REPOSITORY_PRIVILEGE_APPLIES_USE}
              required
              disabled={isReadOnly}
              allowCustom
              placeholder={PrivilegesStrings.FORM.SELECT_REPOSITORY_PLACEHOLDER}
            />
            <SettingsCheckboxGroup
              {...form.checkboxGroup('properties.actions')}
              label={CommonStrings.LABELS.ACTIONS}
              options={getActionsForPrivilegeType(type)}
              helpText={PrivilegesStrings.FORM.SPECIFIC_ADMINISTRATIVE_VIEW_ACTIONS_ALLOWED}
              required
              disabled={isReadOnly}
              layout="horizontal"
            />
          </Box>
        );
      case PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR:
        return null;
      case PRIVILEGE_TYPES.SCRIPT:
        return (
          <Box>
            <SettingsCombobox
              {...form.field('properties.name')}
              label={PrivilegesStrings.FORM.SCRIPT_NAME}
              options={scriptOptions}
              helpText={PrivilegesStrings.FORM.NAME_SCRIPT_PRIVILEGE_APPLIES_SELECT}
              required
              disabled={isReadOnly}
              allowCustom
              placeholder={PrivilegesStrings.FORM.SELECT_TYPE_SCRIPT_NAME_PLACEHOLDER}
            />
            <SettingsCheckboxGroup
              {...form.checkboxGroup('properties.actions')}
              label={CommonStrings.LABELS.ACTIONS}
              options={getActionsForPrivilegeType(type)}
              helpText={PrivilegesStrings.FORM.SPECIFIC_SCRIPT_ACTIONS_ALLOWED_FORMAT}
              required
              disabled={isReadOnly}
              layout="horizontal"
            />
          </Box>
        );
      default:
        return null;
    }
  };

  if (isLoading) {
    return (
      <Flex align="center" justify="center" className="privilege-form__loading">
        <Loader2 size={24} className="privilege-form__spinner" />
        <Text size="2">{CommonStrings.MESSAGES.LOADING_FORM}</Text>
      </Flex>
    );
  }

  const showSetupSection = !hideActions || wizardStep === 1;
  const showScopeSection = !hideActions || wizardStep === 2;
  const showActionsSection = !hideActions || wizardStep === 2;
  const showConfigSection = !hideActions || wizardStep === 2;
  const showPreviewSection = hideActions && wizardStep === 3;
  const isContentSelector = (typeId || formData?.type) === PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR;
  const isAuditMode = isContentSelector && showPreviewSection;

  const renderFormFields = () => (
    <Box>
      {showSetupSection && (
        <SettingsFormSection title={PrivilegesStrings.FORM.PRIVILEGE_SETUP} defaultOpen>
          <SettingsTextInput
            {...form.field('name')}
            label={CommonStrings.LABELS.NAME}
            helpText={PrivilegesStrings.FORM.UNIQUE_NAME_USED_IDENTIFY_PRIVILEGE}
            required
            disabled={!isCreate || isReadOnly}
          />
          <SettingsTextArea
            {...form.field('description')}
            label={CommonStrings.LABELS.DESCRIPTION}
            helpText={PrivilegesStrings.FORM.BRIEF_EXPLANATION_WHAT_PRIVILEGE_ALLOWS}
            rows={2}
            disabled={isReadOnly}
          />
          {!typeId && (
            <SettingsSelect
              name="type"
              label={CommonStrings.LABELS.TYPE}
              value={formData?.type || ''}
              onChange={handleTypeChange}
              options={context.privilegeTypes?.map((t: any) => ({
                value: t.id,
                label: t.name || getPrivilegeTypeLabel(t.id),
              })) || []}
              helpText={PrivilegesStrings.FORM.FUNDAMENTAL_CATEGORY_PRIVILEGE_WHICH_DETERMINES}
              required
              disabled={!isCreate || isReadOnly}
            />
          )}
        </SettingsFormSection>
      )}
      {showConfigSection && isContentSelector ? (
        <>
          {showScopeSection && (
            <SettingsFormSection title={PrivilegesStrings.FORM.CONTENT} defaultOpen>
              <SettingsCombobox
                {...form.field('properties.contentSelector')}
                label={PrivilegesStrings.FORM.CONTENT_SELECTOR}
                options={contentSelectorOptions}
                helpText={PrivilegesStrings.FORM.CSEL_EXPRESSION_LIMITS_WHICH_PATHS}
                required
                disabled={isReadOnly}
                placeholder={PrivilegesStrings.FORM.SELECT_CONTENT_SELECTOR_PLACEHOLDER}
              />
            </SettingsFormSection>
          )}
          {showScopeSection && (
            <SettingsFormSection title={PrivilegesStrings.FORM.REPOSITORY} defaultOpen>
              <SettingsCombobox
                {...form.field('properties.repository')}
                label={PrivilegesStrings.FORM.REPOSITORY}
                options={contentSelectorRepositoryOptions}
                helpText={PrivilegesStrings.FORM.CONTENT_REPOSITORY_HELP}
                required
                disabled={isReadOnly}
                allowCustom
                placeholder={PrivilegesStrings.FORM.SELECT_REPOSITORY_PLACEHOLDER}
              />
            </SettingsFormSection>
          )}
          {showActionsSection && (
            <SettingsFormSection title={CommonStrings.LABELS.ACTIONS} defaultOpen>
              <SettingsCheckboxGroup
                {...form.checkboxGroup('properties.actions')}
                label={CommonStrings.LABELS.ACTIONS}
                options={getActionsForPrivilegeType(PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR)}
                helpText={PrivilegesStrings.FORM.SPECIFIC_CONTENT_ACTIONS_ALLOWED_FORMAT}
                required
                disabled={isReadOnly}
                layout="horizontal"
              />
            </SettingsFormSection>
          )}
        </>
      ) : showConfigSection ? (
        <SettingsFormSection title={PrivilegesStrings.FORM.CONFIGURATION} defaultOpen>
          {renderTypeSpecificFields()}
        </SettingsFormSection>
      ) : null}
    </Box>
  );

  const renderFormContent = () => (
    <Box width="100%">
      {isAuditMode ? (
        // Preview step: SelectionInsights full-width, no form fields
        <Box width="100%">
          <SelectionInsights
            repository={selectedRepo}
            allRepositories={formData?.properties?.repository === '*'}
            selectedFormat={selectedFormat}
            contentSelector={selectedSelector}
          />
        </Box>
      ) : (
        <Box width="100%">{renderFormFields()}</Box>
      )}
    </Box>
  );

  if (hideActions) {
    return <Box className="privilege-form">{renderFormContent()}</Box>;
  }

  return (
    <Box className="privilege-form">
      <SettingsForm
        testId="privilege-form"
        onSubmit={() => form.send('SUBMIT')}
        onCancel={onCancel}
        loading={isSaving || loading}
        pristine={form.isPristine}
        error={error || (form.state.matches('error') ? form.state.context.error : undefined)}
        submitLabel={isCreate ? CommonStrings.ACTIONS.CREATE : CommonStrings.ACTIONS.SAVE}
        submitAnalyticsId={isCreate ? 'as-privilege-create' : 'as-privilege-save'}
        footerExtra={
          !isCreate && onDelete && !isReadOnly ? (
            <SettingsButton
              testId="form-delete"
              variant="danger"
              onClick={onDelete}
              disabled={isSaving || loading}
              icon={Trash2}
              data-analytics-id="as-privilege-delete"
            >
              {CommonStrings.ACTIONS.DELETE}
            </SettingsButton>
          ) : undefined
        }
      >
        {renderFormContent()}
      </SettingsForm>
    </Box>
  );
}

export default PrivilegeForm;
