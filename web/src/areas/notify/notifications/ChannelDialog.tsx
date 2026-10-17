import {
  Badge,
  Button,
  Callout,
  Checkbox,
  Dialog,
  RadioCards,
  SegmentedControl,
  Select,
  Switch,
  Text,
  TextField,
} from '@radix-ui/themes';
import { Hash, Info, Mail, Send, Webhook } from 'lucide-react';
import { type ReactNode, useId, useMemo, useState } from 'react';
import { ApiError, api, useApi, useInvalidateApi } from '../../../api';
import { useT } from '../../../i18n';
import { describeError, useToast } from '../../../ui';
import styles from '../Notify.module.css';
import {
  CHANNELS,
  type Channel,
  type ChannelType,
  type ChannelValues,
  CONNECTION_HELP,
  EVENT_HELP,
  EVENT_LABELS,
  EVENTS,
  EXAMPLE_BODY,
  initialValues,
  type TestResult,
  TYPE_LABELS,
  toRequest,
  validateChannel,
} from './model';

interface Repository {
  name: string;
  format: string;
  type: string;
}

type Errors = Partial<Record<keyof ChannelValues, string>>;

const TYPE_ICONS: Record<ChannelType, typeof Send> = {
  telegram: Send,
  slack: Hash,
  webhook: Webhook,
  email: Mail,
};

function Field({
  id,
  label,
  hint,
  error,
  children,
}: {
  id: string;
  label: string;
  hint?: ReactNode;
  error?: string;
  children: ReactNode;
}) {
  return (
    <div className={styles.field}>
      <label htmlFor={id} className={styles.label}>
        {label}
      </label>
      {children}
      {hint && !error && <p className={styles.hint}>{hint}</p>}
      {error && (
        <p className={styles.error} role="alert">
          {error}
        </p>
      )}
    </div>
  );
}

function SavedSecret({
  stored,
  removed,
  onRemove,
  allowRemove,
}: {
  stored: boolean;
  removed: boolean;
  onRemove(): void;
  allowRemove: boolean;
}) {
  const t = useT();
  if (!stored) {
    return null;
  }
  if (removed) {
    return <p className={styles.hint}>{t('notify.notifications.savedSecretRemoved')}</p>;
  }
  return (
    <p className={styles.hint}>
      {t('notify.notifications.secretSaved')}{' '}
      {allowRemove && (
        <button type="button" className={styles.linkButton} onClick={onRemove}>
          {t('notify.notifications.savedSecretRemove')}
        </button>
      )}
    </p>
  );
}

function Help({ type }: { type: ChannelType }) {
  const t = useT();
  const steps: ReactNode[] =
    type === 'telegram'
      ? [
          t.rich('notify.notifications.telegramHelp.bot', {
            link: (
              <a
                key="link"
                href="https://t.me/BotFather"
                target="_blank"
                rel="noreferrer"
                className={styles.helpLink}
              >
                @BotFather
              </a>
            ),
          }),
          t('notify.notifications.telegramHelp.chat'),
          t.rich('notify.notifications.telegramHelp.id', {
            url: <code key="url">{'https://api.telegram.org/bot<token>/getUpdates'}</code>,
            field: <code key="field">chat.id</code>,
          }),
        ]
      : type === 'slack'
        ? [t('notify.notifications.slackHelp.webhook'), t('notify.notifications.slackHelp.url')]
        : type === 'webhook'
          ? [
              t('notify.notifications.webhookHelp.endpoint'),
              t.rich('notify.notifications.webhookHelp.headers', {
                event: <code key="event">X-Astrum-Event</code>,
                published: <code key="published">component.published</code>,
                updated: <code key="updated">component.updated</code>,
                deleted: <code key="deleted">component.deleted</code>,
                test: <code key="test">test</code>,
                delivery: <code key="delivery">X-Astrum-Delivery</code>,
              }),
              t.rich('notify.notifications.webhookHelp.signature', {
                header: <code key="header">X-Astrum-Signature</code>,
                prefix: <code key="prefix">sha256=</code>,
              }),
            ]
          : [t('notify.notifications.emailHelp.server'), t('notify.notifications.emailHelp.recipients')];
  const title =
    type === 'telegram'
      ? t('notify.notifications.telegramHelp.title')
      : type === 'slack'
        ? t('notify.notifications.slackHelp.title')
        : type === 'webhook'
          ? t('notify.notifications.webhookHelp.title')
          : t('notify.notifications.emailHelp.title');
  return (
    <Callout.Root color="gray" variant="surface" size="1" className={styles.help}>
      <Callout.Icon>
        <Info size={16} />
      </Callout.Icon>
      <div>
        <Text as="p" size="2" weight="medium">
          {title}
        </Text>
        <ol className={styles.steps}>
          {steps.map((step, index) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: the steps are fixed
            <li key={index}>{step}</li>
          ))}
        </ol>
        {type === 'webhook' && (
          <details className={styles.example}>
            <summary>{t('notify.notifications.webhookHelp.example')}</summary>
            <pre>{EXAMPLE_BODY}</pre>
          </details>
        )}
      </div>
    </Callout.Root>
  );
}

export function ChannelDialog({
  channel,
  open,
  onClose,
}: {
  channel: Channel | null;
  open: boolean;
  onClose(): void;
}) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const repositories = useApi<Repository[]>('/service/rest/v1/repositories');
  const id = useId();
  const [values, setValues] = useState<ChannelValues>(() => initialValues(channel, 'ru'));
  const [errors, setErrors] = useState<Errors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [filter, setFilter] = useState('');
  const set = <K extends keyof ChannelValues>(name: K, value: ChannelValues[K]) => {
    setValues((current) => ({ ...current, [name]: value }));
    setErrors((current) => ({ ...current, [name]: undefined }));
  };
  const sorted = useMemo(
    () =>
      [...(repositories.data ?? [])]
        .sort((left, right) => left.name.localeCompare(right.name))
        .filter((repository) => repository.name.toLowerCase().includes(filter.trim().toLowerCase())),
    [repositories.data, filter],
  );
  const stored = (flag: keyof Channel) => Boolean(channel?.[flag]) && channel?.type === values.type;

  const failure = (error: unknown) => {
    if (error instanceof ApiError && error.status === 400 && error.fieldErrors.length > 0) {
      const mapped: Errors = {};
      const unmatched: string[] = [];
      for (const problem of error.fieldErrors) {
        if (problem.id in values) {
          mapped[problem.id as keyof ChannelValues] = problem.message;
        } else {
          unmatched.push(problem.message);
        }
      }
      setErrors(mapped);
      setFormError(unmatched.length > 0 ? unmatched.join(' ') : null);
    } else {
      setFormError(describeError(t, error));
    }
  };

  const save = async () => {
    const problems = validateChannel(t, values, channel);
    setErrors(problems);
    if (Object.values(problems).some(Boolean)) {
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      const body = toRequest(values, channel);
      if (channel) {
        await api.put(`${CHANNELS}/${encodeURIComponent(channel.id)}`, body);
      } else {
        await api.post(CHANNELS, body);
      }
      toast.success(t('notify.notifications.saved', { name: values.name.trim() }));
      await invalidate(CHANNELS);
      onClose();
    } catch (error) {
      failure(error);
    } finally {
      setSaving(false);
    }
  };

  const test = async () => {
    const problems = validateChannel(t, values, channel, true);
    setErrors(problems);
    if (Object.values(problems).some(Boolean)) {
      return;
    }
    setTesting(true);
    setFormError(null);
    try {
      const result = await api.post<TestResult>(`${CHANNELS}/test`, toRequest(values, channel));
      if (result.success) {
        toast.success(t('notify.notifications.testDelivered'));
      } else {
        setFormError(`${t('notify.notifications.testFailure')}: ${result.message ?? ''}`);
      }
    } catch (error) {
      failure(error);
    } finally {
      setTesting(false);
    }
  };

  const secretLabel =
    values.type === 'telegram'
      ? t('notify.notifications.botToken')
      : values.type === 'slack'
        ? t('notify.notifications.slackUrl')
        : t('notify.notifications.webhookUrl');

  return (
    <Dialog.Root open={open} onOpenChange={(next) => !next && onClose()}>
      <Dialog.Content maxWidth="600px" className={styles.dialog} aria-describedby={`${id}-description`}>
        <Dialog.Title>
          {channel ? t('notify.notifications.editTitle') : t('notify.notifications.createTitle')}
        </Dialog.Title>
        <Dialog.Description id={`${id}-description`} size="2" color="gray" className={styles.dialogText}>
          {t('notify.notifications.description')}
        </Dialog.Description>
        <form
          noValidate
          className={styles.form}
          onSubmit={(event) => {
            event.preventDefault();
            void save();
          }}
        >
          <Field id={`${id}-name`} label={t('notify.notifications.name')} error={errors.name}>
            <TextField.Root
              id={`${id}-name`}
              value={values.name}
              autoFocus
              maxLength={100}
              placeholder={t('notify.notifications.namePlaceholder')}
              onChange={(event) => set('name', event.target.value)}
            />
          </Field>
          <div className={styles.field}>
            <span className={styles.label} id={`${id}-type`}>
              {t('notify.notifications.type')}
            </span>
            <RadioCards.Root
              value={values.type}
              columns={{ initial: '1', sm: '4' }}
              gap="4"
              aria-labelledby={`${id}-type`}
              className={styles.types}
              onValueChange={(value) => set('type', value as ChannelType)}
            >
              {(['telegram', 'slack', 'webhook', 'email'] as ChannelType[]).map((type) => {
                const Icon = TYPE_ICONS[type];
                return (
                  <RadioCards.Item key={type} value={type} className={styles.typeCard}>
                    <span className={styles.typeCardInner}>
                      <Icon size={16} aria-hidden="true" />
                      {t(TYPE_LABELS[type])}
                    </span>
                  </RadioCards.Item>
                );
              })}
            </RadioCards.Root>
          </div>
          <Help type={values.type} />
          {values.type === 'email' ? (
            <Field
              id={`${id}-recipients`}
              label={t('notify.notifications.recipients')}
              hint={t('notify.notifications.recipientsHelp')}
              error={errors.recipients}
            >
              <TextField.Root
                id={`${id}-recipients`}
                value={values.recipients}
                placeholder="team@example.com"
                onChange={(event) => set('recipients', event.target.value)}
              />
            </Field>
          ) : (
            <Field
              id={`${id}-secret`}
              label={secretLabel}
              hint={
                stored('hasSecret')
                  ? t('notify.notifications.secretSaved')
                  : values.type === 'telegram'
                    ? undefined
                    : t('notify.notifications.webhookHelpText')
              }
              error={errors.secret}
            >
              <TextField.Root
                id={`${id}-secret`}
                type="password"
                autoComplete="off"
                value={values.secret}
                placeholder={
                  stored('hasSecret')
                    ? '••••••••'
                    : values.type === 'telegram'
                      ? '123456789:AAH…'
                      : values.type === 'slack'
                        ? 'https://hooks.slack.com/services/…'
                        : 'https://ci.example.com/hooks/astrum'
                }
                onChange={(event) => set('secret', event.target.value)}
              />
            </Field>
          )}
          {values.type === 'telegram' && (
            <div className={styles.chatPair}>
              <Field
                id={`${id}-chat`}
                label={t('notify.notifications.chatId')}
                hint={t('notify.notifications.chatIdHelp')}
                error={errors.chatId}
              >
                <TextField.Root
                  id={`${id}-chat`}
                  value={values.chatId}
                  placeholder="-1001234567890"
                  onChange={(event) => set('chatId', event.target.value)}
                />
              </Field>
              <Field
                id={`${id}-thread`}
                label={t('notify.notifications.thread')}
                hint={t('notify.notifications.threadHelp')}
                error={errors.messageThreadId}
              >
                <TextField.Root
                  id={`${id}-thread`}
                  value={values.messageThreadId}
                  inputMode="numeric"
                  onChange={(event) => set('messageThreadId', event.target.value)}
                />
              </Field>
            </div>
          )}
          {values.type === 'webhook' && (
            <div className={styles.pair}>
              <Field
                id={`${id}-signing`}
                label={t('notify.notifications.signingSecret')}
                hint={
                  stored('hasSigningSecret')
                    ? undefined
                    : t.rich('notify.notifications.signingSecretHelp', {
                        header: <code key="header">X-Astrum-Signature</code>,
                      })
                }
              >
                <TextField.Root
                  id={`${id}-signing`}
                  type="password"
                  autoComplete="off"
                  value={values.signingSecret}
                  placeholder={
                    stored('hasSigningSecret') && !values.removeSigningSecret ? '••••••••' : undefined
                  }
                  onChange={(event) => set('signingSecret', event.target.value)}
                />
                <SavedSecret
                  stored={stored('hasSigningSecret')}
                  removed={values.removeSigningSecret}
                  allowRemove
                  onRemove={() => set('removeSigningSecret', true)}
                />
              </Field>
              <Field
                id={`${id}-authorization`}
                label={t('notify.notifications.authorization')}
                hint={
                  stored('hasAuthorization')
                    ? undefined
                    : t.rich('notify.notifications.authorizationHelp', {
                        header: <code key="header">Authorization</code>,
                        example: <code key="example">Bearer token</code>,
                      })
                }
                error={errors.authorization}
              >
                <TextField.Root
                  id={`${id}-authorization`}
                  type="password"
                  autoComplete="off"
                  value={values.authorization}
                  placeholder={
                    stored('hasAuthorization') && !values.removeAuthorization ? '••••••••' : 'Bearer …'
                  }
                  onChange={(event) => set('authorization', event.target.value)}
                />
                <SavedSecret
                  stored={stored('hasAuthorization')}
                  removed={values.removeAuthorization}
                  allowRemove
                  onRemove={() => set('removeAuthorization', true)}
                />
              </Field>
            </div>
          )}
          {values.type !== 'email' && (
            <div className={styles.field}>
              <span className={styles.label} id={`${id}-connection`}>
                {t('notify.notifications.connection')}
              </span>
              <Select.Root
                value={values.proxyMode}
                onValueChange={(value) => set('proxyMode', value as ChannelValues['proxyMode'])}
              >
                <Select.Trigger aria-labelledby={`${id}-connection`} className={styles.wide} />
                <Select.Content position="popper">
                  <Select.Item value="global">{t('notify.notifications.connectionModes.global')}</Select.Item>
                  <Select.Item value="none">{t('notify.notifications.connectionModes.none')}</Select.Item>
                  <Select.Item value="custom">{t('notify.notifications.connectionModes.custom')}</Select.Item>
                </Select.Content>
              </Select.Root>
              <p className={styles.hint}>{t(CONNECTION_HELP[values.proxyMode])}</p>
            </div>
          )}
          {values.type !== 'email' && values.proxyMode === 'custom' && (
            <div className={styles.proxy}>
              <div className={styles.field}>
                <span className={styles.label} id={`${id}-proxy-type`}>
                  {t('notify.notifications.proxyType')}
                </span>
                <SegmentedControl.Root
                  size="1"
                  value={values.proxyType}
                  aria-labelledby={`${id}-proxy-type`}
                  onValueChange={(value) => set('proxyType', value as ChannelValues['proxyType'])}
                  className={styles.segmented}
                >
                  <SegmentedControl.Item value="http">HTTP</SegmentedControl.Item>
                  <SegmentedControl.Item value="socks5">SOCKS5</SegmentedControl.Item>
                </SegmentedControl.Root>
              </div>
              <div className={styles.hostPort}>
                <Field
                  id={`${id}-proxy-host`}
                  label={t('notify.notifications.proxyHost')}
                  error={errors.proxyHost}
                >
                  <TextField.Root
                    id={`${id}-proxy-host`}
                    value={values.proxyHost}
                    onChange={(event) => set('proxyHost', event.target.value)}
                  />
                </Field>
                <Field
                  id={`${id}-proxy-port`}
                  label={t('notify.notifications.proxyPort')}
                  error={errors.proxyPort}
                >
                  <TextField.Root
                    id={`${id}-proxy-port`}
                    value={values.proxyPort}
                    inputMode="numeric"
                    onChange={(event) => set('proxyPort', event.target.value)}
                  />
                </Field>
              </div>
              <div className={styles.pair}>
                <Field
                  id={`${id}-proxy-user`}
                  label={t('notify.notifications.proxyUsername')}
                  error={errors.proxyUsername}
                >
                  <TextField.Root
                    id={`${id}-proxy-user`}
                    value={values.proxyUsername}
                    autoComplete="off"
                    onChange={(event) => set('proxyUsername', event.target.value)}
                  />
                </Field>
                <Field
                  id={`${id}-proxy-password`}
                  label={t('notify.notifications.proxyPassword')}
                  hint={
                    stored('hasProxyPassword') ? undefined : t('notify.notifications.proxyCredentialsHelp')
                  }
                >
                  <TextField.Root
                    id={`${id}-proxy-password`}
                    type="password"
                    autoComplete="new-password"
                    value={values.proxyPassword}
                    placeholder={
                      stored('hasProxyPassword') && !values.removeProxyPassword ? '••••••••' : undefined
                    }
                    onChange={(event) => set('proxyPassword', event.target.value)}
                  />
                  <SavedSecret
                    stored={stored('hasProxyPassword')}
                    removed={values.removeProxyPassword}
                    allowRemove
                    onRemove={() => set('removeProxyPassword', true)}
                  />
                </Field>
              </div>
            </div>
          )}
          <div className={styles.field} role="group" aria-labelledby={`${id}-events`}>
            <span className={styles.label} id={`${id}-events`}>
              {t('notify.notifications.events')}
            </span>
            <div className={styles.checks}>
              {EVENTS.map((event) => (
                // biome-ignore lint/a11y/noLabelWithoutControl: the checkbox inside is the control
                <label key={event} className={styles.check}>
                  <Checkbox
                    checked={values.events.includes(event)}
                    onCheckedChange={(checked) =>
                      set(
                        'events',
                        checked === true
                          ? [...values.events, event]
                          : values.events.filter((known) => known !== event),
                      )
                    }
                  />
                  <span>
                    <span className={styles.checkTitle}>{t(EVENT_LABELS[event])}</span>
                    <span className={styles.checkText}>{t(EVENT_HELP[event])}</span>
                  </span>
                </label>
              ))}
            </div>
            {errors.events && (
              <p className={styles.error} role="alert">
                {errors.events}
              </p>
            )}
          </div>
          <div className={styles.field} role="group" aria-labelledby={`${id}-repositories`}>
            <span className={styles.label} id={`${id}-repositories`}>
              {t('notify.notifications.repositories')}
            </span>
            <div className={styles.repositories}>
              <TextField.Root
                size="1"
                value={filter}
                aria-label={t('notify.notifications.repositoriesFilter')}
                placeholder={t('notify.notifications.repositoriesFilter')}
                onChange={(event) => setFilter(event.target.value)}
              />
              <div className={styles.repositoryList}>
                {sorted.length === 0 && !repositories.isPending && (
                  <p className={styles.hint}>{t('notify.notifications.repositoriesEmpty')}</p>
                )}
                {sorted.map((repository) => (
                  // biome-ignore lint/a11y/noLabelWithoutControl: the checkbox inside is the control
                  <label key={repository.name} className={styles.repository}>
                    <Checkbox
                      checked={values.repositories.includes(repository.name)}
                      onCheckedChange={(checked) =>
                        set(
                          'repositories',
                          checked === true
                            ? [...values.repositories, repository.name]
                            : values.repositories.filter((name) => name !== repository.name),
                        )
                      }
                    />
                    <span>{repository.name}</span>
                    <Badge color="gray">{repository.format}</Badge>
                  </label>
                ))}
              </div>
            </div>
            <p className={styles.hint}>{t('notify.notifications.repositoriesHelp')}</p>
          </div>
          <div className={styles.languageRow}>
            <div className={styles.field}>
              <span className={styles.label} id={`${id}-language`}>
                {t('notify.notifications.language')}
              </span>
              <Select.Root
                value={values.language}
                onValueChange={(value) => set('language', value as ChannelValues['language'])}
              >
                <Select.Trigger aria-labelledby={`${id}-language`} className={styles.language} />
                <Select.Content position="popper">
                  <Select.Item value="ru">{t('notify.notifications.languages.ru')}</Select.Item>
                  <Select.Item value="en">{t('notify.notifications.languages.en')}</Select.Item>
                </Select.Content>
              </Select.Root>
            </div>
            {/* biome-ignore lint/a11y/noLabelWithoutControl: the switch inside is the control */}
            <label className={styles.enabled}>
              <Switch checked={values.enabled} onCheckedChange={(checked) => set('enabled', checked)} />
              {t('notify.notifications.enabled')}
            </label>
          </div>
          {formError && (
            <Callout.Root color="red" role="alert">
              <Callout.Text>{formError}</Callout.Text>
            </Callout.Root>
          )}
          <div className={styles.footer}>
            <Button type="button" variant="soft" color="gray" loading={testing} onClick={() => void test()}>
              {t('notify.notifications.test')}
            </Button>
            <div className={styles.footerEnd}>
              <Dialog.Close>
                <Button type="button" variant="soft" color="gray">
                  {t('notify.notifications.cancel')}
                </Button>
              </Dialog.Close>
              <Button type="submit" loading={saving}>
                {t('notify.notifications.save')}
              </Button>
            </div>
          </div>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
