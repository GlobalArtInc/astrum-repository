

import React from 'react';
import { Dialog, Flex, Text, Box } from '@radix-ui/themes';
import { X, Check, Ban } from 'lucide-react';

import { RoutingRule, ROUTING_MODE_LABELS } from './types';

import './RoutingRuleMatchModal.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import RoutingRulesStrings from '../../../../constants/pages/settings/repository/RoutingRulesStrings';

export interface RoutingRuleMatchModalProps {
  isOpen: boolean;
  onClose: () => void;
  rule: RoutingRule | null;
  path: string;
}

/**
 * RoutingRuleMatchModal - Displays routing rule details
 *
 * When a user clicks on a routing rule name in the preview, this modal shows:
 * - Rule name and description
 * - Rule mode (Allow/Block)
 * - All configured matcher patterns for the rule
 */
export function RoutingRuleMatchModal({
  isOpen,
  onClose,
  rule,
  path,
}: RoutingRuleMatchModalProps): React.ReactElement {
  if (!rule) return <></>;

  const modeIcon = rule.mode === 'ALLOW' ? Check : Ban;

  return (
    // non-modal: allows users to reference the preview tree while reading rule details
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()} modal={false}>
      <Dialog.Content className="routing-rule-match-modal" aria-describedby={undefined}>
        {/* Header with rule name and close button */}
        <Flex justify="between" align="center" gap="3" className="routing-rule-match-modal__header">
          <Dialog.Title asChild>
            <Text as="h2" size="5" weight="bold">
              {rule.name}
            </Text>
          </Dialog.Title>
          <Dialog.Close asChild>
            <button
              type="button"
              aria-label={CommonStrings.ACTIONS.CLOSE}
              className="routing-rule-match-modal__close"
            >
              <X size={20} />
            </button>
          </Dialog.Close>
        </Flex>

        {/* Modal body */}
        <Box className="routing-rule-match-modal__body">
          {/* Rule description */}
          {rule.description && (
            <Box className="routing-rule-match-modal__section">
              <Text as="p" size="2" weight="medium" className="routing-rule-match-modal__label">
                {CommonStrings.LABELS.DESCRIPTION}
              </Text>
              <Text as="p" size="2" color="gray">
                {rule.description}
              </Text>
            </Box>
          )}

          {/* Rule mode */}
          <Box className="routing-rule-match-modal__section">
            <Text as="p" size="2" weight="medium" className="routing-rule-match-modal__label">
              {RoutingRulesStrings.MATCH_MODAL.MODE}
            </Text>
            <Flex align="center" gap="2" className="routing-rule-match-modal__mode">
              <span className={`routing-rule-match-modal__mode-icon routing-rule-match-modal__mode-icon--${rule.mode.toLowerCase()}`}>
                {React.createElement(modeIcon, { size: 16 })}
              </span>
              <Text as="p" size="2">
                {ROUTING_MODE_LABELS[rule.mode]}
              </Text>
            </Flex>
          </Box>

          {/* Matching path info */}
          <Box className="routing-rule-match-modal__section routing-rule-match-modal__section--path">
            <Text as="p" size="2" weight="medium" className="routing-rule-match-modal__label">
              {RoutingRulesStrings.MATCH_MODAL.MATCHED_PATH}
            </Text>
            <Text as="p" size="2" className="routing-rule-match-modal__path">
              {path}
            </Text>
          </Box>

          {/* Configured matchers - all patterns for this rule */}
          {rule.matchers && rule.matchers.length > 0 && (
            <Box className="routing-rule-match-modal__section routing-rule-match-modal__section--matchers">
              <Text as="p" size="2" weight="medium" className="routing-rule-match-modal__label">
                {RoutingRulesStrings.MATCH_MODAL.CONFIGURED_PATTERNS}
              </Text>
              <Text as="p" size="2" color="gray" className="routing-rule-match-modal__matchers-note">
                {RoutingRulesStrings.MATCH_MODAL.RULE_MATCH_IF_ANY_FOLLOWING}
              </Text>
              <ul className="routing-rule-match-modal__matchers-list">
                {rule.matchers.map((matcher, index) => (
                  <li key={index} className="routing-rule-match-modal__matcher-item">
                    <code className="routing-rule-match-modal__matcher">{matcher}</code>
                  </li>
                ))}
              </ul>
            </Box>
          )}

          {/* Rule name for reference */}
          <Box className="routing-rule-match-modal__section routing-rule-match-modal__section--id">
            <Text as="p" size="2" weight="medium" className="routing-rule-match-modal__label">
              {RoutingRulesStrings.MATCH_MODAL.RULE_NAME}
            </Text>
            <Text as="p" size="2" color="gray" className="routing-rule-match-modal__id">
              {rule.name}
            </Text>
          </Box>
        </Box>

        {/* Footer with close button */}
        <Flex justify="end" className="routing-rule-match-modal__footer">
          <button
            type="button"
            className="routing-rule-match-modal__close-button"
            onClick={onClose}
          >
            {CommonStrings.ACTIONS.CLOSE}
          </button>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

export default RoutingRuleMatchModal;
