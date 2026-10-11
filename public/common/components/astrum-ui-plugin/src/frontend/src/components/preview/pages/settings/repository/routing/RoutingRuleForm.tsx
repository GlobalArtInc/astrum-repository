

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Trash2, Info, ExternalLink, Loader2 } from 'lucide-react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
  SettingsTextArea,
  SettingsSelect,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import { useRoutingRulesForm } from './useRoutingRulesForm';
import { RoutingRuleMatcher } from './RoutingRuleMatcher';
import {
  RoutingRuleFormProps,
  RoutingRuleFormData,
  RoutingMode,
  ROUTING_MODE_LABELS,
  ROUTING_MODE_HELP,
  hasInvalidMatchers,
} from './types';

import './RoutingRuleForm.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import RoutingRulesStrings from '../../../../constants/pages/settings/repository/RoutingRulesStrings';
import plural from '../../../../../../interface/plural';

/**
 * RoutingRuleForm - Create/Edit form for routing rules
 * Now uses XState for state management via useRoutingRulesForm hook
 */
export function RoutingRuleForm({
  rule,
  isCreate,
  onSave,
  onCancel,
  onDelete,
  loading = false,
  error,
}: RoutingRuleFormProps) {
  const [testPath, setTestPath] = React.useState('');

  // Use XState form hook
  const {
    form,
    routingRule,
    canDelete,
    matchers,
    handleMatchersChange,
    handleModeChange,
  } = useRoutingRulesForm({
    ruleName: isCreate ? undefined : rule?.name,
    rule: rule || undefined,
    onSave,
    onCancel,
  });

  const currentRule = rule || routingRule;
  const formData = form.data as RoutingRuleFormData;

  // Mode options for select
  const modeOptions = [
    { value: 'BLOCK', label: ROUTING_MODE_LABELS.BLOCK },
    { value: 'ALLOW', label: ROUTING_MODE_LABELS.ALLOW },
  ];

  // Check if save should be disabled due to invalid matchers
  const hasBlockingErrors = hasInvalidMatchers(matchers || []);

  // Show loading state
  if (form.isLoading) {
    return (
      <Flex align="center" justify="center" className="routing-rule-form__loading">
        <Loader2 size={24} className="routing-rule-form__spinner" />
        <Text size="2">{CommonStrings.MESSAGES.LOADING_FORM}</Text>
      </Flex>
    );
  }

  return (
    <Box className="routing-rule-form">
      <SettingsForm
        onSubmit={() => form.send('SUBMIT')}
        onCancel={onCancel}
        loading={form.isSaving || loading}
        pristine={form.isPristine}
        error={error || form.saveError || undefined}
        submitLabel={isCreate ? CommonStrings.ACTIONS.CREATE : CommonStrings.ACTIONS.SAVE}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        submitDisabled={hasBlockingErrors}
        confirmDiscard
        testId="routing-rule-form"
        footerExtra={
          !isCreate && onDelete ? (
            <SettingsButton
              testId="form-delete"
              variant="danger"
              onClick={onDelete}
              disabled={form.isSaving || loading || !canDelete}
              icon={Trash2}
              title={
                !canDelete
                  ? plural(RoutingRulesStrings.FORM.CANNOT_DELETE_ASSIGNED, currentRule?.assignedRepositoryCount ?? 0)
                  : RoutingRulesStrings.FORM.DELETE_ROUTING_RULE
              }
            >
              {CommonStrings.ACTIONS.DELETE}
            </SettingsButton>
          ) : undefined
        }
      >
        <SettingsFormSection title={RoutingRulesStrings.FORM.BASIC_INFORMATION}>
          <SettingsTextInput
            {...form.field('name')}
            label={CommonStrings.LABELS.NAME}
            placeholder="e.g., block-sources"
            helpText={RoutingRulesStrings.FORM.UNIQUE_NAME_ROUTING_RULE}
            required
            disabled={!isCreate || form.isSaving || loading}
            readOnly={!isCreate}
          />

          <SettingsTextArea
            {...form.field('description')}
            label={CommonStrings.LABELS.DESCRIPTION}
            placeholder={RoutingRulesStrings.FORM.DESCRIBE_WHAT_ROUTING_RULE_DOES_PLACEHOLDER}
            helpText={RoutingRulesStrings.FORM.OPTIONAL_DESCRIPTION_ROUTING_RULE}
            disabled={form.isSaving || loading}
            rows={3}
          />

          <SettingsSelect
            name="mode"
            label={RoutingRulesStrings.FORM.MODE}
            value={formData.mode || 'BLOCK'}
            onChange={(value) => handleModeChange(value as RoutingMode)}
            options={modeOptions}
            helpText={ROUTING_MODE_HELP[formData.mode || 'BLOCK']}
            required
            disabled={form.isSaving || loading}
          />
        </SettingsFormSection>

        <SettingsFormSection title={RoutingRulesStrings.FORM.MATCHERS}>
          <RoutingRuleMatcher
            matchers={matchers || ['']}
            onChange={handleMatchersChange}
            error={form.touched?.matchers ? form.validationErrors?.matchers : undefined}
            disabled={form.isSaving || loading}
            testPath={testPath}
            testMode={formData.mode || 'BLOCK'}
            onTest={setTestPath}
          />
        </SettingsFormSection>

        {/* Help Section */}
        <Box className="routing-rule-form__help">
          <Flex align="center" gap="2" className="routing-rule-form__help-header">
            <Info size={16} />
            <Text size="2" weight="medium">{RoutingRulesStrings.FORM.ABOUT_ROUTING_RULES}</Text>
          </Flex>
          <Text size="2" className="routing-rule-form__help-text">
            {RoutingRulesStrings.FORM.ABOUT_TEXT}
            {' '}
            <a
              href="http://links.globalart.dev/products/astrum/docs/routing-rule"
              target="_blank"
              rel="noopener noreferrer"
              className="routing-rule-form__help-link"
            >
              {CommonStrings.ACTIONS.LEARN_MORE}
              <ExternalLink size={12} />
            </a>
          </Text>
        </Box>

        {/* Warning for assigned repositories */}
        {!isCreate && (currentRule?.assignedRepositoryCount ?? 0) > 0 && (
          <SettingsAlert type="warning" className="routing-rule-form__warning">
            {plural(RoutingRulesStrings.FORM.ASSIGNED_WARNING, currentRule?.assignedRepositoryCount ?? 0)}{' '}
            {currentRule?.assignedRepositoryNames?.join(', ')}.{' '}
            {RoutingRulesStrings.FORM.UNASSIGN_BEFORE_DELETING}
          </SettingsAlert>
        )}
      </SettingsForm>
    </Box>
  );
}

export default RoutingRuleForm;
