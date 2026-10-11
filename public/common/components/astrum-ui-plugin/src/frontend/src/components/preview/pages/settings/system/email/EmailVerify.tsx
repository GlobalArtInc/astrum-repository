

import React, { useState, useCallback } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Send } from 'lucide-react';

import { SettingsTextInput, SettingsButton, SettingsAlert } from '../../../../shared/form';
import { EmailVerificationResult } from './types';

import './EmailVerify.scss';
import EmailStrings from '../../../../constants/pages/settings/system/EmailStrings';

interface EmailVerifyProps {
  onSendTest: (email: string) => Promise<EmailVerificationResult>;
  loading?: boolean;
  disabled?: boolean;
}

/**
 * EmailVerify - Send test email component
 *
 * Allows users to verify their email configuration by sending a test email.
 */
export function EmailVerify({ onSendTest, loading = false, disabled = false }: EmailVerifyProps) {
  const [email, setEmail] = useState('');
  const [result, setResult] = useState<EmailVerificationResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const isValidEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  const isButtonDisabled = disabled || loading || !isValidEmail;

  const handleSendTest = useCallback(async () => {
    if (!isValidEmail) return;

    setResult(null);
    setError(null);

    try {
      const verificationResult = await onSendTest(email);
      setResult(verificationResult);
    } catch (err: any) {
      setError(err?.message || EmailStrings.VERIFY.FAILED_SEND_TEST_EMAIL);
    }
  }, [email, isValidEmail, onSendTest]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !isButtonDisabled) {
      e.preventDefault();
      handleSendTest();
    }
  }, [isButtonDisabled, handleSendTest]);

  const handleEmailChange = useCallback((value: string) => {
    setEmail(value);
    // Clear previous results when email changes
    setResult(null);
    setError(null);
  }, []);

  return (
    <Box className="email-verify">
      <Text size="2" className="email-verify__description">
        {EmailStrings.VERIFY.SEND_TEST_EMAIL_VERIFY_SMTP}
      </Text>

      <Flex gap="3" align="end" className="email-verify__form">
        <Box className="email-verify__input-wrapper">
          <SettingsTextInput
            name="testEmail"
            label={EmailStrings.VERIFY.TEST_EMAIL_ADDRESS}
            type="email"
            value={email}
            onChange={handleEmailChange}
            onKeyDown={handleKeyDown}
            placeholder="test@example.com"
            helpText={EmailStrings.VERIFY.ENTER_EMAIL_ADDRESS_RECEIVE_TEST}
            disabled={disabled}
          />
        </Box>

        <SettingsButton
          type="button"
          variant="secondary"
          onClick={handleSendTest}
          disabled={isButtonDisabled}
          loading={loading}
          className="email-verify__button"
          icon={Send}
          data-analytics-id="as-email-test"
        >
          {EmailStrings.VERIFY.SEND_TEST}
        </SettingsButton>
      </Flex>

      {/* Results */}
      {result && (
        <Box className="email-verify__result">
          {result.success ? (
            <SettingsAlert type="success">
              <Text>{EmailStrings.VERIFY.TEST_EMAIL_SENT_SUCCESSFULLY_CHECK}</Text>
            </SettingsAlert>
          ) : (
            <SettingsAlert type="error">
              <Text>{EmailStrings.VERIFY.FAILED_SEND_TEST_EMAIL_REASON(result.reason || EmailStrings.VERIFY.UNABLE_VERIFY_CHECK_EMAIL_CONFIGURATION)}</Text>
            </SettingsAlert>
          )}
        </Box>
      )}

      {error && (
        <Box className="email-verify__result">
          <SettingsAlert type="error" onClose={() => setError(null)}>
            {error}
          </SettingsAlert>
        </Box>
      )}
    </Box>
  );
}

export default EmailVerify;
