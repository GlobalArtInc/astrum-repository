import { Button } from '@radix-ui/themes';
import { Trash2 } from 'lucide-react';
import { type FormEvent, useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ConfirmDialog, describeError, ErrorState, FormError, Page, useToast } from '../../ui';
import { Choice, Input } from '../identity/shared/Fields';
import { EditBar, EditForm, FormSection } from '../security/shared/Edit';
import { type Level, LOGGERS, LOGGING_PAGE, LOGGING_UPDATE, type Logger } from './api';
import styles from './Support.module.css';

const CHOICES: readonly Level[] = ['ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE', 'OFF'];

const NAME = /^(ROOT|[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*)$/;

export default function LoggerPage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const { name: existing } = useParams();
  const creating = existing === undefined;
  const canUpdate = useCan(LOGGING_UPDATE);
  const logger = useApi<Logger>(creating ? null : `${LOGGERS}${path`/${existing ?? ''}`}`);
  const [name, setName] = useState('');
  const [level, setLevel] = useState<Level>('INFO');
  const [touched, setTouched] = useState(false);
  const [saving, setSaving] = useState(false);
  const [failure, setFailure] = useState<unknown>(null);
  const [deleting, setDeleting] = useState(false);
  useEffect(() => {
    if (logger.data) {
      setName(logger.data.name);
      setLevel(logger.data.level);
    }
  }, [logger.data]);
  const trimmed = name.trim();
  const nameError = !touched
    ? undefined
    : trimmed === ''
      ? t('support.logger.nameRequired')
      : NAME.test(trimmed)
        ? undefined
        : t('support.logger.nameInvalid');
  const dirty = creating ? trimmed !== '' : logger.data !== undefined && level !== logger.data.level;
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setTouched(true);
    if (!NAME.test(trimmed)) {
      return;
    }
    setSaving(true);
    setFailure(null);
    try {
      await api.put(`${LOGGERS}${path`/${trimmed}`}`, { name: trimmed, level });
      toast.success(
        creating
          ? t('support.logger.created', { name: trimmed })
          : t('support.logger.updated', { name: trimmed, level }),
      );
      await invalidate(LOGGERS);
      navigate(LOGGING_PAGE);
    } catch (error) {
      setFailure(error);
    } finally {
      setSaving(false);
    }
  };
  const remove = async () => {
    await api.post(`${LOGGERS}${path`/${trimmed}/reset`}`);
    toast.success(t('support.logger.removed', { name: trimmed }));
    await invalidate(LOGGERS);
    navigate(LOGGING_PAGE);
  };
  const title = creating ? t('support.logger.createTitle') : (existing ?? '');
  return (
    <Page
      gutter="small"
      className={styles.ruled}
      title={title}
      description={creating ? undefined : t('support.logger.editIntro')}
      breadcrumbs={[
        { label: t('support.crumbs.settings'), to: '/settings' },
        { label: t('support.logging.title'), to: LOGGING_PAGE },
        { label: creating ? t('support.logger.createCrumb') : (existing ?? '') },
      ]}
    >
      {logger.error ? (
        <ErrorState error={logger.error} onRetry={() => void logger.refetch()} />
      ) : (
        <div className={styles.loggerForm}>
          <EditForm
            id="logger-form"
            onSubmit={submit}
            bar={
              <EditBar
                start={
                  !creating &&
                  canUpdate &&
                  logger.data?.override && (
                    <Button
                      type="button"
                      size="3"
                      color="red"
                      className={styles.large}
                      onClick={() => setDeleting(true)}
                    >
                      <Trash2 size={16} aria-hidden="true" />
                      {t('support.logger.delete')}
                    </Button>
                  )
                }
                end={
                  <>
                    <Button type="button" variant="outline" onClick={() => navigate(LOGGING_PAGE)}>
                      {t('support.logger.cancel')}
                    </Button>
                    {canUpdate && (
                      <Button type="submit" disabled={!dirty} loading={saving}>
                        {creating ? t('support.logger.createTitle') : t('support.logger.save')}
                      </Button>
                    )}
                  </>
                }
              />
            }
          >
            <FormSection>
              <FormError message={failure === null ? null : describeError(t, failure)} />
              <Input
                label={t('support.logger.name')}
                name="name"
                required
                help={t('support.logger.nameHelp')}
                placeholder={t('support.logger.namePlaceholder')}
                value={name}
                disabled={!creating}
                autoFocus={creating}
                error={nameError}
                onChange={(value) => {
                  setName(value);
                  setTouched(true);
                }}
              />
              <div className={styles.levelField}>
                <Choice
                  label={t('support.logger.level')}
                  name="level"
                  help={t('support.logger.levelHelp')}
                  value={level}
                  disabled={!canUpdate}
                  options={CHOICES.map((option) => ({ value: option, label: option }))}
                  onChange={(value) => setLevel(value as Level)}
                />
                <p className={styles.fieldNote}>
                  {t('support.logger.overrides')}
                  {!creating && logger.data?.override && (
                    <>
                      <br />
                      {t('support.logger.deleteToRevert')}
                    </>
                  )}
                </p>
              </div>
            </FormSection>
          </EditForm>
        </div>
      )}
      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        title={t('support.logger.deleteTitle')}
        description={t('support.logger.deleteText', { name: trimmed })}
        confirmLabel={t('support.logger.delete')}
        danger
        onConfirm={remove}
      />
    </Page>
  );
}
