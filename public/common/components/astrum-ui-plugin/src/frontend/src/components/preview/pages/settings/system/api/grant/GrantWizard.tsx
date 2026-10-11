

import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Badge, Box, Button, Checkbox, Flex, Heading, Spinner, Table, Text, TextField } from '@radix-ui/themes';

import { getSourceLabel } from '../../../security/users/types';
import { useToast } from '../../../../../shared';
import { WizardForm } from '../../../../../shared/form/WizardForm';

import { NEW_ROLE_ID_PATTERN } from './grantWizardMachine';
import { useGrantWizard, userDirectoryKey } from './useGrantWizard';
import type { MergedApiEndpoint } from '../utils/mergeSwaggerPermissions';
import ApiStrings from '../../../../../constants/pages/settings/system/ApiStrings';
import CommonStrings from '../../../../../constants/CommonStrings';
import plural from '../../../../../../../interface/plural';

export interface GrantWizardProps {
  row: MergedApiEndpoint;
  active: boolean;
}

const WIZARD_STEPS = [
  { id: 'permissions', label: ApiStrings.GRANT_WIZARD.PERMISSIONS },
  { id: 'role', label: ApiStrings.GRANT_WIZARD.CHOOSE_ROLE },
  { id: 'users', label: ApiStrings.GRANT_WIZARD.SELECT_USERS },
  { id: 'confirm', label: CommonStrings.ACTIONS.CONFIRM },
];

function stateToStep(state: { matches: (s: string) => boolean }): number {
  if (state.matches('step1')) return 0;
  if (state.matches('step2')) return 1;
  if (state.matches('step3')) return 2;
  if (state.matches('step4') || state.matches('applying')) return 3;
  return 0;
}

export function GrantWizard({ row, active }: GrantWizardProps) {
  const {
    state,
    send,
    session,
    loadingDir,
    dirError,
    recommendedRoles,
    userCountByRoleId,
    noMappedPermissions,
    suggestedRoleId,
    suggestedRoleName,
  } = useGrantWizard(row, active);

  const toast = useToast();
  const prevDoneRef = useRef(false);

  useEffect(() => {
    const isDone = state.matches('done');
    if (isDone && !prevDoneRef.current) {
      const ctx = state.context;
      const results = ctx.applyResults;
      const successCount = results.filter((r) => r.ok).length;
      const failCount = results.length - successCount;
      const successNames = results.filter((r) => r.ok).map((r) => r.userKey.split('\0')[0]);
      const roleName = ctx.mode === 'create'
        ? (ctx.newRoleName.trim() || ctx.newRoleId.trim())
        : (session?.roles.find((r) => r.id === ctx.existingRoleId)?.name || ctx.existingRoleId || 'role');
      const userList = successNames.length <= 3
        ? successNames.join(', ')
        : ApiStrings.GRANT_WIZARD.AND_MORE(successNames.slice(0, 3).join(', '), successNames.length - 3);
      if (failCount === 0) {
        toast.success(ApiStrings.GRANT_WIZARD.ROLE_GRANTED(roleName, userList));
      } else if (successCount > 0) {
        toast.success(ApiStrings.GRANT_WIZARD.ROLE_GRANTED_FAILED(roleName, userList, failCount));
      } else {
        toast.error(ApiStrings.GRANT_WIZARD.FAILED_GRANT_ROLE_SEE_DETAILS(roleName));
      }
    }
    prevDoneRef.current = isDone;
  }, [state, toast, session]);

  const [userFilter, setUserFilter] = useState('');

  const permissionReqs = row.permission?.permissions ?? [];
  const logical = permissionReqs[0]?.logical === 'OR' ? 'OR' : 'AND';

  const filteredUsers = useMemo(() => {
    if (!session) {
      return [];
    }
    const q = userFilter.trim().toLowerCase();
    if (!q) {
      return session.users;
    }
    return session.users.filter((u) => {
      const blob = `${u.userId} ${u.firstName} ${u.lastName} ${u.emailAddress}`.toLowerCase();
      return blob.includes(q);
    });
  }, [session, userFilter]);

  const wizardStep = stateToStep(state);
  const ctx = state.context;

  const handleStepChange = useCallback(
    (newStep: number) => {
      if (newStep > wizardStep) {
        send({ type: 'NEXT' });
      } else if (newStep < wizardStep) {
        send({ type: 'BACK' });
      }
    },
    [wizardStep, send]
  );

  const handleComplete = useCallback(() => {
    send({ type: 'APPLY' });
  }, [send]);

  const handleCancel = useCallback(() => {
    send({ type: 'RESET' });
  }, [send]);

  const canAdvance = useMemo(() => {
    if (wizardStep === 0) return true;
    if (wizardStep === 1) {
      if (ctx.mode === 'existing') return Boolean(ctx.existingRoleId);
      const id = ctx.newRoleId.trim();
      return NEW_ROLE_ID_PATTERN.test(id) && ctx.newRoleName.trim().length > 0;
    }
    if (wizardStep === 2) return ctx.selectedUserKeys.length > 0;
    return true;
  }, [wizardStep, ctx]);

  if (!active) {
    return null;
  }

  if (noMappedPermissions || permissionReqs.length === 0) {
    return (
      <Box data-testid="api-grant-wizard" className="api-grant-wizard">
        <Text size="2" color="gray">
          {ApiStrings.GRANT_WIZARD.GRANT_ACCESS_NEEDS_MAPPED_PERMISSION}
        </Text>
      </Box>
    );
  }

  if (loadingDir && !session) {
    return (
      <Flex align="center" gap="2" py="4" data-testid="api-grant-wizard">
        <Spinner />
        <Text size="2">{ApiStrings.GRANT_WIZARD.LOADING_ROLES_USERS}</Text>
      </Flex>
    );
  }

  if (dirError) {
    return (
      <Text size="2" color="red" data-testid="api-grant-wizard">
        {dirError}
      </Text>
    );
  }

  if (!session) {
    return null;
  }

  const anonymous = row.permission && !row.permission.authenticated;

  const summaryRoleLabel =
    ctx.mode === 'create'
      ? `${ctx.newRoleName.trim() || ctx.newRoleId.trim() || ApiStrings.GRANT_WIZARD.NEW_ROLE}`
      : session.roles.find((r) => r.id === ctx.existingRoleId)?.name || ctx.existingRoleId || '';

  if (state.matches('done')) {
    return (
      <Box className="api-grant-wizard" data-testid="api-grant-wizard">
        <Heading as="h3" size="3" weight="medium" mb="2">
          {ApiStrings.GRANT_WIZARD.RESULT}
        </Heading>
        <Table.Root variant="surface" mb="3">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeaderCell>{ApiStrings.GRANT_WIZARD.USER}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{CommonStrings.LABELS.STATUS}</Table.ColumnHeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {ctx.applyResults.map((r) => {
              const [uid] = r.userKey.split('\0');
              return (
                <Table.Row key={r.userKey}>
                  <Table.Cell>{uid}</Table.Cell>
                  <Table.Cell>
                    {r.ok ? (
                      <Text size="2" color="green">
                        {ApiStrings.GRANT_WIZARD.ASSIGNED}
                      </Text>
                    ) : (
                      <Text size="2" color="red">
                        {r.message || ApiStrings.GRANT_WIZARD.FAILED}
                      </Text>
                    )}
                  </Table.Cell>
                </Table.Row>
              );
            })}
          </Table.Body>
        </Table.Root>
        <Flex gap="2" wrap="wrap">
          <Button variant="soft" onClick={() => send({ type: 'DONE_ANOTHER' })}>
            {ApiStrings.GRANT_WIZARD.GRANT_MORE_USERS}
          </Button>
          <Button onClick={() => send({ type: 'RESET' })}>{ApiStrings.GRANT_WIZARD.START_OVER}</Button>
        </Flex>
      </Box>
    );
  }

  return (
    <Box className="api-grant-wizard" data-testid="api-grant-wizard">
      {anonymous && (
        <Box mb="3" p="2" className="api-who-has-access-tab__banner">
          <Text size="2">
            {ApiStrings.GRANT_WIZARD.ENDPOINT_MAY_ALLOW_ANONYMOUS_ACCESS}
          </Text>
        </Box>
      )}

      <WizardForm
        steps={WIZARD_STEPS}
        currentStep={wizardStep}
        onStepChange={handleStepChange}
        onComplete={handleComplete}
        onCancel={handleCancel}
        completeLabel={ApiStrings.GRANT_WIZARD.APPLY}
        canAdvance={canAdvance}
        loading={state.matches('applying')}
        error={ctx.applyError || undefined}
        dirty={false}
        noDirtyTracking
        testId="api-grant-wizard"
        className="api-grant-wizard__form"
      >
        {wizardStep === 0 && (
          <Box>
            <Text size="2" color="gray" mb="2">
              {logical === 'OR' ? ApiStrings.GRANT_WIZARD.REQUIRES_ANY : ApiStrings.GRANT_WIZARD.REQUIRES_ALL}
            </Text>
            <Box mb="3">
              {permissionReqs.map((p) => (
                <Text key={p.permission} size="2" as="div">
                  <Text weight="bold" as="span">
                    {p.permission}
                  </Text>
                </Text>
              ))}
            </Box>
          </Box>
        )}

        {wizardStep === 1 && (
          <Box>
            <Text size="2" color="gray" mb="3">
              {plural(ApiStrings.GRANT_WIZARD.PICK_EXISTING_ROLE, permissionReqs.length)}
            </Text>

            {recommendedRoles.length > 0 && (
              <Box mb="3">
                <Text size="2" weight="medium" mb="2">
                  {ApiStrings.GRANT_WIZARD.RECOMMENDED}
                </Text>
                <Flex direction="column" gap="2">
                  {recommendedRoles.map((r, idx) => (
                    <Flex key={r.id} align="center" gap="2" wrap="wrap">
                      <Button
                        type="button"
                        variant={ctx.existingRoleId === r.id && ctx.mode === 'existing' ? 'solid' : 'outline'}
                        onClick={() => send({ type: 'SELECT_EXISTING', roleId: r.id })}
                      >
                        {r.name}
                      </Button>
                      {idx === 0 && (
                        <Badge color="green" size="1">
                          {ApiStrings.GRANT_WIZARD.MOST_SPECIFIC}
                        </Badge>
                      )}
                      <Text size="1" color="gray">
                        {plural(ApiStrings.GRANT_WIZARD.ROLE_USER_COUNT, userCountByRoleId.get(r.id) ?? 0)} · {r.id}
                      </Text>
                    </Flex>
                  ))}
                </Flex>
              </Box>
            )}

            <Box mb="3">
              <Button
                type="button"
                variant={ctx.mode === 'create' ? 'solid' : 'outline'}
                onClick={() => {
                  send({ type: 'SELECT_CREATE' });
                  if (!ctx.newRoleId.trim()) {
                    send({ type: 'SET_NEW_ROLE_FIELD', field: 'newRoleId', value: suggestedRoleId });
                  }
                  if (!ctx.newRoleName.trim()) {
                    send({ type: 'SET_NEW_ROLE_FIELD', field: 'newRoleName', value: suggestedRoleName });
                  }
                }}
              >
                {ApiStrings.GRANT_WIZARD.CREATE_NEW_ROLE}
              </Button>
            </Box>

            {ctx.mode === 'create' && (
              <Box mb="3" style={{ maxWidth: 480 }}>
                <Text size="2" mb="2" as="div">
                  {ApiStrings.GRANT_WIZARD.ROLE_ID}
                </Text>
                <TextField.Root
                  mb="2"
                  value={ctx.newRoleId}
                  onChange={(e) =>
                    send({ type: 'SET_NEW_ROLE_FIELD', field: 'newRoleId', value: e.target.value })
                  }
                  aria-label={ApiStrings.GRANT_WIZARD.NEW_ROLE_ID_ARIA}
                />
                {!NEW_ROLE_ID_PATTERN.test(ctx.newRoleId.trim()) && ctx.newRoleId.trim() !== '' && (
                  <Text size="1" color="red" mb="2">
                    {ApiStrings.GRANT_WIZARD.USE_LETTERS_DIGITS_PERIODS_HYPHENS}
                  </Text>
                )}
                <Text size="2" mb="2" as="div">
                  {ApiStrings.GRANT_WIZARD.DISPLAY_NAME}
                </Text>
                <TextField.Root
                  mb="2"
                  value={ctx.newRoleName}
                  onChange={(e) =>
                    send({ type: 'SET_NEW_ROLE_FIELD', field: 'newRoleName', value: e.target.value })
                  }
                  aria-label={ApiStrings.GRANT_WIZARD.NEW_ROLE_NAME_ARIA}
                />
                <Text size="2" mb="2" as="div">
                  {ApiStrings.GRANT_WIZARD.DESCRIPTION_OPTIONAL}
                </Text>
                <TextField.Root
                  value={ctx.newRoleDescription}
                  onChange={(e) =>
                    send({ type: 'SET_NEW_ROLE_FIELD', field: 'newRoleDescription', value: e.target.value })
                  }
                  aria-label={ApiStrings.GRANT_WIZARD.NEW_ROLE_DESCRIPTION_ARIA}
                />
              </Box>
            )}
          </Box>
        )}

        {wizardStep === 2 && (
          <Box>
            <Text size="2" color="gray" mb="2">
              {ApiStrings.GRANT_WIZARD.CHOOSE_ONE_MORE_USERS_RECEIVE}
            </Text>
            <Box mb="2" style={{ maxWidth: 360 }}>
              <TextField.Root
                placeholder={ApiStrings.GRANT_WIZARD.FILTER_USERS_PLACEHOLDER}
                value={userFilter}
                onChange={(e) => setUserFilter(e.target.value)}
                aria-label={ApiStrings.GRANT_WIZARD.FILTER_USERS_GRANT_ARIA}
              />
            </Box>
            <Table.Root variant="surface" mb="3">
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell style={{ width: 48 }}> </Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{ApiStrings.GRANT_WIZARD.USER}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{ApiStrings.GRANT_WIZARD.SOURCE}</Table.ColumnHeaderCell>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {filteredUsers.map((u) => {
                  const key = userDirectoryKey(u);
                  const checked = ctx.selectedUserKeys.includes(key);
                  return (
                    <Table.Row key={key}>
                      <Table.Cell>
                        <Checkbox
                          checked={checked}
                          onCheckedChange={() => send({ type: 'TOGGLE_USER', userKey: key })}
                          aria-label={ApiStrings.GRANT_WIZARD.SELECT_USER(u.userId)}
                        />
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="2">{u.userId}</Text>
                        <Text size="1" color="gray" as="div">
                          {u.firstName} {u.lastName}
                        </Text>
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="2">{getSourceLabel(u.source)}</Text>
                      </Table.Cell>
                    </Table.Row>
                  );
                })}
              </Table.Body>
            </Table.Root>
          </Box>
        )}

        {wizardStep === 3 && (
          <Box>
            <Text size="2" mb="3">
              {ctx.mode === 'create' ? (
                <>
                  {ApiStrings.GRANT_WIZARD.CREATE_ROLE_SUMMARY(
                    <Text weight="bold">{ctx.newRoleId.trim()}</Text>,
                    summaryRoleLabel,
                    ctx.selectedUserKeys.length
                  )}
                </>
              ) : (
                <>
                  {ApiStrings.GRANT_WIZARD.ASSIGN_ROLE_SUMMARY(<Text weight="bold">{summaryRoleLabel}</Text>, ctx.selectedUserKeys.length)}
                </>
              )}
            </Text>
          </Box>
        )}
      </WizardForm>
    </Box>
  );
}
