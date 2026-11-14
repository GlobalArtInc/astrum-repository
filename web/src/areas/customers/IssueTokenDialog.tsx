import { Button, Callout, Code, Dialog, Flex, TextField as RadixTextField, Text } from '@radix-ui/themes';
import { useId, useState } from 'react';
import { ApiError, api, useApi, useInvalidateApi, withQuery } from '../../api';
import { useCan, useSession } from '../../auth/state';
import { useT } from '../../i18n';
import {
  CodeBlock,
  CopyButton,
  FieldShell,
  FormError,
  MultiSelectField,
  NumberField,
  type Option,
  SelectField,
  TextField,
  useForm,
} from '../../ui';
import { DEFAULT_SOURCE, USERS, type User } from '../security/api';
import { CUSTOMERS, type IssuedToken, REPOSITORIES, type Repository, tokensUrl } from './api';
import styles from './Customers.module.css';
import {
  CUSTOM_EXPIRY,
  EMPTY_ISSUE_TOKEN,
  EXPIRY_PRESETS,
  hasGroup,
  type IssueTokenValues,
  MAX_TOKEN_DAYS,
  NAME_MAX,
  type ReadyLine,
  readyLines,
  repositoryOptions,
  serverErrors,
  toIssueTokenRequest,
  USERS_READ,
  usableOwners,
  validateIssueToken,
} from './model';

export interface IssueTokenDialogProps {
  customerId: string;
  open: boolean;
  onOpenChange(open: boolean): void;
}

interface IssuedPanel {
  token: string;
  lines: ReadyLine[];
}

export function IssueTokenDialog({ customerId, open, onOpenChange }: IssueTokenDialogProps) {
  const t = useT();
  const id = useId();
  const invalidate = useInvalidateApi();
  const { state } = useSession();
  const canReadUsers = useCan(USERS_READ);
  const repositories = useApi<Repository[]>(REPOSITORIES, { enabled: open });
  const users = useApi<User[]>(withQuery(USERS, { source: DEFAULT_SOURCE }), {
    enabled: open && canReadUsers,
  });
  const [issued, setIssued] = useState<IssuedPanel | null>(null);
  const known = repositories.data ?? [];
  const formatOf = (name: string) => known.find((repository) => repository.name === name)?.format ?? '';
  const form = useForm<IssueTokenValues>({
    initial: EMPTY_ISSUE_TOKEN,
    validate: (values) => validateIssueToken(t, values),
    onSubmit: async (values) => {
      try {
        const created = await api.post<IssuedToken>(tokensUrl(customerId), toIssueTokenRequest(values));
        const targets = created.repositories.map((name) => ({ name, format: formatOf(name) }));
        setIssued({
          token: created.token,
          lines: readyLines(state.baseUrl, customerId, created.token, targets),
        });
      } catch (failure) {
        if (failure instanceof ApiError && failure.status === 400 && failure.fieldErrors.length > 0) {
          const mapped = serverErrors(failure);
          form.setErrors(mapped.errors, mapped.unmatched.length > 0 ? mapped.unmatched.join(' ') : null);
          return;
        }
        throw failure;
      }
    },
  });
  const change = (next: boolean) => {
    if (form.submitting) {
      return;
    }
    if (!next) {
      if (issued) {
        void invalidate(CUSTOMERS);
      }
      setIssued(null);
      form.reset(EMPTY_ISSUE_TOKEN);
    }
    onOpenChange(next);
  };
  const ownerId = `${id}-owner`;
  const ownerListId = `${id}-owners`;
  const grouped = hasGroup(form.values.repositories, known);
  const expiryOptions: Option[] = [
    ...EXPIRY_PRESETS.map((days) => ({
      value: String(days),
      label: t('customers.period.days', { count: days }),
    })),
    { value: CUSTOM_EXPIRY, label: t('customers.issue.custom') },
  ];
  const ownerError = form.errors.owner;
  return (
    <Dialog.Root open={open} onOpenChange={change}>
      <Dialog.Content maxWidth="560px">
        <Dialog.Title>{issued ? t('customers.issue.issued') : t('customers.issue.title')}</Dialog.Title>
        <Dialog.Description size="2" color="gray">
          {t('customers.issue.intro')}
        </Dialog.Description>
        {issued ? (
          <Flex direction="column" gap="4" mt="4">
            <Callout.Root color="amber">
              <Callout.Text>{t('customers.issue.once')}</Callout.Text>
            </Callout.Root>
            <div className={styles.field}>
              <Text as="span" size="2" weight="medium">
                {t('customers.issue.token')}
              </Text>
              <div className={styles.secretRow}>
                <Code size="2" variant="soft" className={styles.secret}>
                  {issued.token}
                </Code>
                <CopyButton text={issued.token} label={t('customers.issue.copy')} />
              </div>
            </div>
            <div className={styles.field}>
              <Text as="span" size="2" weight="medium">
                {t('customers.issue.linesTitle')}
              </Text>
              <Text as="p" size="2" color="gray">
                {t('customers.issue.linesText')}
              </Text>
            </div>
            {issued.lines.map((line) => (
              <CodeBlock
                key={line.repository}
                title={line.repository}
                code={line.text}
                copyLabel={t('customers.issue.copy')}
              />
            ))}
            <Flex justify="end">
              <Dialog.Close>
                <Button>{t('customers.issue.done')}</Button>
              </Dialog.Close>
            </Flex>
          </Flex>
        ) : (
          <form onSubmit={form.submit} noValidate>
            <Flex direction="column" gap="4" mt="4">
              <TextField
                {...form.field('name')}
                label={t('customers.issue.name')}
                required
                maxLength={NAME_MAX}
                autoComplete="off"
              />
              <FieldShell
                id={ownerId}
                label={t('customers.issue.owner')}
                hint={t('customers.issue.ownerHint')}
                error={ownerError}
                required
              >
                <RadixTextField.Root
                  id={ownerId}
                  name="owner"
                  value={form.values.owner}
                  list={ownerListId}
                  required
                  autoComplete="off"
                  aria-invalid={ownerError ? true : undefined}
                  aria-describedby={ownerError ? `${ownerId}-hint ${ownerId}-error` : `${ownerId}-hint`}
                  onChange={(event) => form.set('owner', event.target.value)}
                />
                <datalist id={ownerListId}>
                  {usableOwners(users.data ?? []).map((owner) => (
                    <option key={owner} value={owner} />
                  ))}
                </datalist>
              </FieldShell>
              <MultiSelectField
                {...form.field('repositories')}
                id={`${id}-repositories`}
                label={t('customers.issue.repositories')}
                hint={t('customers.issue.repositoriesHint')}
                placeholder={t('customers.issue.repositoriesPlaceholder')}
                options={repositoryOptions(known)}
                required
              />
              {grouped && (
                <Callout.Root color="amber" size="1">
                  <Callout.Text>{t('customers.issue.group')}</Callout.Text>
                </Callout.Root>
              )}
              <SelectField
                id={`${id}-expiry`}
                label={t('customers.issue.expiry')}
                value={form.values.expiry}
                onChange={(value) => form.set('expiry', value)}
                options={expiryOptions}
                placeholder={t('customers.issue.choose')}
                error={form.errors.expiry}
                required
              />
              {form.values.expiry === CUSTOM_EXPIRY && (
                <NumberField
                  {...form.field('customDays')}
                  id={`${id}-days`}
                  label={t('customers.issue.customDays')}
                  min={1}
                  max={MAX_TOKEN_DAYS}
                  required
                />
              )}
              <FormError message={form.formError} />
            </Flex>
            <Flex gap="3" mt="5" justify="end">
              <Dialog.Close>
                <Button type="button" variant="soft" color="gray" disabled={form.submitting}>
                  {t('customers.issue.cancel')}
                </Button>
              </Dialog.Close>
              <Button type="submit" loading={form.submitting}>
                {t('customers.issue.submit')}
              </Button>
            </Flex>
          </form>
        )}
      </Dialog.Content>
    </Dialog.Root>
  );
}
