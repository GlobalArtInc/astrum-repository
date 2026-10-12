import { Flex, Skeleton, Text, TextField } from '@radix-ui/themes';
import { Search, ShieldOff } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';
import type { RepositoryType } from '../../../api';
import { useSession } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { EmptyState, ErrorState, FormatBadge, formatName, Page, TypeBadge } from '../../../ui';
import { createAddress, LIST_ADDRESS, type Recipe, useRecipes } from '../api';
import { TYPES } from '../model/spec';
import styles from '../repositories.module.css';

const DESCRIPTIONS: Record<RepositoryType, string> = {
  hosted: 'repositories.types.hosted',
  proxy: 'repositories.types.proxy',
  group: 'repositories.types.group',
};

const CHOOSE: Record<RepositoryType, string> = {
  hosted: 'repositories.create.chooseHosted',
  proxy: 'repositories.create.chooseProxy',
  group: 'repositories.create.chooseGroup',
};

export interface FormatChoice {
  format: string;
  types: RepositoryType[];
}

export function groupRecipes(
  recipes: readonly Recipe[],
  allowed: (format: string) => boolean,
  filter: string,
): FormatChoice[] {
  const wanted = filter.trim().toLowerCase();
  const byFormat = new Map<string, RepositoryType[]>();
  for (const recipe of recipes) {
    if (!allowed(recipe.format)) {
      continue;
    }
    const name = formatName(recipe.format).toLowerCase();
    if (wanted !== '' && !name.includes(wanted) && !recipe.format.includes(wanted)) {
      continue;
    }
    const types = byFormat.get(recipe.format) ?? [];
    if (!types.includes(recipe.type)) {
      types.push(recipe.type);
    }
    byFormat.set(recipe.format, types);
  }
  return [...byFormat.entries()]
    .map(([format, types]) => ({
      format,
      types: TYPES.filter((type) => types.includes(type)),
    }))
    .sort((left, right) => formatName(left.format).localeCompare(formatName(right.format)));
}

export function createLabel(t: Translate, format: string, type: RepositoryType): string {
  return t(CHOOSE[type], { format: formatName(format) });
}

export default function CreatePickerPage() {
  const t = useT();
  const { can } = useSession();
  const recipes = useRecipes();
  const [filter, setFilter] = useState('');
  const allowed = (format: string) => can({ some: `astrum:repository-admin:${format}:*:add` });
  const choices = recipes.data ? groupRecipes(recipes.data, allowed, filter) : [];
  const anyAllowed = recipes.data ? groupRecipes(recipes.data, allowed, '').length > 0 : true;

  return (
    <Page
      title={t('repositories.create.title')}
      description={t('repositories.create.description')}
      breadcrumbs={[
        { label: t('repositories.crumbs.settings'), to: '/settings' },
        { label: t('repositories.crumbs.repositories'), to: LIST_ADDRESS },
        { label: t('repositories.crumbs.create') },
      ]}
    >
      {anyAllowed && (
        <TextField.Root
          size="3"
          type="search"
          value={filter}
          placeholder={t('repositories.create.filter')}
          aria-label={t('repositories.create.filter')}
          onChange={(event) => setFilter(event.target.value)}
        >
          <TextField.Slot>
            <Search size={16} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
      )}
      {recipes.isPending ? (
        <ul className={styles.formats}>
          {[0, 1, 2, 3, 4].map((index) => (
            <li key={index}>
              <Skeleton height="180px" />
            </li>
          ))}
        </ul>
      ) : recipes.error ? (
        <ErrorState error={recipes.error} onRetry={() => void recipes.refetch()} />
      ) : !anyAllowed ? (
        <EmptyState
          icon={ShieldOff}
          title={t('repositories.create.none')}
          description={t('repositories.create.noneText')}
        />
      ) : choices.length === 0 ? (
        <EmptyState icon={Search} title={t('repositories.create.noMatches')} />
      ) : (
        <ul className={styles.formats}>
          {choices.map((choice) => (
            <li key={choice.format} className={styles.formatCard}>
              <Flex align="center" gap="2">
                <FormatBadge format={choice.format} />
                <Text size="4" weight="medium">
                  {formatName(choice.format)}
                </Text>
              </Flex>
              <ul className={styles.recipes}>
                {choice.types.map((type) => (
                  <li key={type}>
                    <Link
                      to={createAddress(choice.format, type)}
                      className={styles.recipe}
                      aria-label={createLabel(t, choice.format, type)}
                    >
                      <span>
                        <TypeBadge type={type} />
                      </span>
                      <Text size="2" color="gray">
                        {t(DESCRIPTIONS[type])}
                      </Text>
                    </Link>
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      )}
    </Page>
  );
}
