import { Button, Flex, Text } from '@radix-ui/themes';
import { Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { DateTime, describeError, ErrorState, Page, StatusBadge, useConfirm, useToast } from '../../../ui';
import { POLICY_EXCEPTIONS } from '../api';
import { type ListColumn, ListTable, type RowButton } from '../shared/AdminList';
import list from '../shared/AdminList.module.css';
import { ExceptionDialog } from './ExceptionDialog';
import { expired, type PolicyException, RULE_OPTIONS, type Rule, subjectText } from './exceptions';
import { EXCEPTION_CREATE, EXCEPTION_DELETE, EXCEPTION_READ } from './permissions';

export default function ExceptionsPage() {
  const t = useT();
  const toast = useToast();
  const confirm = useConfirm();
  const invalidate = useInvalidateApi();
  const canRead = useCan(EXCEPTION_READ);
  const canCreate = useCan(EXCEPTION_CREATE);
  const canDelete = useCan(EXCEPTION_DELETE);
  const exceptions = useApi<PolicyException[]>(canRead ? POLICY_EXCEPTIONS : null);
  const [creating, setCreating] = useState(false);
  const now = Date.now();
  const ruleText = (rule: Rule) => t(RULE_OPTIONS.find((option) => option.value === rule)?.label ?? rule);
  const columns: ListColumn<PolicyException>[] = [
    {
      id: 'policy',
      header: t('security.exceptions.columns.policy'),
      cell: (exception) => exception.policy ?? t('security.exceptions.allPolicies'),
    },
    {
      id: 'rule',
      header: t('security.exceptions.columns.rule'),
      cell: (exception) => ruleText(exception.rule),
    },
    {
      id: 'subject',
      header: t('security.exceptions.columns.subject'),
      cell: (exception) => subjectText(exception),
    },
    {
      id: 'reason',
      header: t('security.exceptions.columns.reason'),
      cell: (exception) => exception.reason,
    },
    {
      id: 'expires',
      header: t('security.exceptions.columns.expires'),
      cell: (exception) =>
        exception.expiresAt === null ? (
          <Text as="span" color="gray">
            {t('security.exceptions.never')}
          </Text>
        ) : (
          <Flex gap="2" align="center" wrap="wrap">
            <Text as="span" color={expired(exception, now) ? 'gray' : undefined}>
              <DateTime value={exception.expiresAt} />
            </Text>
            {expired(exception, now) && (
              <StatusBadge tone="danger">{t('security.exceptions.expired')}</StatusBadge>
            )}
          </Flex>
        ),
    },
    {
      id: 'createdBy',
      header: t('security.exceptions.columns.createdBy'),
      cell: (exception) => exception.createdBy,
    },
  ];
  const revoke = async (exception: PolicyException) => {
    const agreed = await confirm({
      title: t('security.exceptions.revokeTitle'),
      description: t('security.exceptions.revokeText'),
      confirmLabel: t('security.exceptions.revoke'),
      danger: true,
    });
    if (!agreed) {
      return;
    }
    try {
      await api.delete(`${POLICY_EXCEPTIONS}${path`/${exception.id}`}`);
      toast.success(t('security.exceptions.revoked'));
      await invalidate(POLICY_EXCEPTIONS);
    } catch (error) {
      toast.error(t('security.exceptions.revokeFailed'), describeError(t, error));
    }
  };
  const rowButtons = (exception: PolicyException): RowButton[] =>
    canDelete
      ? [
          {
            label: t('security.exceptions.revoke'),
            icon: Trash2,
            onClick: () => void revoke(exception),
            danger: true,
          },
        ]
      : [];
  return (
    <Page
      title={t('security.exceptions.title')}
      description={t('security.exceptions.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.exceptions.title') },
      ]}
      actions={
        canCreate && (
          <Button size="3" className={list.bigButton} onClick={() => setCreating(true)}>
            <Plus size={16} aria-hidden="true" />
            {t('security.exceptions.add')}
          </Button>
        )
      }
    >
      {exceptions.error ? (
        <ErrorState error={exceptions.error} onRetry={() => void exceptions.refetch()} />
      ) : (
        <ListTable
          plain
          label={t('security.exceptions.title')}
          columns={columns}
          rows={exceptions.data ?? []}
          loading={exceptions.isPending}
          getRowId={(exception) => exception.id}
          buttons={rowButtons}
          empty={<div className={list.count}>{t('security.exceptions.empty')}</div>}
        />
      )}
      <ExceptionDialog open={creating} onOpenChange={setCreating} />
    </Page>
  );
}
