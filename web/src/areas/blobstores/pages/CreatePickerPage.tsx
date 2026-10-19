import { Button } from '@radix-ui/themes';
import { ChevronRight, Cloud, HardDrive, Layers, Network, PaintBucket } from 'lucide-react';
import type { ReactNode } from 'react';
import { useState } from 'react';
import { useNavigate } from 'react-router';
import { useT } from '../../../i18n';
import { Page } from '../../../ui';
import { WizardBar } from '../../repositories/components/Wizard';
import { createAddress, LIST_ADDRESS, type StoreType } from '../api';
import styles from '../blobstores.module.css';

interface Choice {
  id: string;
  type: StoreType | null;
  name: string;
  text: string;
  icon: ReactNode;
}

export function wizardCrumbs(t: ReturnType<typeof useT>) {
  return [
    { label: t('blobstores.crumbs.settings'), to: '/settings' },
    { label: t('blobstores.crumbs.list'), to: LIST_ADDRESS },
    { label: t('blobstores.crumbs.create') },
  ];
}

export default function CreatePickerPage() {
  const t = useT();
  const navigate = useNavigate();
  const [chosen, setChosen] = useState<StoreType | null>(null);
  const choices: Choice[] = [
    {
      id: 'file',
      type: 'file',
      name: t('blobstores.picker.fileName'),
      text: t('blobstores.picker.file'),
      icon: <HardDrive size={28} color="var(--accent-11)" />,
    },
    {
      id: 's3',
      type: 's3',
      name: t('blobstores.types.s3'),
      text: t('blobstores.picker.s3'),
      icon: <PaintBucket size={28} color="#f59e0b" />,
    },
    {
      id: 'remote',
      type: 'remote',
      name: t('blobstores.picker.remoteName'),
      text: t('blobstores.picker.remote'),
      icon: <Network size={28} color="var(--accent-11)" />,
    },
    {
      id: 'azure',
      type: null,
      name: 'Azure Blob',
      text: t('blobstores.picker.azure'),
      icon: <Cloud size={28} color="#0078d4" />,
    },
    {
      id: 'google',
      type: null,
      name: 'Google Cloud',
      text: t('blobstores.picker.google'),
      icon: <Cloud size={28} color="#34a853" />,
    },
    {
      id: 'group',
      type: 'group',
      name: t('blobstores.picker.groupName'),
      text: t('blobstores.picker.group'),
      icon: <Layers size={28} color="var(--accent-11)" />,
    },
  ];
  return (
    <Page
      className={styles.wizardPage}
      title={t('blobstores.create.title')}
      description={t('blobstores.create.description')}
      breadcrumbs={wizardCrumbs(t)}
    >
      <WizardBar
        step={1}
        steps={4}
        onCancel={() => navigate(LIST_ADDRESS)}
        next={
          <Button disabled={!chosen} onClick={() => chosen && navigate(createAddress(chosen))}>
            {t('repositories.create.next')}
          </Button>
        }
      />
      <div className={styles.body}>
        <h2 className={styles.stepTitle}>{t('blobstores.picker.title')}</h2>
        <div>
          <h3 className={styles.sectionTitle}>{t('blobstores.picker.subtitle')}</h3>
          <p className={styles.sectionText}>{t('blobstores.picker.text')}</p>
        </div>
        <ul className={styles.types} aria-label={t('blobstores.picker.label')}>
          {choices.map((choice) => (
            <li key={choice.id}>
              <button
                type="button"
                className={styles.type}
                data-selected={choice.type !== null && choice.type === chosen ? '' : undefined}
                aria-pressed={choice.type !== null && choice.type === chosen}
                aria-disabled={choice.type === null}
                title={choice.type === null ? t('blobstores.picker.unavailable') : undefined}
                onClick={() => choice.type && setChosen(choice.type)}
                onDoubleClick={() => choice.type && navigate(createAddress(choice.type))}
              >
                <span className={styles.typeIcon} aria-hidden="true">
                  {choice.icon}
                </span>
                <span className={styles.typeText}>
                  <span className={styles.typeName}>{choice.name}</span>
                  <span className={styles.typeDescription}>{choice.text}</span>
                </span>
                {choice.type !== null && choice.type === chosen && (
                  <ChevronRight size={16} aria-hidden="true" color="var(--accent-11)" />
                )}
              </button>
            </li>
          ))}
        </ul>
      </div>
    </Page>
  );
}
