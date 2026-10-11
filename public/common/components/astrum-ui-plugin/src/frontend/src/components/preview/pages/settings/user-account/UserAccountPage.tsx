

import React, { useEffect, useState } from 'react';
import { Box, Button, Card, Flex, Text, Heading, Badge } from '@radix-ui/themes';
import { ArrowRight, Key, Package, User, Loader2, Shield, Mail } from 'lucide-react';
import { ExtJS } from '../../../../../interface/ExtJS';

import {
  SettingsFormSection,
  SettingsPasswordInput,
  SettingsTextInput,
  SettingsAlert,
} from '../../../shared/form';
import { useRouteVisibility } from '../../../shared/Navigation/useRouteVisibility';
import { useUserAccountForm } from './useUserAccountForm';
import { useUserDetailsForm } from './useUserDetailsForm';

import './UserAccountPage.scss';
import UserPagesStrings from '../../../constants/pages/user/UserPagesStrings';
import CommonStrings from '../../../constants/CommonStrings';

interface UserAccountPageProps {
  className?: string;
}

const LOAD_ERROR_MESSAGE = UserPagesStrings.ACCOUNT.FAILED_LOAD_ACCOUNT_INFORMATION;

export function UserAccountPage({ className }: UserAccountPageProps) {
  const user = ExtJS.useUser();
  const form = useUserAccountForm();
  const details = useUserDetailsForm();
  const showUserToken = useRouteVisibility('preview.user.usertoken');
  const showNugetApiKey = useRouteVisibility('preview.user.nugetapitoken');
  const [dismissedError, setDismissedError] = useState(false);

  useEffect(() => {
    if (form.hasLoadError) {
      setDismissedError(false);
    }
  }, [form.hasLoadError]);

  if (!user) {
    return (
      <Box className={`user-account-page ${className || ''}`.trim()}>
        <Flex align="center" gap="3" className="user-account-page__header">
          <User size={24} className="user-account-page__icon" />
          <Box>
            <Heading as="h1" size="6" weight="medium">{UserPagesStrings.ACCOUNT.USER_ACCOUNT}</Heading>
            <Text size="2" className="user-account-page__description">
              {UserPagesStrings.ACCOUNT.MANAGE_ACCOUNT_SETTINGS}
            </Text>
          </Box>
        </Flex>

        <SettingsAlert type="warning">
          {UserPagesStrings.ACCOUNT.YOU_MUST_LOGGED_VIEW_ACCOUNT}
        </SettingsAlert>
      </Box>
    );
  }

  if (form.isLoading || details.isLoading) {
    return (
      <Box className={`user-account-page ${className || ''}`.trim()}>
        <Flex align="center" justify="center" className="user-account-page__loading">
          <Loader2 size={24} className="user-account-page__spinner" />
          <Text size="2">{UserPagesStrings.ACCOUNT.LOADING_ACCOUNT_INFORMATION}</Text>
        </Flex>
      </Box>
    );
  }

  const { data, validationErrors, touched } = form;
  const profileLoaded = !form.hasLoadError && Boolean(data.userId);

  const displayName = data.firstName || data.lastName
    ? `${data.firstName} ${data.lastName}`.trim()
    : UserPagesStrings.ACCOUNT.NOT_SET;
  const displayEmail = data.email || UserPagesStrings.ACCOUNT.NOT_SET;
  const isExternal = data.external;
  const showLoadError = form.hasLoadError && !dismissedError;

  return (
    <Box className={`user-account-page ${className || ''}`.trim()}>
      <Flex align="center" gap="3" className="user-account-page__header">
        <User size={24} className="user-account-page__icon" />
        <Box>
          <Heading as="h1" size="6" weight="medium">{UserPagesStrings.ACCOUNT.USER_ACCOUNT}</Heading>
          <Text size="2" className="user-account-page__description">
            {UserPagesStrings.ACCOUNT.MANAGE_ACCOUNT_SETTINGS}
          </Text>
        </Box>
      </Flex>

      {showLoadError && (
        <Box className="user-account-page__alerts">
          <SettingsAlert type="error" onClose={() => setDismissedError(true)}>
            {LOAD_ERROR_MESSAGE}
          </SettingsAlert>
        </Box>
      )}

      <SettingsFormSection title={UserPagesStrings.ACCOUNT.ACCOUNT_INFORMATION}>
        <Box className="user-account-page__info">
          <Flex className="user-account-page__info-row">
            <User size={16} className="user-account-page__info-icon" />
            <Box>
              <Text size="1" className="user-account-page__info-label">{UserPagesStrings.ACCOUNT.USERNAME}</Text>
              <Text size="2" weight="medium">{data.userId || user.id}</Text>
            </Box>
          </Flex>

          <Flex className="user-account-page__info-row">
            <Shield size={16} className="user-account-page__info-icon" />
            <Box>
              <Text size="1" className="user-account-page__info-label">{UserPagesStrings.ACCOUNT.SOURCE}</Text>
              <Badge variant="soft" color={isExternal ? 'gray' : 'blue'}>
                {isExternal ? UserPagesStrings.ACCOUNT.SOURCE_EXTERNAL : UserPagesStrings.ACCOUNT.SOURCE_LOCAL}
              </Badge>
            </Box>
          </Flex>

          {(!profileLoaded || isExternal) && (
            <>
              <Flex className="user-account-page__info-row">
                <Mail size={16} className="user-account-page__info-icon" />
                <Box>
                  <Text size="1" className="user-account-page__info-label">{UserPagesStrings.ACCOUNT.EMAIL}</Text>
                  <Text size="2" weight="medium">{displayEmail}</Text>
                </Box>
              </Flex>

              <Flex className="user-account-page__info-row">
                <User size={16} className="user-account-page__info-icon" />
                <Box>
                  <Text size="1" className="user-account-page__info-label">{CommonStrings.LABELS.NAME}</Text>
                  <Text size="2" weight="medium">{displayName}</Text>
                </Box>
              </Flex>
            </>
          )}
        </Box>

        {profileLoaded && isExternal && (
          <Text size="2" className="user-account-page__section-description">
            {UserPagesStrings.ACCOUNT.DETAILS_MANAGED_EXTERNALLY}
          </Text>
        )}

        {profileLoaded && !isExternal && (
          <Flex direction="column" gap="3" mt="5" data-testid="account-details-form">
            <Box className="user-account-page__details">
              <SettingsTextInput
                {...details.field('firstName')}
                label={UserPagesStrings.ACCOUNT.FIRST_NAME}
                autoComplete="given-name"
                required
              />
              <SettingsTextInput
                {...details.field('lastName')}
                label={UserPagesStrings.ACCOUNT.LAST_NAME}
                autoComplete="family-name"
                required
              />
            </Box>
            <SettingsTextInput
              {...details.field('email')}
              label={UserPagesStrings.ACCOUNT.EMAIL}
              type="email"
              autoComplete="email"
              required
            />
            <Flex gap="2" mt="2">
              <Button
                type="button"
                variant="solid"
                size="2"
                disabled={details.isPristine || details.hasValidationErrors || details.isSaving}
                onClick={() => details.submit()}
                data-testid="account-details-submit"
              >
                {UserPagesStrings.ACCOUNT.SAVE_DETAILS}
              </Button>
              <Button
                type="button"
                variant="outline"
                size="2"
                disabled={details.isPristine || details.isSaving}
                onClick={() => details.reset()}
                data-testid="account-details-discard"
              >
                {UserPagesStrings.ACCOUNT.DISCARD}
              </Button>
            </Flex>
          </Flex>
        )}
      </SettingsFormSection>

      {profileLoaded && !isExternal && (
        <SettingsFormSection title={UserPagesStrings.ACCOUNT.CHANGE_PASSWORD}>
          <Text size="2" className="user-account-page__section-description">
            {UserPagesStrings.ACCOUNT.UPDATE_ACCOUNT_PASSWORD_PASSWORD_MUST}
          </Text>

          <Flex direction="column" gap="3" data-testid="change-password-fields">
            <SettingsPasswordInput
              name="currentPassword"
              label={UserPagesStrings.ACCOUNT.CURRENT_PASSWORD}
              value={data.currentPassword}
              onChange={(value) => form.send({ type: 'UPDATE', name: 'currentPassword', value })}
              required
            />

            <SettingsPasswordInput
              name="newPassword"
              label={UserPagesStrings.ACCOUNT.NEW_PASSWORD}
              value={data.newPassword}
              onChange={(value) => form.send({ type: 'UPDATE', name: 'newPassword', value })}
              error={touched.newPassword ? validationErrors.newPassword ?? undefined : undefined}
              helpText={UserPagesStrings.ACCOUNT.MINIMUM_8_CHARACTERS}
              required
            />

            <SettingsPasswordInput
              name="confirmPassword"
              label={UserPagesStrings.ACCOUNT.CONFIRM_NEW_PASSWORD}
              value={data.confirmPassword}
              onChange={(value) => form.send({ type: 'UPDATE', name: 'confirmPassword', value })}
              error={touched.confirmPassword ? validationErrors.confirmPassword ?? undefined : undefined}
              required
            />
          </Flex>

          <Flex gap="2" mt="4">
            <Button
              type="button"
              variant="solid"
              size="2"
              disabled={!form.canSubmitPassword || form.isSaving}
              onClick={() => form.submit()}
              data-testid="change-password-submit"
            >
              {UserPagesStrings.ACCOUNT.CHANGE_PASSWORD}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="2"
              disabled={form.isPristine || form.isSaving}
              onClick={() => form.reset()}
              data-testid="change-password-cancel"
            >
              {UserPagesStrings.ACCOUNT.DISCARD}
            </Button>
          </Flex>
        </SettingsFormSection>
      )}

      {profileLoaded && isExternal && (
        <SettingsFormSection title={UserPagesStrings.ACCOUNT.PASSWORD}>
          <SettingsAlert type="info">
            {UserPagesStrings.ACCOUNT.ACCOUNT_MANAGED_EXTERNALLY_PASSWORD_CHANGES}
          </SettingsAlert>
        </SettingsFormSection>
      )}

      {(showUserToken || showNugetApiKey) && (
      <SettingsFormSection title={UserPagesStrings.ACCOUNT.SECURITY}>
        <Flex direction="column" gap="3" data-testid="security-section">
          {showUserToken && (
          <Card size="1" data-testid="user-token-link-card">
            <Flex align="center" justify="between">
              <Flex align="center" gap="3">
                <Key size={20} color="var(--gray-10)" />
                <Box>
                  <Text size="2" weight="medium">{UserPagesStrings.ACCOUNT.USER_TOKEN}</Text>
                  <Text size="1" color="gray" style={{ display: 'block' }}>
                    {UserPagesStrings.ACCOUNT.PERSONAL_ACCESS_TOKEN_API_AUTHENTICATION}
                  </Text>
                </Box>
              </Flex>
              <Button
                variant="ghost"
                size="2"
                onClick={() => { window.location.hash = '#preview/user/usertoken'; }}
                aria-label={UserPagesStrings.ACCOUNT.MANAGE_USER_TOKEN_ARIA}
                data-testid="manage-user-token-btn"
              >
                {UserPagesStrings.ACCOUNT.MANAGE_USER_TOKEN}
                <ArrowRight size={14} />
              </Button>
            </Flex>
          </Card>
          )}

          {showNugetApiKey && (
          <Card size="1" data-testid="nuget-key-link-card">
            <Flex align="center" justify="between">
              <Flex align="center" gap="3">
                <Package size={20} color="var(--gray-10)" />
                <Box>
                  <Text size="2" weight="medium">{UserPagesStrings.ACCOUNT.NUGET_API_KEY}</Text>
                  <Text size="1" color="gray" style={{ display: 'block' }}>
                    {UserPagesStrings.ACCOUNT.API_KEY_NUGET_PACKAGE_FEED}
                  </Text>
                </Box>
              </Flex>
              <Button
                variant="ghost"
                size="2"
                onClick={() => { window.location.hash = '#preview/user/nugetapitoken'; }}
                aria-label={UserPagesStrings.ACCOUNT.MANAGE_NUGET_API_KEY_ARIA}
                data-testid="manage-nuget-key-btn"
              >
                {UserPagesStrings.ACCOUNT.MANAGE_NUGET_API_KEY}
                <ArrowRight size={14} />
              </Button>
            </Flex>
          </Card>
          )}
        </Flex>
      </SettingsFormSection>
      )}
    </Box>
  );
}

export default UserAccountPage;
