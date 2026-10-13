import { Badge, Button, Card, Spinner, TextField } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { Puzzle, Search } from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router';
import { api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page, useToast } from '../../../ui';
import styles from '../Notify.module.css';
import { TYPE_ICONS } from './CapabilitiesPage';
import { CapabilityForm } from './CapabilityForm';
import { About } from './CapabilityPage';
import {
  CAPABILITIES,
  type Capability,
  type CapabilityType,
  type CapabilityValues,
  LIST_PAGE,
  TYPES,
  toCapabilityRequest,
} from './model';

export default function CreateCapabilityPage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const types = useApi<CapabilityType[]>(TYPES);
  const existing = useApi<Capability[]>(CAPABILITIES);
  const [selected, setSelected] = useState<CapabilityType | null>(null);
  const [chosen, setChosen] = useState<CapabilityType | null>(null);
  const [filter, setFilter] = useState('');
  const configured = new Set((existing.data ?? []).map((capability) => capability.type));
  const all = types.data ?? [];
  const available = all.filter((type) => !configured.has(type.id));
  const visible = [...available, ...all.filter((type) => configured.has(type.id))].filter((type) =>
    type.name.toLowerCase().includes(filter.trim().toLowerCase()),
  );
  const crumbs = [
    { label: t('notify.crumbs.settings'), to: '/settings' },
    { label: t('notify.capabilities.title'), to: LIST_PAGE },
    { label: t('notify.capabilities.createCrumb') },
  ];

  const create = async (values: CapabilityValues) => {
    if (!chosen) {
      return;
    }
    const created = await api.post<Capability>(CAPABILITIES, toCapabilityRequest(chosen, values));
    toast.success(t('notify.capabilities.created', { name: created.typeName }));
    await Promise.all([invalidate(CAPABILITIES), client.invalidateQueries({ queryKey: STATE_KEY })]);
    navigate(`${LIST_PAGE}/${encodeURIComponent(created.id)}`);
  };

  if (types.error || existing.error) {
    return (
      <Page title={t('notify.capabilities.createCrumb')} breadcrumbs={crumbs}>
        <ErrorState error={types.error ?? existing.error} onRetry={() => void types.refetch()} />
      </Page>
    );
  }
  if (chosen) {
    return (
      <Page title={chosen.name} breadcrumbs={crumbs}>
        <Card className={styles.aboutCard}>
          <About type={chosen} />
        </Card>
        <CapabilityForm
          type={chosen}
          capability={null}
          readOnly={false}
          submitLabel={t('notify.capabilities.createAction')}
          onSubmit={create}
          onCancel={() => setChosen(null)}
        />
      </Page>
    );
  }
  return (
    <Page
      title={t('notify.capabilities.createCrumb')}
      breadcrumbs={crumbs}
      actions={
        <div className={styles.headerActions}>
          <Button variant="soft" color="gray" onClick={() => navigate(LIST_PAGE)}>
            {t('notify.capabilities.cancel')}
          </Button>
          <Button disabled={!selected} onClick={() => setChosen(selected)}>
            {t('notify.capabilities.continue')}
          </Button>
        </div>
      }
    >
      <h2 className={styles.formTitle}>{t('notify.capabilities.selectType')}</h2>
      <p className={styles.muted}>
        {t('notify.capabilities.typesAvailable', { available: available.length, total: all.length })}
      </p>
      <TextField.Root
        className={styles.search}
        value={filter}
        placeholder={t('notify.capabilities.filterTypes')}
        aria-label={t('notify.capabilities.filterTypes')}
        onChange={(event) => setFilter(event.target.value)}
      >
        <TextField.Slot>
          <Search size={14} aria-hidden="true" />
        </TextField.Slot>
      </TextField.Root>
      {types.isPending ? (
        <Spinner />
      ) : (
        <div className={styles.typeList} role="radiogroup" aria-label={t('notify.capabilities.selectType')}>
          {visible.map((type) => {
            const Icon = TYPE_ICONS[type.id] ?? Puzzle;
            const taken = configured.has(type.id);
            return (
              <button
                key={type.id}
                type="button"
                role="radio"
                aria-checked={selected?.id === type.id}
                disabled={taken}
                className={styles.typeCard}
                data-active={selected?.id === type.id || undefined}
                onClick={() => setSelected(type)}
                onDoubleClick={() => setChosen(type)}
              >
                <span className={styles.typeCardTitle}>
                  <Icon size={16} aria-hidden="true" />
                  {type.name}
                  {taken && <Badge color="gray">{t('notify.capabilities.alreadyConfigured')}</Badge>}
                </span>
                <span className={styles.typeCardText}>{type.about.replace(/<[^>]*>/g, ' ').trim()}</span>
              </button>
            );
          })}
        </div>
      )}
    </Page>
  );
}
