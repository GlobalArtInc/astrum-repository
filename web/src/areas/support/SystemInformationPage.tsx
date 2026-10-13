import { Button, Select } from '@radix-ui/themes';
import {
  ChevronDown,
  ChevronRight,
  ChevronsDownUp,
  ChevronsUpDown,
  Copy,
  Download,
  RefreshCw,
} from 'lucide-react';
import { useId, useState } from 'react';
import { useApi } from '../../api';
import { useT } from '../../i18n';
import { copyText, ErrorState, Page, useToast } from '../../ui';
import { HelpCard } from '../security/shared/AdminList';
import { DOCS, flatten, orderSections, SYSTEM_INFORMATION, saveJson, sectionTitle } from './api';
import styles from './Support.module.css';

type Report = Record<string, unknown>;

const OPEN_FIRST = 2;

export default function SystemInformationPage() {
  const t = useT();
  const toast = useToast();
  const ids = useId();
  const report = useApi<Report>(SYSTEM_INFORMATION);
  const sections = orderSections(report.data ?? {});
  const [open, setOpen] = useState<Record<string, boolean>>({});
  const isOpen = (key: string, index: number) => open[key] ?? index < OPEN_FIRST;
  const setAll = (value: boolean) => setOpen(Object.fromEntries(sections.map(([key]) => [key, value])));
  const jump = (key: string) => {
    setOpen((previous) => ({ ...previous, [key]: true }));
    requestAnimationFrame(() =>
      document.getElementById(`${ids}-${key}`)?.scrollIntoView({ behavior: 'smooth', block: 'start' }),
    );
  };
  return (
    <Page
      gutter="large"
      title={t('support.sysinfo.title')}
      description={t('support.sysinfo.intro')}
      breadcrumbs={[
        { label: t('support.crumbs.settings'), to: '/settings' },
        { label: t('support.sysinfo.title') },
      ]}
      actions={
        <div className={styles.actions}>
          <Button
            variant="ghost"
            size="3"
            className={styles.ghostAction}
            onClick={async () => {
              await report.refetch();
              toast.success(t('support.sysinfo.refreshed'));
            }}
          >
            <RefreshCw size={16} aria-hidden="true" />
            {t('support.common.refresh')}
          </Button>
          <Button
            variant="ghost"
            size="3"
            className={styles.ghostAction}
            disabled={!report.data}
            onClick={async () => {
              await copyText(JSON.stringify(report.data, null, 2));
              toast.success(t('support.common.copied'));
            }}
          >
            <Copy size={16} aria-hidden="true" />
            {t('support.common.copy')}
          </Button>
          <Button
            variant="ghost"
            size="3"
            className={styles.ghostAction}
            disabled={!report.data}
            onClick={() => {
              saveJson('system-information.json', report.data);
              toast.success(t('support.sysinfo.downloaded'));
            }}
          >
            <Download size={16} aria-hidden="true" />
            {t('support.common.download')}
          </Button>
        </div>
      }
    >
      <div className={styles.stack}>
        {report.error ? (
          <ErrorState error={report.error} onRetry={() => void report.refetch()} />
        ) : (
          <>
            <div className={styles.jumpBar}>
              <div className={styles.toolbarGroup}>
                <span id={`${ids}-jump`} className={styles.jumpLabel}>
                  {t('support.sysinfo.jumpTo')}
                </span>
                <Select.Root value="" onValueChange={jump}>
                  <Select.Trigger
                    className={styles.jumpSelect}
                    placeholder={t('support.sysinfo.selectSection')}
                    aria-labelledby={`${ids}-jump`}
                  />
                  <Select.Content position="popper">
                    {sections.map(([key]) => (
                      <Select.Item key={key} value={key}>
                        {sectionTitle(key)}
                      </Select.Item>
                    ))}
                  </Select.Content>
                </Select.Root>
              </div>
              <div className={styles.toolbarGroup}>
                <Button variant="ghost" size="3" className={styles.ghostAction} onClick={() => setAll(true)}>
                  <ChevronsUpDown size={16} aria-hidden="true" />
                  {t('support.sysinfo.expandAll')}
                </Button>
                <Button variant="ghost" size="3" className={styles.ghostAction} onClick={() => setAll(false)}>
                  <ChevronsDownUp size={16} aria-hidden="true" />
                  {t('support.sysinfo.collapseAll')}
                </Button>
              </div>
            </div>
            {!report.isPending && sections.length === 0 && (
              <p className={styles.muted}>{t('support.sysinfo.empty')}</p>
            )}
            {sections.map(([key, value], index) => {
              const rows = flatten(value);
              const expanded = isOpen(key, index);
              return (
                <section key={key} id={`${ids}-${key}`} className={styles.infoSection}>
                  <button
                    type="button"
                    className={styles.infoHeader}
                    aria-expanded={expanded}
                    onClick={() => setOpen((previous) => ({ ...previous, [key]: !expanded }))}
                  >
                    {expanded ? (
                      <ChevronDown size={16} aria-hidden="true" />
                    ) : (
                      <ChevronRight size={16} aria-hidden="true" />
                    )}
                    <span className={styles.infoTitle}>{sectionTitle(key)}</span>
                    <span className={styles.infoCount}>
                      {t('support.sysinfo.count', { count: rows.length })}
                    </span>
                  </button>
                  {expanded && (
                    <dl className={styles.infoRows}>
                      {rows.map(([name, shown]) => (
                        <div key={name} className={styles.infoRow}>
                          <dt>{name}</dt>
                          <dd>{shown}</dd>
                        </div>
                      ))}
                    </dl>
                  )}
                </section>
              );
            })}
          </>
        )}
        <HelpCard title={t('support.sysinfo.helpTitle')} link={DOCS} linkLabel={t('support.common.docs')}>
          {t('support.sysinfo.helpText')}
        </HelpCard>
      </div>
    </Page>
  );
}
