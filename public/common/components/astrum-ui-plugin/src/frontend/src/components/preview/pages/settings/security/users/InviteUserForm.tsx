

import React, { useState, useCallback } from 'react';

import {
  SettingsForm,
  SettingsFormSection,
  SettingsTextInput,
} from '../../../../shared/form';
import { useUsersApi, UserInviteData } from './useUsersApi';

interface InviteUserFormProps {
  onSuccess: (email: string) => void;
  onCancel: () => void;
  loading?: boolean;
  error?: string;
}

interface InviteFormState {
  firstName: string;
  lastName: string;
  email: string;
}

interface InviteFormErrors {
  firstName?: string;
  lastName?: string;
  email?: string;
}

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function validate(data: InviteFormState): InviteFormErrors {
  const errors: InviteFormErrors = {};
  if (!data.firstName.trim()) errors.firstName = 'First name is required';
  if (!data.lastName.trim()) errors.lastName = 'Last name is required';
  if (!data.email.trim()) {
    errors.email = 'Email is required';
  } else if (!EMAIL_REGEX.test(data.email)) {
    errors.email = 'Enter a valid email address';
  }
  return errors;
}

/**
 * InviteUserForm - Form for inviting users in cloud distribution.
 *
 * Calls POST /v1/security/users/invite with firstName, lastName, and email.
 * Modelled after UserForm but simplified to only the fields required by the invite endpoint.
 */
export function InviteUserForm({ onSuccess, onCancel, loading: externalLoading = false, error: externalError }: InviteUserFormProps) {
  const { inviteUser, loading: apiLoading, error: apiError } = useUsersApi();

  const [formData, setFormData] = useState<InviteFormState>({
    firstName: '',
    lastName: '',
    email: '',
  });
  const [fieldErrors, setFieldErrors] = useState<InviteFormErrors>({});
  const [submitted, setSubmitted] = useState(false);

  const loading = externalLoading || apiLoading;
  const error = externalError || apiError || undefined;

  const handleChange = useCallback((field: keyof InviteFormState) => (value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    if (submitted) {
      setFieldErrors((prev) => ({ ...prev, [field]: validate({ ...formData, [field]: value })[field] }));
    }
  }, [formData, submitted]);

  const handleSubmit = useCallback(async () => {
    setSubmitted(true);
    const errors = validate(formData);
    setFieldErrors(errors);
    if (Object.keys(errors).length > 0) return;

    const payload: UserInviteData = {
      firstName: formData.firstName.trim(),
      lastName: formData.lastName.trim(),
      email: formData.email.trim(),
    };

    try {
      await inviteUser(payload);
      onSuccess(payload.email);
    } catch {
      // error state is managed by useUsersApi
    }
  }, [formData, inviteUser, onSuccess]);

  const isDirty = formData.firstName.trim() !== '' ||
    formData.lastName.trim() !== '' ||
    formData.email.trim() !== '';

  return (
    <SettingsForm
      testId="invite-user-form"
      onSubmit={handleSubmit}
      onCancel={onCancel}
      loading={loading}
      pristine={false}
      dirty={isDirty}
      confirmDiscard={true}
      noDirtyTracking={!isDirty}
      error={error}
      submitLabel="Invite User"
    >
      <SettingsFormSection title="User Details" defaultOpen>
        <SettingsTextInput
          name="firstName"
          label="First Name"
          placeholder="John"
          helpText="User's first name"
          required
          value={formData.firstName}
          onChange={handleChange('firstName')}
          error={fieldErrors.firstName}
        />

        <SettingsTextInput
          name="lastName"
          label="Last Name"
          placeholder="Smith"
          helpText="User's last name"
          required
          value={formData.lastName}
          onChange={handleChange('lastName')}
          error={fieldErrors.lastName}
        />

        <SettingsTextInput
          name="email"
          label="Email"
          placeholder="jsmith@example.com"
          type="email"
          helpText="Invitation will be sent to this address"
          required
          value={formData.email}
          onChange={handleChange('email')}
          error={fieldErrors.email}
        />
      </SettingsFormSection>
    </SettingsForm>
  );
}

export default InviteUserForm;
