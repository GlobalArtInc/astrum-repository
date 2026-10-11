

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { Box, Flex, Text, Dialog, TextField, Badge } from '@radix-ui/themes';
import {
  CheckCircle2,
  Loader2,
  Play,
  AlertCircle,
  User,
  Lock,
  KeyRound,
  Trash2,
} from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsCheckbox,
  SettingsSelect,
  SettingsButton,
  SettingsAlert,
  WizardForm,
} from '../../../../shared/form';
import { EntityTable, type TableColumn } from '../../../../shared';
import { ConfirmDialog } from '../shared/ConfirmDialog';
import { useLdapForm } from './useLdapForm';
import { validateConnection, validateUserGroup } from './ldapFormMachine';
import {
  LdapServer,
  LdapFormData,
  LdapSchemaTemplate,
  LdapUser,
  AUTH_SCHEMES,
} from './types';
import type { UseLdapFormOptions } from './useLdapForm';

import './LdapForm.scss';
import LdapStrings from '../../../../constants/pages/settings/security/LdapStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface LdapFormProps {
  server?: LdapServer | null;
  isCreate: boolean;
  existingNames?: string[];
  onSave: (data: LdapFormData) => Promise<void>;
  onCancel: () => void;
  onDelete?: () => void;
  loading?: boolean;
  error?: string;
  /** API functions from the single shared useLdapApi instance owned by LdapPage */
  fetchTemplates: () => Promise<LdapSchemaTemplate[]>;
  createServer: UseLdapFormOptions['createServer'];
  updateServer: UseLdapFormOptions['updateServer'];
  verifyConnection: (data: LdapFormData, existingServerName?: string) => Promise<void>;
  verifyUserMapping: (data: LdapFormData, existingServerName?: string) => Promise<LdapUser[]>;
  verifyLogin: (data: LdapFormData, username: string, password: string, existingServerName?: string) => Promise<void>;
}

const WIZARD_STEPS = [
  { id: 'connection', label: LdapStrings.FORM.CONNECTION },
  { id: 'userGroup', label: LdapStrings.FORM.USER_GROUP },
];

/**
 * Fields that identify what the LDAP server connects to and as whom:
 * protocol, host, port, searchBase, useTrustStore, authScheme, authUsername,
 * authPassword, and authRealm (the SASL realm for DIGEST-MD5/CRAM-MD5 -
 * still part of "as whom"). Changing any of these invalidates a prior
 * "Connection successful" verification, so the "verified" badge must be
 * cleared. connectionTimeout, connectionRetryDelay, maxIncidentsCount, and
 * name are intentionally excluded: they don't change what's being connected
 * to/as, so clearing the badge on their change would just prompt spurious
 * re-verification.
 */
const CONNECTION_IDENTITY_FIELDS = new Set<keyof LdapFormData>([
  'protocol',
  'host',
  'port',
  'searchBase',
  'useTrustStore',
  'authScheme',
  'authUsername',
  'authPassword',
  'authRealm',
]);

/**
 * Columns for the Verify User Mapping results table, mirroring the legacy
 * ExtJS reference grid (LdapServerUserAndGroupMappingTestResults.js). There
 * is intentionally no row cap here - EntityTable has no built-in pagination
 * and the ExtJS reference doesn't cap results either, so every verified
 * user is shown.
 */
const USER_MAPPING_COLUMNS: TableColumn<LdapUser>[] = [
  { id: 'username', header: 'ID', accessor: 'username' },
  { id: 'realName', header: CommonStrings.LABELS.NAME, accessor: (u) => u.realName || '' },
  { id: 'email', header: LdapStrings.FORM.EMAIL, accessor: (u) => u.email || '' },
  {
    id: 'membership',
    header: LdapStrings.FORM.ROLES,
    accessor: (u) => (u.membership?.length ? u.membership.join(', ') : ''),
  },
];

/**
 * LdapForm - Multi-step wizard form for creating/editing LDAP servers.
 * Uses shared WizardForm component with step indicator and navigation.
 */
export function LdapForm({
  server,
  isCreate,
  existingNames,
  onSave,
  onCancel,
  onDelete,
  loading = false,
  error,
  fetchTemplates,
  createServer,
  updateServer,
  verifyConnection,
  verifyUserMapping,
  verifyLogin,
}: LdapFormProps) {
  // Verify* endpoints require astrum:ldap:update regardless of create/edit mode
  // (LdapApiResource.java has no separate create-time verify permission), but
  // field editability and Save/Create should follow the permission for the
  // action actually being performed — mirrors LdapServerConnectionAdd.js's
  // editableCondition override of the shared form's astrum:ldap:update default.
  const canUpdate = ExtJS.checkPermission('astrum:ldap:update');
  const canEditFields = isCreate ? ExtJS.checkPermission('astrum:ldap:create') : canUpdate;

  // Use XState form hook
  const { form, applyTemplate: hookApplyTemplate, changeProtocol, confirmPasswordAndSubmit } = useLdapForm({
    serverId: isCreate ? undefined : server?.id,
    server: server || null,
    existingNames,
    onSave,
    onCancel,
    createServer,
    updateServer,
  });

  const formData = form.data as LdapFormData;

  // Presentation-only raw strings for the three numeric connection settings fields.
  // Using type="text" (not type="number") prevents the browser from showing its
  // native invalid-number indicator (cursor shift / shrink) when the user types a
  // letter. Storing the raw string lets the machine receive NaN for non-numeric
  // input, which the validator in ldapFormMachine.ts catches and surfaces as a
  // range message, distinct from undefined which means "This field is required".
  // Mirrors the timeoutRaw pattern in CrowdPage.tsx.
  const [connectionTimeoutRaw, setConnectionTimeoutRaw] = useState<string>(() =>
    formData.connectionTimeout != null && !Number.isNaN(formData.connectionTimeout)
      ? String(formData.connectionTimeout) : ''
  );
  const [connectionRetryDelayRaw, setConnectionRetryDelayRaw] = useState<string>(() =>
    formData.connectionRetryDelay != null && !Number.isNaN(formData.connectionRetryDelay)
      ? String(formData.connectionRetryDelay) : ''
  );
  const [maxIncidentsCountRaw, setMaxIncidentsCountRaw] = useState<string>(() =>
    formData.maxIncidentsCount != null && !Number.isNaN(formData.maxIncidentsCount)
      ? String(formData.maxIncidentsCount) : ''
  );

  // Re-sync the raw strings when the form resets to pristine (after a successful
  // save, discard, or initial load from the server) so the displayed values
  // track the machine state. While the user is actively editing (isPristine=false)
  // the raw strings are authoritative and must not be overwritten.
  useEffect(() => {
    if (!form.isPristine) return;
    setConnectionTimeoutRaw(
      formData.connectionTimeout != null && !Number.isNaN(formData.connectionTimeout)
        ? String(formData.connectionTimeout) : ''
    );
    setConnectionRetryDelayRaw(
      formData.connectionRetryDelay != null && !Number.isNaN(formData.connectionRetryDelay)
        ? String(formData.connectionRetryDelay) : ''
    );
    setMaxIncidentsCountRaw(
      formData.maxIncidentsCount != null && !Number.isNaN(formData.maxIncidentsCount)
        ? String(formData.maxIncidentsCount) : ''
    );
  }, [form.isPristine, formData.connectionTimeout, formData.connectionRetryDelay, formData.maxIncidentsCount]);

  const [currentStep, setCurrentStep] = useState(0);
  const [templates, setTemplates] = useState<LdapSchemaTemplate[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<string>('');
  const [templateError, setTemplateError] = useState<string | null>(null);

  // Verification state
  // Connection and user-mapping verification each get their own flag so that
  // clicking one Verify button does not disable/spin the other buttons (F6) -
  // loginVerifying below is already independent, and the Verify Login button
  // itself only opens a modal (synchronous), so it needs no verifying flag.
  const [connectionVerifying, setConnectionVerifying] = useState(false);
  const [userMappingVerifying, setUserMappingVerifying] = useState(false);
  const [connectionVerified, setConnectionVerified] = useState(false);
  const [verificationError, setVerificationError] = useState<string | null>(null);
  const [verifiedUsers, setVerifiedUsers] = useState<LdapUser[]>([]);
  const [showUserResults, setShowUserResults] = useState(false);
  const userResultsRef = useRef<HTMLDivElement>(null);
  // A11y (ASTRUM-53625 A4): persistent visually-hidden live region announcer for
  // the connection / user-mapping verification outcomes. The region is mounted
  // before any verification runs; each announcement bumps `key` so React
  // inserts a fresh node into the live region — VoiceOver announces added nodes
  // reliably, whereas swapping text in place (or mounting a region together
  // with its content) is frequently missed. See ASTRUM-53625 comment #887700.
  const [formStatus, setFormStatus] = useState<{ message: string; key: number }>({ message: '', key: 0 });
  const announceStatus = useCallback((message: string) => {
    setFormStatus((prev) => ({ message, key: prev.key + 1 }));
  }, []);
  // Same technique for the Verify Login modal result (a separate region is
  // required because it lives inside the aria-modal Dialog subtree).
  const [loginAnnounceKey, setLoginAnnounceKey] = useState(0);
  // Return focus to the Test Login button after a verification completes.
  // While verifying, the button is disabled (loading), so the browser drops
  // focus and the Dialog pulls it back to the title — disorienting for
  // keyboard/SR users. Re-focus the button once it is enabled again.
  const loginSubmitRef = useRef<HTMLButtonElement>(null);

  // Verify Login modal state
  const [showLoginModal, setShowLoginModal] = useState(false);
  const [loginUsername, setLoginUsername] = useState('');
  const [loginPassword, setLoginPassword] = useState('');
  const [loginResult, setLoginResult] = useState<{ success: boolean; message: string } | null>(null);
  const [loginVerifying, setLoginVerifying] = useState(false);

  // Password re-entry modal state (ASTRUM-23184).
  // pendingVerifyAction tracks whether the modal was opened from a verify button
  // ('connection' | 'userMapping') or from the save flow (null). The submit
  // handler branches on this to either run verification or commit the save.
  const [showPasswordModal, setShowPasswordModal] = useState(false);
  const [pendingVerifyAction, setPendingVerifyAction] = useState<'connection' | 'userMapping' | null>(null);
  const [reenteredPassword, setReenteredPassword] = useState('');
  const [passwordError, setPasswordError] = useState<string | null>(null);
  // ASTRUM-53959: verify the re-entered password with an LDAP bind before
  // letting Save proceed. Without this, the modal was a security no-op — any
  // string was accepted and the update PUT went through regardless.
  const [passwordVerifying, setPasswordVerifying] = useState(false);

  // Load templates
  useEffect(() => {
    // Clear any stale error before the (re-)fetch: the JSX renders the error branch before the
    // template dropdown, so a leftover error from a previous run would hide a now-successful load.
    let cancelled = false;
    setTemplateError(null);
    fetchTemplates()
      .then((t) => { if (!cancelled) setTemplates(t); })
      .catch((err: any) => { if (!cancelled) setTemplateError(err.message || LdapStrings.FORM.FAILED_LOAD_TEMPLATES); });
    return () => { cancelled = true; };
  }, [fetchTemplates]);

  useEffect(() => {
    if (showUserResults && userResultsRef.current) {
      userResultsRef.current.scrollIntoView?.({ behavior: 'smooth', block: 'start' });
    }
  }, [showUserResults]);

  // Helper to update form field via machine
  const handleChange = useCallback(<K extends keyof LdapFormData>(
    field: K,
    value: LdapFormData[K]
  ) => {
    if (field === 'protocol') {
      changeProtocol(value as string);
    } else {
      form.send({ type: 'UPDATE', name: field as string, value });
    }

    if (CONNECTION_IDENTITY_FIELDS.has(field)) {
      setConnectionVerified(false);
      // Clear the stale verification announcement so it is not left in the
      // live region after the verified state has been invalidated.
      setFormStatus((prev) => (prev.message ? { message: '', key: prev.key + 1 } : prev));
    }
  }, [form, changeProtocol]);

  // Handle delete with confirmation
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleDeleteConfirm = useCallback(() => {
    if (onDelete) {
      onDelete();
    }
    setShowDeleteConfirm(false);
  }, [onDelete]);

  // Check if connection step is valid (used for save/continue button gating)
  const connectionValid = useMemo(() => {
    const stepErrors = validateConnection(formData, isCreate, existingNames);
    return Object.keys(stepErrors).length === 0;
  }, [formData, isCreate, existingNames]);

  // Check if user/group step is valid
  const userGroupValid = useMemo(() => {
    const stepErrors = validateUserGroup(formData);
    return Object.keys(stepErrors).length === 0;
  }, [formData]);

  // Apply template
  const handleApplyTemplate = useCallback((templateName: string) => {
    const template = templates.find((t) => t.name === templateName);
    if (template) {
      hookApplyTemplate(template);
      setSelectedTemplate(templateName);
    }
  }, [templates, hookApplyTemplate]);

  // Internal runner: executes verifyConnection with the given data. Separated
  // from the button handler so the password-modal path can call it with an
  // augmented copy of formData that includes the modal-supplied password.
  const runVerifyConnection = useCallback(async (data: LdapFormData) => {
    const stepErrors = validateConnection(data, isCreate);
    if (Object.keys(stepErrors).length > 0) return;
    setConnectionVerifying(true);
    setVerificationError(null);
    try {
      await verifyConnection(data, isCreate ? undefined : server?.name);
      setConnectionVerified(true);
      announceStatus(LdapStrings.FORM.CONNECTION_SUCCESSFUL);
    } catch (err: any) {
      const message = err.message || LdapStrings.FORM.CONNECTION_VERIFICATION_FAILED;
      setVerificationError(message);
      announceStatus(message);
    } finally {
      setConnectionVerifying(false);
    }
  }, [verifyConnection, isCreate, server?.name, announceStatus]);

  // Internal runner: executes verifyUserMapping with the given data.
  const runVerifyUserMapping = useCallback(async (data: LdapFormData) => {
    const stepErrors = validateUserGroup(data);
    if (Object.keys(stepErrors).length > 0) return;
    setUserMappingVerifying(true);
    setVerificationError(null);
    setShowUserResults(false);
    try {
      const users = await verifyUserMapping(data, isCreate ? undefined : server?.name);
      setVerifiedUsers(users);
      setShowUserResults(true);
      announceStatus(`Found ${users.length} user(s)`);
    } catch (err: any) {
      const message = err.message || LdapStrings.FORM.USER_MAPPING_VERIFICATION_FAILED;
      setVerificationError(message);
      announceStatus(message);
    } finally {
      setUserMappingVerifying(false);
    }
  }, [verifyUserMapping, isCreate, server?.name, announceStatus]);

  // Verify connection — in edit mode with non-anonymous auth the user must
  // supply credentials via the password modal before the request fires. This
  // mirrors the Classic UI's LdapSystemPasswordModal pattern (ASTRUM-54062).
  const handleVerifyConnection = useCallback(async () => {
    if (!isCreate && formData.authScheme !== 'none') {
      setReenteredPassword('');
      setPasswordError(null);
      setPendingVerifyAction('connection');
      setShowPasswordModal(true);
      return;
    }

    const stepErrors = validateConnection(formData, isCreate);
    if (Object.keys(stepErrors).length > 0) {
      return;
    }

    await runVerifyConnection(formData);
  }, [formData, isCreate, runVerifyConnection]);

  // Verify user mapping — same password-modal gate as handleVerifyConnection.
  const handleVerifyUserMapping = useCallback(async () => {
    if (!isCreate && formData.authScheme !== 'none') {
      setReenteredPassword('');
      setPasswordError(null);
      setPendingVerifyAction('userMapping');
      setShowPasswordModal(true);
      return;
    }

    const stepErrors = validateUserGroup(formData);
    if (Object.keys(stepErrors).length > 0) {
      return;
    }

    await runVerifyUserMapping(formData);
  }, [formData, isCreate, runVerifyUserMapping]);

  // Verify login
  const handleOpenLoginModal = useCallback(() => {
    setLoginUsername('');
    setLoginPassword('');
    setLoginResult(null);
    setShowLoginModal(true);
  }, []);

  const handleVerifyLogin = useCallback(async () => {
    if (!(loginUsername && loginPassword)) return;

    setLoginVerifying(true);
    setLoginResult(null);
    try {
      await verifyLogin(formData, loginUsername, loginPassword, isCreate ? undefined : server?.name);
      setLoginResult({ success: true, message: LdapStrings.FORM.LDAP_LOGIN_COMPLETED_SUCCESSFULLY((formData.protocol || 'ldap').toLowerCase(), formData.host, formData.port) });
      setLoginAnnounceKey((k) => k + 1);
    } catch (err: any) {
      setLoginResult({ success: false, message: err.message || LdapStrings.FORM.LOGIN_VERIFICATION_FAILED });
      setLoginAnnounceKey((k) => k + 1);
    } finally {
      setLoginVerifying(false);
    }
  }, [formData, loginUsername, loginPassword, verifyLogin, isCreate, server?.name]);

  // A11y (ASTRUM-53625): once a login verification finishes, the Test Login
  // button is re-enabled — restore focus to it so keyboard/SR focus doesn't
  // stay stranded on the Dialog title (where the browser sent it when the
  // button became disabled during the request).
  useEffect(() => {
    if (showLoginModal && !loginVerifying && loginResult) {
      loginSubmitRef.current?.focus();
    }
  }, [showLoginModal, loginVerifying, loginResult]);

  // Handle password entry from the modal. Routes to the verify runner (when
  // opened from a verify button) or to the save flow (when opened from Save).
  const handlePasswordReentry = useCallback(async () => {
    if (!reenteredPassword) {
      setPasswordError(LdapStrings.FORM.PASSWORD_REQUIRED);
      return;
    }

    setPasswordError(null);

    if (pendingVerifyAction === 'connection') {
      // Verify flow: close modal immediately, then run with entered password.
      setShowPasswordModal(false);
      const password = reenteredPassword;
      setReenteredPassword('');
      setPendingVerifyAction(null);
      runVerifyConnection({ ...formData, authPassword: password });
    } else if (pendingVerifyAction === 'userMapping') {
      setShowPasswordModal(false);
      const password = reenteredPassword;
      setReenteredPassword('');
      setPendingVerifyAction(null);
      runVerifyUserMapping({ ...formData, authPassword: password });
    } else {
      // Save flow (ASTRUM-53959): bind against LDAP with the re-entered password
      // BEFORE submitting. If the bind fails, the modal stays open with the
      // backend's error message and the update PUT is never issued.
      setPasswordVerifying(true);
      try {
        // Server name is present in edit mode - that's the only path that opens
        // this modal (handleSave gates on !isCreate). Pass it so the backend
        // scopes verify to this server's identity.
        await verifyConnection(
          { ...formData, authPassword: reenteredPassword },
          server?.name
        );
      } catch (err: any) {
        setPasswordError(err?.message || LdapStrings.FORM.CONNECTION_VERIFICATION_FAILED);
        setPasswordVerifying(false);
        return;
      }
      setPasswordVerifying(false);
      // Assign the re-entered password and submit in one synchronous machine
      // transition, so there is no window where the password hasn't been
      // applied yet when SUBMIT is processed.
      confirmPasswordAndSubmit(reenteredPassword);
      setShowPasswordModal(false);
      setReenteredPassword('');
    }
  }, [reenteredPassword, confirmPasswordAndSubmit, pendingVerifyAction, formData, runVerifyConnection, runVerifyUserMapping, verifyConnection, server?.name]);

  const handleStepChange = useCallback((step: number) => {
    if (step > currentStep) {
      const stepErrors = validateConnection(formData, isCreate, existingNames);
      if (Object.keys(stepErrors).length > 0) {
        return;
      }
      setVerificationError(null);
      // Do NOT clear connectionVerified here - CONNECTION_IDENTITY_FIELDS
      // (via handleChange) is the single source of truth for invalidating a
      // verified badge (ASTRUM-53623 F2). Clearing it on every forward
      // navigation regardless of whether anything changed meant clicking
      // Continue immediately dropped the badge, so returning via Back showed
      // an unverified connection the user had, in fact, just verified.
    } else {
      setVerificationError(null);
      setShowUserResults(false);
    }
    setCurrentStep(step);
  }, [currentStep, formData, isCreate, existingNames]);

  const handleSave = useCallback(() => {
    const allErrors = {
      ...validateConnection(formData, isCreate, existingNames),
      ...validateUserGroup(formData),
    };

    if (Object.keys(allErrors).length > 0) {
      return;
    }

    // For edit mode with auth scheme requiring password, show password re-entry modal.
    // This only triggers when authPassword is currently empty - the common
    // case, since the API never returns a stored password. If the user has
    // instead typed a new password directly into the Password field,
    // formData.authPassword is already non-empty and this falls through to
    // the direct SUBMIT below with no modal - correctly, since the value is
    // already in machine state and there's no UPDATE/SUBMIT race to guard
    // against (that race, fixed by CONFIRM_PASSWORD_AND_SUBMIT per F8, only
    // existed for the modal's own re-entry flow).
    if (!isCreate && formData.authScheme !== 'none' && !formData.authPassword) {
      setReenteredPassword('');
      setPasswordError(null);
      // pendingVerifyAction intentionally NOT set → null = save flow in handlePasswordReentry
      setShowPasswordModal(true);
      return;
    }

    form.send('SUBMIT');
  }, [formData, isCreate, existingNames, form]);

  // Show loading state
  if (form.isLoading) {
    return (
      <Flex align="center" justify="center" gap="2" p="4">
        <Loader2 size={24} aria-hidden="true" />
        <Text size="2">{CommonStrings.MESSAGES.LOADING_FORM}</Text>
      </Flex>
    );
  }

  const isStep1 = currentStep === 0;

  return (
    <Box
      className="ldap-form"
      data-testid="ldap-form"
      data-loading={loading || form.isSaving ? 'true' : 'false'}
      data-dirty={!form.isPristine ? 'true' : 'false'}
      data-mode={isCreate ? 'create' : 'edit'}
      data-step={isStep1 ? 'connection' : 'userGroup'}
    >
      {/* Permission Warning — mirrors CrowdPage.tsx / SamlPage.tsx's read-only banner */}
      {!canEditFields && (
        <Box className="ldap-form__permission-warning" mb="3">
          <SettingsAlert type="warning">
            {isCreate
              ? LdapStrings.FORM.YOU_DONT_HAVE_PERMISSION_CREATE
              : LdapStrings.FORM.YOU_DONT_HAVE_PERMISSION_EDIT}
          </SettingsAlert>
        </Box>
      )}

      {/* A11y (ASTRUM-53625 A4): visually-hidden alert for the Connection / User
          Mapping verification outcomes. The role="alert" node is *mounted
          fresh* on each announcement (conditional render, keyed by the
          announcement counter) rather than kept persistent with its text
          swapped in place: VoiceOver reliably announces an alert element that
          is newly inserted into the DOM, but frequently misses a text change
          inside an already-present alert/live region. role="alert" also makes
          it assertive, so the result interrupts instead of queueing behind
          whatever VoiceOver is currently describing. */}
      {formStatus.message && (
        <Box
          key={formStatus.key}
          className="ldap-form__sr-status"
          role="alert"
          aria-live="assertive"
          aria-atomic="true"
          data-testid="ldap-user-mapping-status"
        >
          {formStatus.message}
        </Box>
      )}

      <WizardForm
        steps={WIZARD_STEPS}
        currentStep={currentStep}
        onStepChange={handleStepChange}
        onComplete={handleSave}
        onCancel={onCancel}
        completeLabel={isCreate ? CommonStrings.ACTIONS.CREATE : CommonStrings.ACTIONS.SAVE}
        canAdvance={isStep1 ? connectionValid : (connectionValid && userGroupValid && canEditFields)}
        dirty={!form.isPristine}
        loading={form.isSaving || loading}
        error={error}
        noDirtyTracking={form.isPristine}
        testId="ldap-wizard-form"
        submitAnalyticsId={currentStep < WIZARD_STEPS.length - 1 ? 'as-ldap-form-next' : (isCreate ? 'as-ldap-form-create' : 'as-ldap-form-save')}
        cancelAnalyticsId="as-ldap-form-cancel"
        backAnalyticsId="as-ldap-form-back"
        footerExtra={
          !isCreate && onDelete ? (
            <SettingsButton
              variant="danger"
              icon={Trash2}
              onClick={() => setShowDeleteConfirm(true)}
              disabled={loading || form.isSaving}
              testId="form-delete"
              data-analytics-id="as-ldap-form-delete"
            >
              {CommonStrings.ACTIONS.DELETE}
            </SettingsButton>
          ) : undefined
        }
      >
        {/* Step 1: Connection */}
        {isStep1 && (<>
          <SettingsFormSection title={LdapStrings.FORM.SERVER_CONFIGURATION} defaultOpen>
            <SettingsTextInput
              {...form.field('name')}
              label={CommonStrings.LABELS.NAME}
              helpText={LdapStrings.FORM.UNIQUE_NAME_LDAP_SERVER}
              required
              disabled={!canEditFields}
            />

            <Flex gap="3" className="ldap-form__field-row">
              <Box className="ldap-form__field-col ldap-form__field-col--protocol">
                <SettingsSelect
                  name="protocol"
                  label={LdapStrings.FORM.PROTOCOL}
                  value={(formData.protocol || 'ldap').toLowerCase()}
                  onChange={(value) => handleChange('protocol', value as 'ldap' | 'ldaps')}
                  helpText={LdapStrings.FORM.CONNECTION_PROTOCOL_USE_LDAPS_ENCRYPTED}
                  required
                  disabled={!canEditFields}
                  options={[
                    { value: 'ldap', label: 'ldap' },
                    { value: 'ldaps', label: 'ldaps (SSL)' },
                  ]}
                />
              </Box>
              <Box className="ldap-form__field-col ldap-form__field-col--host">
                <SettingsTextInput
                  {...form.field('host')}
                  onChange={(value) => handleChange('host', value as string)}
                  label={LdapStrings.FORM.HOSTNAME}
                  placeholder="ldap.example.com"
                  helpText={LdapStrings.FORM.LDAP_SERVER_HOSTNAME_IP_ADDRESS}
                  required
                  disabled={!canEditFields}
                />
              </Box>
              <Box className="ldap-form__field-col ldap-form__field-col--port">
                <SettingsTextInput
                  name="port"
                  label={LdapStrings.FORM.PORT}
                  type="number"
                  value={formData.port ?? 389}
                  onChange={(value) => {
                    const parsed = parseInt(value as any, 10);
                    // Port intentionally reverts to 389 on clear — unlike the connection-
                    // timeout/retry/incidents fields (ASTRUM-54076), an empty port is not a
                    // meaningful intermediate state and the backend rejects a missing port.
                    handleChange('port', Number.isNaN(parsed) ? 389 : parsed);
                  }}
                  helpText={LdapStrings.FORM.DEFAULT_389_LDAP_636_LDAPS}
                  error={form.validationErrors.port ?? undefined}
                  required
                  disabled={!canEditFields}
                />
              </Box>
            </Flex>

            {formData.protocol === 'ldaps' && (
              <SettingsCheckbox
                name="useTrustStore"
                label={LdapStrings.FORM.USE_ASTRUM_SSL_TRUST_STORE}
                checked={formData.useTrustStore}
                onChange={(checked) => handleChange('useTrustStore', checked)}
                description={LdapStrings.FORM.USE_ASTRUM_TRUST_STORE_SSL}
                disabled={!canEditFields}
              />
            )}

            <SettingsTextInput
              {...form.field('searchBase')}
              onChange={(value) => handleChange('searchBase', value as string)}
              label={LdapStrings.FORM.SEARCH_BASE_DN}
              placeholder="dc=example,dc=com"
              helpText={LdapStrings.FORM.BASE_DN_SEARCH_USERS}
              required
              disabled={!canEditFields}
            />
          </SettingsFormSection>

          <SettingsFormSection title={LdapStrings.FORM.AUTHENTICATION} defaultOpen>
            <SettingsSelect
              name="authScheme"
              label={LdapStrings.FORM.AUTHENTICATION_METHOD}
              value={formData.authScheme || 'simple'}
              onChange={(value) => handleChange('authScheme', value)}
              required
              disabled={!canEditFields}
              options={AUTH_SCHEMES.map((scheme) => ({
                value: scheme.value,
                label: scheme.label,
              }))}
            />

            {formData.authScheme !== 'none' && (
              <>
                <SettingsTextInput
                  {...form.field('authUsername')}
                  onChange={(value) => handleChange('authUsername', value as string)}
                  label={LdapStrings.FORM.USERNAME}
                  placeholder="cn=admin,dc=example,dc=com"
                  helpText={LdapStrings.FORM.DN_USER_BIND_SEARCHES}
                  required
                  disabled={!canEditFields}
                />

                <SettingsPasswordInput
                  {...form.field('authPassword')}
                  onChange={(value) => handleChange('authPassword', value as string)}
                  label={LdapStrings.FORM.PASSWORD}
                  helpText={isCreate ? LdapStrings.FORM.PASSWORD_BIND_USER : LdapStrings.FORM.LEAVE_BLANK_KEEP_EXISTING_PASSWORD}
                  required={isCreate}
                  disabled={!canEditFields}
                />

                {(formData.authScheme === 'DIGEST-MD5' || formData.authScheme === 'CRAM-MD5') && (
                  <SettingsTextInput
                    {...form.field('authRealm')}
                    onChange={(value) => handleChange('authRealm', value as string)}
                    label={LdapStrings.FORM.SASL_REALM}
                    helpText={LdapStrings.FORM.SASL_REALM_DIGEST_MD5_CRAM}
                    disabled={!canEditFields}
                  />
                )}
              </>
            )}
          </SettingsFormSection>

          <SettingsFormSection title={LdapStrings.FORM.CONNECTION_SETTINGS}>
            <Flex gap="3" className="ldap-form__field-row">
              <SettingsTextInput
                name="connectionTimeout"
                label={LdapStrings.FORM.CONNECTION_TIMEOUT_SECONDS}
                type="text"
                value={connectionTimeoutRaw}
                onChange={(value) => {
                  const val = value as string;
                  setConnectionTimeoutRaw(val);
                  handleChange('connectionTimeout', val.trim() === '' ? undefined : Number(val));
                }}
                helpText="1-3600 seconds"
                disabled={!canEditFields}
                error={form.validationErrors.connectionTimeout ?? undefined}
              />

              <SettingsTextInput
                name="connectionRetryDelay"
                label={LdapStrings.FORM.RETRY_DELAY_SECONDS}
                type="text"
                value={connectionRetryDelayRaw}
                onChange={(value) => {
                  const val = value as string;
                  setConnectionRetryDelayRaw(val);
                  handleChange('connectionRetryDelay', val.trim() === '' ? undefined : Number(val));
                }}
                helpText={LdapStrings.FORM.DELAY_BETWEEN_CONNECTION_RETRIES}
                disabled={!canEditFields}
                error={form.validationErrors.connectionRetryDelay ?? undefined}
              />

              <SettingsTextInput
                name="maxIncidentsCount"
                label={LdapStrings.FORM.MAX_INCIDENTS}
                type="text"
                value={maxIncidentsCountRaw}
                onChange={(value) => {
                  const val = value as string;
                  setMaxIncidentsCountRaw(val);
                  handleChange('maxIncidentsCount', val.trim() === '' ? undefined : Number(val));
                }}
                helpText={LdapStrings.FORM.MAX_FAILURES_BEFORE_BLACKLISTING}
                disabled={!canEditFields}
                error={form.validationErrors.maxIncidentsCount ?? undefined}
              />
            </Flex>
          </SettingsFormSection>

          <Box className="ldap-form__verification">
            <Flex align="center" gap="3">
              <SettingsButton
                variant="secondary"
                onClick={handleVerifyConnection}
                disabled={connectionVerifying || !connectionValid || !canUpdate}
                loading={connectionVerifying}
                icon={Play}
                data-analytics-id="as-ldap-form-verify-connection"
              >
                {LdapStrings.FORM.VERIFY_CONNECTION}
              </SettingsButton>

              {connectionVerified && (
                <Flex align="center" gap="2" className="ldap-form__success">
                  <CheckCircle2 size={16} aria-hidden="true" />
                  <Text size="2">{LdapStrings.FORM.CONNECTION_SUCCESSFUL}</Text>
                </Flex>
              )}
            </Flex>

            {verificationError && (
              <Box className="ldap-form__verification-error">
                <SettingsAlert type="error">{verificationError}</SettingsAlert>
              </Box>
            )}
          </Box>

        </>
        )}

        {/* Step 2: User & Group */}
        {!isStep1 && (
        <>
          {/* Template Selection */}
          {templateError ? (
            <Box className="ldap-form__template">
              <SettingsAlert type="error">{templateError}</SettingsAlert>
            </Box>
          ) : templates.length > 0 && (
            <Box className="ldap-form__template">
              <SettingsSelect
                name="template"
                label={LdapStrings.FORM.LOAD_TEMPLATE}
                value={selectedTemplate}
                onChange={handleApplyTemplate}
                helpText={LdapStrings.FORM.SELECT_TEMPLATE_PRE_FILL_COMMON}
                placeholder={LdapStrings.FORM.SELECT_TEMPLATE_PLACEHOLDER}
                disabled={!canEditFields}
                options={templates.map((template) => ({
                  value: template.name,
                  label: template.name,
                }))}
              />
            </Box>
          )}

          <SettingsFormSection title={LdapStrings.FORM.USER_MAPPING} defaultOpen>
            <SettingsTextInput
              {...form.field('userBaseDn')}
              label={LdapStrings.FORM.USER_RELATIVE_DN}
              placeholder="ou=users"
              helpText={LdapStrings.FORM.RELATIVE_DN_WHERE_USER_OBJECTS}
              disabled={!canEditFields}
            />

            <SettingsCheckbox
              name="userSubtree"
              label={LdapStrings.FORM.SEARCH_SUBTREE}
              checked={formData.userSubtree}
              onChange={(checked) => handleChange('userSubtree', checked)}
              description={LdapStrings.FORM.SEARCH_ENTIRE_SUBTREE_USERS}
              disabled={!canEditFields}
            />

            <SettingsTextInput
              {...form.field('userObjectClass')}
              label={LdapStrings.FORM.OBJECT_CLASS}
              placeholder="inetOrgPerson"
              helpText={LdapStrings.FORM.LDAP_OBJECT_CLASS_USERS_E}
              required
              disabled={!canEditFields}
            />

            <SettingsTextInput
              {...form.field('userLdapFilter')}
              label={LdapStrings.FORM.LDAP_FILTER}
              placeholder="(memberOf=cn=users,dc=example,dc=com)"
              helpText={LdapStrings.FORM.OPTIONAL_ADDITIONAL_FILTER_USERS}
              disabled={!canEditFields}
            />

            <Flex gap="3" className="ldap-form__field-row">
              <SettingsTextInput
                {...form.field('userIdAttribute')}
                label={LdapStrings.FORM.USER_ID_ATTRIBUTE}
                placeholder="uid"
                helpText={LdapStrings.FORM.ATTRIBUTE_CONTAINING_USERNAME_E_G}
                required
                disabled={!canEditFields}
              />

              <SettingsTextInput
                {...form.field('userRealNameAttribute')}
                label={LdapStrings.FORM.REAL_NAME_ATTRIBUTE}
                placeholder="cn"
                helpText={LdapStrings.FORM.ATTRIBUTE_CONTAINING_DISPLAY_NAME_E}
                required
                disabled={!canEditFields}
              />
            </Flex>

            <Flex gap="3" className="ldap-form__field-row">
              <SettingsTextInput
                {...form.field('userEmailAddressAttribute')}
                label={LdapStrings.FORM.EMAIL_ATTRIBUTE}
                placeholder="mail"
                required
                disabled={!canEditFields}
              />

              <SettingsTextInput
                {...form.field('userPasswordAttribute')}
                label={LdapStrings.FORM.PASSWORD_ATTRIBUTE}
                placeholder="userPassword"
                helpText={LdapStrings.FORM.IF_FIELD_LEFT_BLANK_USER}
                disabled={!canEditFields}
              />
            </Flex>
          </SettingsFormSection>

          <SettingsFormSection title={LdapStrings.FORM.GROUP_MAPPING} defaultOpen>
            <SettingsCheckbox
              name="ldapGroupsAsRoles"
              label={LdapStrings.FORM.MAP_LDAP_GROUPS_ROLES}
              checked={formData.ldapGroupsAsRoles}
              onChange={(checked) => handleChange('ldapGroupsAsRoles', checked)}
              description={LdapStrings.FORM.ENABLE_MAP_LDAP_GROUPS_ASTRUM}
              disabled={!canEditFields}
            />

            {formData.ldapGroupsAsRoles && (
              <>
                <SettingsSelect
                  name="groupType"
                  label={LdapStrings.FORM.GROUP_TYPE}
                  value={formData.groupType || 'dynamic'}
                  onChange={(value) => handleChange('groupType', value as 'static' | 'dynamic')}
                  required
                  disabled={!canEditFields}
                  options={[
                    { value: 'static', label: LdapStrings.FORM.STATIC_GROUPS },
                    { value: 'dynamic', label: LdapStrings.FORM.DYNAMIC_GROUPS },
                  ]}
                />

                {formData.groupType === 'static' && (
                  <>
                    <SettingsTextInput
                      {...form.field('groupBaseDn')}
                      label={LdapStrings.FORM.GROUP_RELATIVE_DN}
                      placeholder="ou=groups"
                      helpText={LdapStrings.FORM.RELATIVE_DN_WHERE_GROUP_OBJECTS}
                      disabled={!canEditFields}
                    />

                    <SettingsCheckbox
                      name="groupSubtree"
                      label={LdapStrings.FORM.SEARCH_GROUP_SUBTREE}
                      checked={formData.groupSubtree}
                      onChange={(checked) => handleChange('groupSubtree', checked)}
                      description={LdapStrings.FORM.SEARCH_ENTIRE_SUBTREE_GROUPS}
                      disabled={!canEditFields}
                    />

                    <Flex gap="3" className="ldap-form__field-row">
                      <SettingsTextInput
                        {...form.field('groupObjectClass')}
                        label={LdapStrings.FORM.GROUP_OBJECT_CLASS}
                        placeholder="groupOfUniqueNames"
                        required
                        disabled={!canEditFields}
                      />

                      <SettingsTextInput
                        {...form.field('groupIdAttribute')}
                        label={LdapStrings.FORM.GROUP_ID_ATTRIBUTE}
                        placeholder="cn"
                        required
                        disabled={!canEditFields}
                      />
                    </Flex>

                    <SettingsTextInput
                      {...form.field('groupMemberAttribute')}
                      label={LdapStrings.FORM.GROUP_MEMBER_ATTRIBUTE}
                      placeholder="uniqueMember"
                      required
                      disabled={!canEditFields}
                    />

                    <SettingsTextInput
                      {...form.field('groupMemberFormat')}
                      label={LdapStrings.FORM.GROUP_MEMBER_FORMAT}
                      placeholder="uid=${username},ou=people,dc=example,dc=com"
                      helpText={LdapStrings.FORM.USE_PLACEHOLDER}
                      required
                      disabled={!canEditFields}
                    />
                  </>
                )}

                {formData.groupType === 'dynamic' && (
                  <SettingsTextInput
                    {...form.field('userMemberOfAttribute')}
                    label={LdapStrings.FORM.GROUP_MEMBER_OF_ATTRIBUTE}
                    placeholder="memberOf"
                    helpText={LdapStrings.FORM.SET_ATTRIBUTE_USED_STORE_ATTRIBUTE}
                    required
                    disabled={!canEditFields}
                  />
                )}
              </>
            )}
          </SettingsFormSection>

          {/* User Mapping Verification */}
          <Box className="ldap-form__verification">
            <Flex align="center" gap="3" wrap="wrap">
              <SettingsButton
                variant="secondary"
                onClick={handleVerifyUserMapping}
                disabled={userMappingVerifying || !userGroupValid || !canUpdate}
                loading={userMappingVerifying}
                data-testid="ldap-verify-user-mapping"
                data-analytics-id="as-ldap-form-verify-user-mapping"
                icon={Play}
              >
                {LdapStrings.FORM.VERIFY_USER_MAPPING}
              </SettingsButton>
              <SettingsButton
                variant="secondary"
                onClick={handleOpenLoginModal}
                disabled={!userGroupValid || !canUpdate}
                data-testid="ldap-verify-login-button"
                data-analytics-id="as-ldap-form-verify-login"
                icon={KeyRound}
              >
                {LdapStrings.FORM.VERIFY_LOGIN}
              </SettingsButton>
              {showUserResults && (
                <Badge
                  variant="soft"
                  color={verifiedUsers.length > 0 ? 'green' : 'red'}
                  size="2"
                  data-testid="ldap-user-mapping-badge"
                >
                  {verifiedUsers.length > 0 ? `${verifiedUsers.length} user(s)` : LdapStrings.FORM.NO_MATCHES_FOUND}
                </Badge>
              )}
            </Flex>

            {verificationError && currentStep === 1 && (
              <Box className="ldap-form__verification-error">
                <SettingsAlert type="error">{verificationError}</SettingsAlert>
              </Box>
            )}

            {showUserResults && (
              <Box ref={userResultsRef} className="ldap-form__user-results">
                <Text size="2" weight="medium" className="ldap-form__user-results-title">
                  {LdapStrings.FORM.FOUND_USER_S(verifiedUsers.length)}
                </Text>
                <EntityTable<LdapUser>
                  data={verifiedUsers}
                  columns={USER_MAPPING_COLUMNS}
                  getRowKey={(u) => u.username}
                  clickable={false}
                  showRowArrow={false}
                  ariaLabel={LdapStrings.FORM.VERIFIED_LDAP_USERS_ARIA}
                />
              </Box>
            )}
          </Box>

        </>
        )}
      </WizardForm>

      {/* Verify Login Modal */}
      <Dialog.Root open={showLoginModal} onOpenChange={setShowLoginModal}>
        <Dialog.Content maxWidth="450px" data-testid="ldap-login-modal">
          <Dialog.Title>
            <Flex align="center" gap="2">
              <KeyRound size={20} aria-hidden="true" />
              {LdapStrings.FORM.TEST_LDAP_LOGIN}
            </Flex>
          </Dialog.Title>
          <Dialog.Description size="2">
            {LdapStrings.FORM.ENTER_CREDENTIALS_VERIFY_USER_CAN}
          </Dialog.Description>

          <Flex direction="column" gap="3" mt="4">
            <Box>
              <Text as="label" htmlFor="ldap-login-username" size="2" weight="medium" mb="1" className="ldap-form__modal-label">
                <User size={14} aria-hidden="true" />
                {LdapStrings.FORM.USERNAME}
              </Text>
              <TextField.Root
                id="ldap-login-username"
                value={loginUsername}
                onChange={(e) => setLoginUsername(e.target.value)}
                placeholder={LdapStrings.FORM.ENTER_LDAP_USERNAME_PLACEHOLDER}
                data-testid="ldap-login-username"
                autoComplete="off"
              />
            </Box>

            <Box>
              <Text as="label" htmlFor="ldap-login-password" size="2" weight="medium" mb="1" className="ldap-form__modal-label">
                <Lock size={14} aria-hidden="true" />
                {LdapStrings.FORM.PASSWORD}
              </Text>
              <TextField.Root
                id="ldap-login-password"
                type="password"
                value={loginPassword}
                onChange={(e) => setLoginPassword(e.target.value)}
                placeholder={LdapStrings.FORM.ENTER_PASSWORD_PLACEHOLDER}
                data-testid="ldap-login-password"
                autoComplete="new-password"
              />
            </Box>

            {/* A11y (ASTRUM-53625 A4): mount the role="alert" node *fresh* on
                each result (conditional render, keyed by the announcement
                counter) so VoiceOver reliably announces it. A persistent alert
                whose text is swapped in place is frequently missed by
                VoiceOver. */}
            {loginResult && (
              <Box
                key={loginAnnounceKey}
                className={`ldap-form__modal-result ldap-form__modal-result--${loginResult.success ? 'success' : 'error'}`}
                role="alert"
                aria-live="assertive"
                aria-atomic="true"
                data-testid="ldap-login-result"
              >
                {loginResult.success ? (
                  <CheckCircle2 size={16} aria-hidden="true" />
                ) : (
                  <AlertCircle size={16} aria-hidden="true" />
                )}
                <Text size="2">{loginResult.message}</Text>
              </Box>
            )}
          </Flex>

          <Flex gap="3" mt="4" justify="end">
            <Dialog.Close>
              <SettingsButton variant="secondary">
                {CommonStrings.ACTIONS.CLOSE}
              </SettingsButton>
            </Dialog.Close>
            <SettingsButton
              ref={loginSubmitRef}
              variant="primary"
              onClick={handleVerifyLogin}
              disabled={!(loginUsername && loginPassword ) || loginVerifying || !canUpdate}
              loading={loginVerifying}
              data-testid="ldap-login-submit"
            >
              {LdapStrings.FORM.TEST_LOGIN}
            </SettingsButton>
          </Flex>
        </Dialog.Content>
      </Dialog.Root>

      {/* Password Modal — used for both the save re-entry flow (ASTRUM-23184) and
          the verify-before-action flow (ASTRUM-54062). pendingVerifyAction
          distinguishes the two paths in handlePasswordReentry. */}
      <Dialog.Root
        open={showPasswordModal}
        onOpenChange={(open) => {
          setShowPasswordModal(open);
          if (!open) {
            setPasswordVerifying(false);
            setReenteredPassword('');
            setPendingVerifyAction(null);
          }
        }}
      >
        <Dialog.Content maxWidth="450px" data-testid="ldap-password-modal">
          <Dialog.Title>
            <Flex align="center" gap="2">
              <Lock size={20} aria-hidden="true" />
              {LdapStrings.FORM.ENTER_LDAP_PASSWORD}
            </Flex>
          </Dialog.Title>
          <Dialog.Description size="2">
            {pendingVerifyAction === 'userMapping'
              ? LdapStrings.FORM.PASSWORD_REQUIRED_VERIFY_LDAP_USER
              : pendingVerifyAction === 'connection'
              ? LdapStrings.FORM.PASSWORD_REQUIRED_VERIFY_LDAP_SERVER
              : LdapStrings.FORM.SECURITY_REASONS_PASSWORD_MUST_RE}
          </Dialog.Description>

          <Flex direction="column" gap="3" mt="4">
            <Box>
              <Text as="label" htmlFor="ldap-password-input" size="2" weight="medium" mb="1" className="ldap-form__modal-label">
                <Lock size={14} aria-hidden="true" />
                {LdapStrings.FORM.PASSWORD}
              </Text>
              <TextField.Root
                id="ldap-password-input"
                type="password"
                value={reenteredPassword}
                onChange={(e) => setReenteredPassword(e.target.value)}
                placeholder={LdapStrings.FORM.ENTER_LDAP_PASSWORD_PLACEHOLDER}
                data-testid="ldap-password-input"
                autoComplete="new-password"
              />
              {passwordError && (
                <Text size="1" color="red" mt="1">{passwordError}</Text>
              )}
            </Box>
          </Flex>

          <Flex gap="3" mt="4" justify="end">
            <Dialog.Close>
              <SettingsButton variant="secondary">
                {CommonStrings.ACTIONS.CANCEL}
              </SettingsButton>
            </Dialog.Close>
            <SettingsButton
              variant="primary"
              onClick={handlePasswordReentry}
              disabled={!reenteredPassword || form.isSaving || passwordVerifying}
              loading={form.isSaving || passwordVerifying}
              data-testid="ldap-password-submit"
              data-analytics-id="as-ldap-form-password-submit"
            >
              {pendingVerifyAction ? LdapStrings.FORM.VERIFY : CommonStrings.ACTIONS.SAVE}
            </SettingsButton>
          </Flex>
        </Dialog.Content>
      </Dialog.Root>

      {/* Delete Confirmation Modal */}
      <ConfirmDialog
        open={showDeleteConfirm}
        testId="delete-ldap-server-dialog"
        onOpenChange={setShowDeleteConfirm}
        title={LdapStrings.FORM.DELETE_LDAP_SERVER}
        message={LdapStrings.FORM.DELETE_SERVER_CONFIRM(formData.name)}
        confirmLabel={LdapStrings.FORM.DELETE_SERVER}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="danger"
        onConfirm={handleDeleteConfirm}
        analyticsId="as-ldap-form-delete-confirm"
      />
    </Box>
  );
}

export default LdapForm;
