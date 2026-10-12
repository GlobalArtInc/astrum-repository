import { Flex, Heading, Link as RadixLink, Text } from '@radix-ui/themes';
import { Link } from 'react-router';
import { useT } from '../../../i18n';
import { FormatBadge, TypeBadge } from '../../../ui';
import { editAddress, type UiRepository } from '../api';
import styles from '../repositories.module.css';
import { Status } from './bits';

function RepositoryItem({
  name,
  found,
  position,
}: {
  name: string;
  found?: UiRepository;
  position?: number;
}) {
  const t = useT();
  return (
    <li className={styles.listItem}>
      {position !== undefined && (
        <Text size="1" color="gray" aria-label={t('repositories.membership.position')}>
          {position}
        </Text>
      )}
      <span className={styles.listName}>
        {found?.permissions.admin ? (
          <RadixLink asChild weight="medium">
            <Link to={editAddress(found.name)}>{found.name}</Link>
          </RadixLink>
        ) : (
          <Text weight="medium">{name}</Text>
        )}
      </span>
      {found ? (
        <span className={styles.badges}>
          <FormatBadge format={found.format} />
          <TypeBadge type={found.type} />
          <Status online={found.online} />
        </span>
      ) : (
        <Text size="1" color="gray">
          {t('repositories.form.memberHidden')}
        </Text>
      )}
    </li>
  );
}

export function MembershipTab({
  repository,
  members,
  repositories,
}: {
  repository: UiRepository;
  members: readonly string[];
  repositories: readonly UiRepository[];
}) {
  const t = useT();
  const find = (name: string) => repositories.find((row) => row.name.toLowerCase() === name.toLowerCase());
  return (
    <Flex direction="column" gap="6" style={{ maxWidth: 880 }}>
      {repository.type === 'group' && (
        <section aria-labelledby="repository-members">
          <Flex direction="column" gap="1" mb="3">
            <Heading as="h2" size="4" id="repository-members">
              {t('repositories.membership.members')}
            </Heading>
            <Text size="2" color="gray">
              {t('repositories.membership.membersText')}
            </Text>
          </Flex>
          {members.length === 0 ? (
            <Text size="2" color="gray">
              {t('repositories.membership.noMembers')}
            </Text>
          ) : (
            <ol className={styles.list} aria-labelledby="repository-members">
              {members.map((name, index) => (
                <RepositoryItem key={name} name={name} found={find(name)} position={index + 1} />
              ))}
            </ol>
          )}
        </section>
      )}
      <section aria-labelledby="repository-groups">
        <Flex direction="column" gap="1" mb="3">
          <Heading as="h2" size="4" id="repository-groups">
            {t('repositories.membership.groups')}
          </Heading>
          <Text size="2" color="gray">
            {t('repositories.membership.groupsText')}
          </Text>
        </Flex>
        {repository.memberOf.length === 0 ? (
          <Text size="2" color="gray">
            {t('repositories.membership.noGroups')}
          </Text>
        ) : (
          <ul className={styles.list} aria-labelledby="repository-groups">
            {repository.memberOf.map((name) => (
              <RepositoryItem key={name} name={name} found={find(name)} />
            ))}
          </ul>
        )}
      </section>
    </Flex>
  );
}
