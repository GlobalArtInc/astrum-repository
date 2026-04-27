import { Button, Select, TextField } from '@radix-ui/themes';
import { Download, Stamp } from 'lucide-react';
import { useEffect, useId, useRef, useState } from 'react';
import { useParams } from 'react-router';
import { api, path, useApi, withQuery } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { describeError, ErrorState, Page, useToast } from '../../ui';
import { LOGGING_MARK, LOGS, LOGS_PAGE, MARK } from './api';
import styles from './Support.module.css';

const RATES = [
  { value: '0', label: 'support.viewer.manual' },
  { value: '20', label: 'support.viewer.every20' },
  { value: '60', label: 'support.viewer.everyMinute' },
  { value: '120', label: 'support.viewer.every2' },
  { value: '300', label: 'support.viewer.every5' },
] as const;

const SIZES = [
  { value: '25', label: 'support.viewer.last25' },
  { value: '50', label: 'support.viewer.last50' },
  { value: '100', label: 'support.viewer.last100' },
] as const;

export default function LogViewerPage() {
  const t = useT();
  const toast = useToast();
  const { file = '' } = useParams();
  const canMark = useCan(LOGGING_MARK);
  const ids = useId();
  const [rate, setRate] = useState('0');
  const [size, setSize] = useState('25');
  const [mark, setMark] = useState('');
  const [marking, setMarking] = useState(false);
  const source = `${LOGS}${path`/${file}`}`;
  const content = useApi<string>(withQuery(source, { bytesCount: -Number(size) * 1024 }), {
    refetchInterval: rate === '0' ? false : Number(rate) * 1000,
  });
  const view = useRef<HTMLPreElement>(null);
  const text = content.data ?? '';
  useEffect(() => {
    if (view.current && text) {
      view.current.scrollTop = view.current.scrollHeight;
    }
  }, [text]);
  const insert = async () => {
    setMarking(true);
    try {
      await api.post(MARK, JSON.stringify(mark.trim() || t('support.viewer.markPlaceholder')), {
        headers: { 'Content-Type': 'application/json' },
      });
      setMark('');
      toast.success(t('support.viewer.marked'));
      await content.refetch();
    } catch (error) {
      toast.error(t('support.viewer.markFailed'), describeError(t, error));
    } finally {
      setMarking(false);
    }
  };
  return (
    <Page
      gutter="small"
      className={styles.ruled}
      title={t('support.viewer.title')}
      description={file}
      breadcrumbs={[
        { label: t('support.crumbs.settings'), to: '/settings' },
        { label: t('support.logs.title'), to: LOGS_PAGE },
        { label: file },
      ]}
    >
      <div className={styles.stack}>
        <div className={styles.viewerHeader}>
          <h2 className={styles.viewerTitle}>{t('support.viewer.viewing', { file })}</h2>
          <Button asChild size="3" className={styles.large}>
            <a href={source} download={file}>
              <Download size={16} aria-hidden="true" />
              {t('support.common.download')}
            </a>
          </Button>
        </div>
        <div className={styles.toolbar}>
          {canMark && (
            <form
              className={styles.toolbarGroup}
              onSubmit={(event) => {
                event.preventDefault();
                void insert();
              }}
            >
              <label htmlFor={`${ids}-mark`} className={styles.toolbarLabel}>
                {t('support.viewer.mark')}
              </label>
              <TextField.Root
                id={`${ids}-mark`}
                size="2"
                className={styles.markInput}
                value={mark}
                placeholder={t('support.viewer.markPlaceholder')}
                onChange={(event) => setMark(event.target.value)}
              />
              <Button
                type="submit"
                size="3"
                variant="outline"
                color="gray"
                className={styles.large}
                loading={marking}
              >
                <Stamp size={16} aria-hidden="true" />
                {t('support.viewer.insert')}
              </Button>
            </form>
          )}
          <div className={styles.toolbarEnd}>
            <div className={styles.toolbarGroup}>
              <span id={`${ids}-rate`} className={styles.toolbarLabel}>
                {t('support.viewer.refreshRate')}
              </span>
              <Select.Root value={rate} onValueChange={setRate}>
                <Select.Trigger className={styles.rateSelect} aria-labelledby={`${ids}-rate`} />
                <Select.Content position="popper">
                  {RATES.map((option) => (
                    <Select.Item key={option.value} value={option.value}>
                      {t(option.label)}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>
            </div>
            <div className={styles.toolbarGroup}>
              <span id={`${ids}-size`} className={styles.toolbarLabel}>
                {t('support.viewer.size')}
              </span>
              <Select.Root value={size} onValueChange={setSize}>
                <Select.Trigger className={styles.sizeSelect} aria-labelledby={`${ids}-size`} />
                <Select.Content position="popper">
                  {SIZES.map((option) => (
                    <Select.Item key={option.value} value={option.value}>
                      {t(option.label)}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>
            </div>
          </div>
        </div>
        {content.error ? (
          <ErrorState error={content.error} onRetry={() => void content.refetch()} />
        ) : (
          <pre
            ref={view}
            role="log"
            className={styles.log}
            aria-label={t('support.viewer.content', { file })}
            aria-busy={content.isPending || undefined}
            // biome-ignore lint/a11y/noNoninteractiveTabindex: the log scrolls and must be reachable by keyboard
            tabIndex={0}
          >
            {content.isPending ? '' : text || t('support.viewer.empty')}
          </pre>
        )}
      </div>
    </Page>
  );
}
