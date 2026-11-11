import { Button, Flex, Spinner } from '@radix-ui/themes';
import { useState } from 'react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  ConfirmDialog,
  DateTime,
  describeError,
  ErrorState,
  FormError,
  type Option,
  SelectField,
  useToast,
} from '../../../ui';
import { USERS } from '../api';
import { ReadOnlyValue } from '../shared/Edit';
import { explainRefusal } from '../shared/messages';
import {
  type RequirementBody,
  type SecondFactorRequirement,
  type UserSecondFactorStatus,
  userRequirementPath,
  userTwoFactorPath,
} from '../two-factor/api';
import { isRequirement } from '../two-factor/model';
import styles from './Users.module.css';

const ADMINISTRATOR = 'astrum:*';
const USERS_UPDATE = 'astrum:users:update';

export interface SecondFactorPanelProps {
  userId: string;
  isMe: boolean;
}

export function SecondFactorPanel({ userId, isMe }: SecondFactorPanelProps) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canUpdate = useCan(USERS_UPDATE);
  const canAdminister = useCan(ADMINISTRATOR);
  const status = useApi<UserSecondFactorStatus>(userTwoFactorPath(userId));
  const [failure, setFailure] = useState<string | undefined>();
  const [resetting, setResetting] = useState(false);

  const change = async (value: string) => {
    if (!isRequirement(value)) {
      return;
    }
    const body: RequirementBody = { requirement: value };
    setFailure(undefined);
    try {
      await explainRefusal(t, () => api.put(userRequirementPath(userId), body));
      toast.success(t('security.twoFactor.requirementSaved', { userId }));
      await invalidate(USERS);
    } catch (error) {
      setFailure(describeError(t, error));
    }
  };

  const reset = () =>
    explainRefusal(t, async () => {
      await api.delete(userTwoFactorPath(userId));
      toast.success(t('security.twoFactor.resetDone', { userId }));
      await invalidate(USERS);
    });

  if (status.isPending) {
    return (
      <Flex justify="center" py="4">
        <Spinner />
      </Flex>
    );
  }
  if (status.error) {
    return <ErrorState error={status.error} onRetry={() => void status.refetch()} />;
  }
  const current = status.data;
  const labels: Record<SecondFactorRequirement, string> = {
    policy: t('security.twoFactor.requirementPolicy'),
    required: t('security.twoFactor.requirementRequired'),
    exempt: t('security.twoFactor.requirementExempt'),
  };
  const requirementOptions: Option[] = [
    { value: 'policy', label: labels.policy },
    { value: 'required', label: labels.required },
    ...(canAdminister ? [{ value: 'exempt', label: labels.exempt }] : []),
  ];
  const sourceText: string =
    current.requirementSource === 'policy'
      ? t('security.twoFactor.sourcePolicy')
      : t('security.twoFactor.sourceUser');
  return (
    <div className={styles.factor}>
      <ReadOnlyValue
        label={t('security.twoFactor.status')}
        value={current.enrolled ? t('security.twoFactor.enrolledYes') : t('security.twoFactor.enrolledNo')}
      />
      <ReadOnlyValue
        label={t('security.twoFactor.recoveryCodes')}
        value={t('security.twoFactor.codesLeft', { count: current.recoveryCodesLeft })}
      />
      {current.lockedUntil && (
        <ReadOnlyValue
          label={t('security.twoFactor.lockedUntil')}
          value={<DateTime value={current.lockedUntil} />}
        />
      )}
      <ReadOnlyValue
        label={t('security.twoFactor.effective')}
        value={current.required ? t('security.twoFactor.requiredNow') : t('security.twoFactor.notRequired')}
      />
      <ReadOnlyValue label={t('security.twoFactor.source')} value={sourceText} />
      {current.enrollBy && (
        <ReadOnlyValue
          label={t('security.twoFactor.enrollBy')}
          value={<DateTime value={current.enrollBy} />}
        />
      )}
      {canUpdate ? (
        <SelectField
          label={t('security.twoFactor.requirement')}
          value={current.requirement}
          options={requirementOptions}
          onChange={(value) => void change(value)}
        />
      ) : (
        <ReadOnlyValue label={t('security.twoFactor.requirement')} value={labels[current.requirement]} />
      )}
      <FormError message={failure} />
      {canUpdate && !isMe && (
        <>
          <Button
            type="button"
            color="red"
            variant="soft"
            className={styles.factorReset}
            onClick={() => setResetting(true)}
          >
            {t('security.twoFactor.reset')}
          </Button>
          <ConfirmDialog
            open={resetting}
            onOpenChange={setResetting}
            onConfirm={reset}
            title={t('security.twoFactor.resetTitle', { userId })}
            description={t('security.twoFactor.resetText')}
            confirmLabel={t('security.twoFactor.reset')}
            danger
          />
        </>
      )}
    </div>
  );
}
