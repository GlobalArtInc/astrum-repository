import { Button } from '@radix-ui/themes';
import { CircleCheck, Download } from 'lucide-react';
import { type FormEvent, useState } from 'react';
import { api, path } from '../../api';
import { useT } from '../../i18n';
import { describeError, FileSize, FormError, Page } from '../../ui';
import { Check, Choice } from '../identity/shared/Fields';
import { type CreatedZip, DOWNLOAD, SUPPORT_ZIP } from './api';
import styles from './Support.module.css';

const CONTENTS = [
  'systemInformation',
  'configuration',
  'security',
  'log',
  'taskLog',
  'auditLog',
  'metrics',
] as const;

type Content = (typeof CONTENTS)[number];

const LABELS: Record<Content, string> = {
  systemInformation: 'support.zip.systemInformation',
  configuration: 'support.zip.configuration',
  security: 'support.zip.security',
  log: 'support.zip.log',
  taskLog: 'support.zip.taskLog',
  auditLog: 'support.zip.auditLog',
  metrics: 'support.zip.metrics',
};

export default function SupportZipPage() {
  const t = useT();
  const [contents, setContents] = useState<Record<Content, boolean>>(
    Object.fromEntries(CONTENTS.map((content) => [content, true])) as Record<Content, boolean>,
  );
  const [archived, setArchived] = useState<'0' | '1' | '2' | '3'>('0');
  const [limitFileSizes, setLimitFileSizes] = useState(true);
  const [limitZipSize, setLimitZipSize] = useState(true);
  const [creating, setCreating] = useState(false);
  const [failure, setFailure] = useState<unknown>(null);
  const [created, setCreated] = useState<CreatedZip | null>(null);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setCreating(true);
    setFailure(null);
    try {
      setCreated(
        await api.post<CreatedZip>(SUPPORT_ZIP, {
          ...contents,
          archivedLog: Number(archived),
          limitFileSizes,
          limitZipSize,
        }),
      );
    } catch (error) {
      setFailure(error);
    } finally {
      setCreating(false);
    }
  };
  return (
    <Page
      gutter="small"
      title={t('support.zip.title')}
      description={t('support.zip.intro')}
      breadcrumbs={[
        { label: t('support.crumbs.settings'), to: '/settings' },
        { label: t('support.zip.title') },
      ]}
    >
      {created ? (
        <section className={styles.zipResult} aria-labelledby="support-zip-created">
          <h2 id="support-zip-created" className={styles.zipResultTitle}>
            <CircleCheck size={22} className={styles.ok} aria-hidden="true" />
            {t('support.zip.created')}
          </h2>
          <p className={styles.zipResultText}>{t('support.zip.createdText')}</p>
          {created.truncated && <p className={styles.zipWarning}>{t('support.zip.truncated')}</p>}
          <dl className={styles.zipFacts}>
            <div>
              <dt>{t('support.zip.name')}</dt>
              <dd>{created.name}</dd>
            </div>
            <div>
              <dt>{t('support.zip.size')}</dt>
              <dd>
                <FileSize bytes={created.size} />
              </dd>
            </div>
            <div>
              <dt>{t('support.zip.path')}</dt>
              <dd className={styles.mono}>{created.file}</dd>
            </div>
          </dl>
          <Button asChild size="3" className={`${styles.large} ${styles.zipDownload}`}>
            <a href={`${DOWNLOAD}${path`/${created.name}`}`} download={created.name}>
              <Download size={16} aria-hidden="true" />
              {t('support.common.download')}
            </a>
          </Button>
        </section>
      ) : (
        <form className={styles.zipForm} onSubmit={submit} noValidate>
          <div className={styles.callout}>
            <p>{t('support.zip.noSend')}</p>
            <p>{t('support.zip.mayTake')}</p>
          </div>
          <div className={styles.zipBody}>
            <h2 className={styles.zipHeading}>{t('support.zip.contents')}</h2>
            <div className={styles.zipChecks}>
              {CONTENTS.map((content) => (
                <Check
                  key={content}
                  name={content}
                  label={t(LABELS[content])}
                  checked={contents[content]}
                  onChange={(checked) => setContents({ ...contents, [content]: checked })}
                />
              ))}
            </div>
            <div className={styles.zipSelect}>
              <Choice
                label={t('support.zip.archived')}
                name="archivedLog"
                value={archived}
                options={[
                  { value: '0', label: t('support.zip.none') },
                  { value: '1', label: t('support.zip.days', { count: 1 }) },
                  { value: '2', label: t('support.zip.days', { count: 2 }) },
                  { value: '3', label: t('support.zip.days', { count: 3 }) },
                ]}
                onChange={(value) => setArchived(value as typeof archived)}
              />
            </div>
            <hr className={styles.rule} />
            <h2 className={styles.zipHeading}>{t('support.zip.options')}</h2>
            <div className={styles.zipChecks}>
              <Check
                name="limitFileSizes"
                label={t('support.zip.limitFiles')}
                checked={limitFileSizes}
                onChange={setLimitFileSizes}
              />
              <Check
                name="limitZipSize"
                label={t('support.zip.limitZip')}
                checked={limitZipSize}
                onChange={setLimitZipSize}
              />
            </div>
            <FormError
              message={failure === null ? null : `${t('support.zip.failed')}: ${describeError(t, failure)}`}
            />
            <div>
              <Button type="submit" size="3" className={styles.large} loading={creating}>
                {creating ? t('support.zip.creating') : t('support.zip.create')}
              </Button>
            </div>
          </div>
        </form>
      )}
    </Page>
  );
}
