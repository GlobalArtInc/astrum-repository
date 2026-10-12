import { Text } from '@radix-ui/themes';
import { Cloud, Folder, Layers, type LucideIcon } from 'lucide-react';
import { Link } from 'react-router';
import { useT } from '../../../i18n';
import { Page } from '../../../ui';
import { createAddress, LIST_ADDRESS, STORE_TYPES, type StoreType } from '../api';
import styles from '../blobstores.module.css';
import { typeLabel } from '../components/bits';

const DESCRIPTIONS: Record<StoreType, string> = {
  file: 'blobstores.types.fileText',
  s3: 'blobstores.types.s3Text',
  group: 'blobstores.types.groupText',
};

const ICONS: Record<StoreType, LucideIcon> = { file: Folder, s3: Cloud, group: Layers };

export default function CreatePickerPage() {
  const t = useT();
  return (
    <Page
      title={t('blobstores.create.title')}
      description={t('blobstores.create.description')}
      breadcrumbs={[
        { label: t('blobstores.crumbs.settings'), to: '/settings' },
        { label: t('blobstores.crumbs.list'), to: LIST_ADDRESS },
        { label: t('blobstores.crumbs.create') },
      ]}
    >
      <ul className={styles.types}>
        {STORE_TYPES.map((type) => {
          const Icon = ICONS[type];
          return (
            <li key={type}>
              <Link to={createAddress(type)} className={styles.typeCard}>
                <span className={styles.typeIcon} aria-hidden="true">
                  <Icon size={18} />
                </span>
                <Text size="3" weight="medium">
                  {typeLabel(t, type)}
                </Text>
                <Text size="2" color="gray">
                  {t(DESCRIPTIONS[type])}
                </Text>
              </Link>
            </li>
          );
        })}
      </ul>
    </Page>
  );
}
