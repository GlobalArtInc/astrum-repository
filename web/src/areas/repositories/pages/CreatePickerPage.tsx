import { Skeleton, TextField } from '@radix-ui/themes';
import { Search, ShieldOff } from 'lucide-react';
import { useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import type { RepositoryType } from '../../../api';
import { useSession } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { EmptyState, ErrorState, formatName, Page } from '../../../ui';
import { createAddress, LIST_ADDRESS, type Recipe, useRecipes } from '../api';
import { Logo, StepTitle, WizardBar } from '../components/Wizard';
import styles from '../components/Wizard.module.css';
import { TYPES } from '../model/spec';

const TYPE_TEXTS: Record<RepositoryType, string> = {
  hosted: 'repositories.types.hosted',
  proxy: 'repositories.types.proxy',
  group: 'repositories.types.group',
};

const TYPE_NAMES: Record<RepositoryType, string> = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
};

const CHOOSE: Record<RepositoryType, string> = {
  hosted: 'repositories.create.chooseHosted',
  proxy: 'repositories.create.chooseProxy',
  group: 'repositories.create.chooseGroup',
};

const FORMAT_TEXTS: Record<string, string> = {
  ansible: 'repositories.formats.ansible',
  apt: 'repositories.formats.apt',
  chef: 'repositories.formats.chef',
  cocoapods: 'repositories.formats.cocoapods',
  composer: 'repositories.formats.composer',
  conan: 'repositories.formats.conan',
  conda: 'repositories.formats.conda',
  docker: 'repositories.formats.docker',
  gitlfs: 'repositories.formats.gitlfs',
  go: 'repositories.formats.go',
  helm: 'repositories.formats.helm',
  hex: 'repositories.formats.hex',
  maven2: 'repositories.formats.maven2',
  nix: 'repositories.formats.nix',
  npm: 'repositories.formats.npm',
  nuget: 'repositories.formats.nuget',
  openvsx: 'repositories.formats.openvsx',
  pub: 'repositories.formats.pub',
  puppet: 'repositories.formats.puppet',
  pypi: 'repositories.formats.pypi',
  r: 'repositories.formats.r',
  raw: 'repositories.formats.raw',
  rubygems: 'repositories.formats.rubygems',
  vagrant: 'repositories.formats.vagrant',
  yum: 'repositories.formats.yum',
};

const TYPE_ORDER: RepositoryType[] = ['proxy', 'hosted', 'group'];

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

export function formatText(t: Translate, format: string): string {
  const key = FORMAT_TEXTS[format];
  return key ? t(key) : t('repositories.formats.other', { format });
}

export default function CreatePickerPage() {
  const t = useT();
  const navigate = useNavigate();
  const { can } = useSession();
  const recipes = useRecipes();
  const [params, setParams] = useSearchParams();
  const [filter, setFilter] = useState('');
  const allowed = (format: string) => can({ some: `astrum:repository-admin:${format}:*:add` });
  const all = recipes.data ? groupRecipes(recipes.data, allowed, '') : [];
  const choices = recipes.data ? groupRecipes(recipes.data, allowed, filter) : [];
  const chosen = all.find((choice) => choice.format === params.get('format'));
  const choose = (format: string | null) =>
    setParams(format ? new URLSearchParams({ format }) : new URLSearchParams());

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
      <WizardBar
        step={chosen ? 2 : 1}
        onCancel={() => navigate(LIST_ADDRESS)}
        onBack={chosen ? () => choose(null) : undefined}
      />
      <div className={styles.body}>
        {recipes.isPending ? (
          <Skeleton height="200px" />
        ) : recipes.error ? (
          <ErrorState error={recipes.error} onRetry={() => void recipes.refetch()} />
        ) : all.length === 0 ? (
          <EmptyState
            icon={ShieldOff}
            title={t('repositories.create.none')}
            description={t('repositories.create.noneText')}
          />
        ) : chosen ? (
          <>
            <StepTitle title={t('repositories.create.chooseType')} count={chosen.types.length} />
            <div className={styles.format}>
              <span className={styles.formatLogo}>
                <Logo format={chosen.format} size={36} />
              </span>
              <span>
                <p className={styles.formatName}>{formatName(chosen.format)}</p>
                <p className={styles.formatText}>{formatText(t, chosen.format)}</p>
              </span>
            </div>
            <ul className={styles.types}>
              {TYPE_ORDER.filter((type) => chosen.types.includes(type)).map((type) => (
                <li key={type}>
                  <button
                    type="button"
                    className={styles.type}
                    aria-label={createLabel(t, chosen.format, type)}
                    onClick={() => navigate(createAddress(chosen.format, type))}
                  >
                    <span className={styles.typeTitle}>{t(TYPE_NAMES[type])}</span>
                    <span className={styles.typeText}>{t(TYPE_TEXTS[type])}</span>
                  </button>
                </li>
              ))}
            </ul>
          </>
        ) : (
          <>
            <StepTitle title={t('repositories.create.chooseFormat')} count={all.length} />
            <TextField.Root
              className={styles.search}
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
            {choices.length === 0 ? (
              <EmptyState icon={Search} title={t('repositories.create.noMatches')} />
            ) : (
              <ul className={styles.tiles}>
                {choices.map((choice) => (
                  <li key={choice.format}>
                    <button type="button" className={styles.tile} onClick={() => choose(choice.format)}>
                      <span className={styles.logoBox}>
                        <Logo format={choice.format} />
                      </span>
                      {formatName(choice.format)}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </>
        )}
      </div>
    </Page>
  );
}
