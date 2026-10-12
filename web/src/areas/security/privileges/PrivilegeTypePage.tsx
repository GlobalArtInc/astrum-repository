import { Flex, Text } from '@radix-ui/themes';
import { ChevronRight } from 'lucide-react';
import { Link } from 'react-router';
import { useT } from '../../../i18n';
import { Page } from '../../../ui';
import { PRIVILEGE_TYPE_IDS } from '../api';
import { typeDescription, typeLabel } from './labels';
import styles from './Privileges.module.css';

export default function PrivilegeTypePage() {
  const t = useT();
  return (
    <Page
      title={t('security.privilege.createTitle')}
      description={t('security.privilege.chooseType')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.privileges.title'), to: '/admin/security/privileges' },
        { label: t('security.privilege.create') },
      ]}
    >
      <ul className={styles.types}>
        {PRIVILEGE_TYPE_IDS.map((id) => (
          <li key={id}>
            <Link to={`/admin/security/privileges/create/${id}`} className={styles.type}>
              <Flex direction="column" gap="1" style={{ minWidth: 0 }}>
                <Text size="3" weight="medium">
                  {typeLabel(t, id)}
                </Text>
                <Text size="2" color="gray">
                  {typeDescription(t, id)}
                </Text>
              </Flex>
              <ChevronRight size={18} aria-hidden="true" className={styles.chevron} />
            </Link>
          </li>
        ))}
      </ul>
    </Page>
  );
}
