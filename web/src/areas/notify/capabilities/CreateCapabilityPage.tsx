import { Badge, Button, Flex, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, ChevronRight, Puzzle, Search } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { ApiError, api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  Breadcrumbs,
  describeError,
  ErrorState,
  FormError,
  UnsavedChangesGuard,
  useToast,
} from '../../../ui';
import { useDocumentTitle } from '../../../ui/hooks';
import { SettingsForm, SettingsSection } from '../../../ui/SettingsForm';
import styles from './Capabilities.module.css';
import { CapabilityFields } from './CapabilityFields';
import { About } from './CapabilityPage';
import {
  aboutParagraphs,
  CAPABILITIES,
  type Capability,
  type CapabilityType,
  type CapabilityValues,
  initialCapabilityValues,
  LIST_PAGE,
  serverProblems,
  TYPE_ICONS,
  TYPES,
  toCapabilityRequest,
  validateCapability,
} from './model';

function TypePicker({
  types,
  configured,
  selected,
  onSelect,
  onChoose,
}: {
  types: CapabilityType[];
  configured: Set<string>;
  selected: string | null;
  onSelect(type: CapabilityType): void;
  onChoose(type: CapabilityType): void;
}) {
  const t = useT();
  const [filter, setFilter] = useState('');
  const words = filter.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const visible = [...types]
    .sort(
      (left, right) =>
        Number(configured.has(left.id)) - Number(configured.has(right.id)) ||
        left.name.localeCompare(right.name),
    )
    .filter((type) => {
      const text = `${type.name} ${type.about}`.toLowerCase();
      return words.every((word) => text.includes(word));
    });
  const available = types.filter((type) => !configured.has(type.id)).length;
  return (
    <>
      <div className={styles.typeFilter}>
        <div className={styles.search}>
          <Search size={16} aria-hidden="true" />
          <input
            type="search"
            value={filter}
            placeholder={t('notify.capabilities.filterTypes')}
            aria-label={t('notify.capabilities.filterTypes')}
            onChange={(event) => setFilter(event.target.value)}
          />
        </div>
        <span className={styles.shown}>
          {t('notify.capabilities.typesAvailable', { available, total: types.length })}
        </span>
      </div>
      <div className={styles.typeList} role="radiogroup" aria-label={t('notify.capabilities.selectType')}>
        {visible.map((type) => {
          const Icon = TYPE_ICONS[type.id] ?? Puzzle;
          const taken = configured.has(type.id);
          return (
            <button
              key={type.id}
              type="button"
              role="radio"
              aria-checked={selected === type.id}
              disabled={taken}
              className={styles.typeCard}
              data-active={selected === type.id || undefined}
              onClick={() => onSelect(type)}
              onDoubleClick={() => onChoose(type)}
            >
              <span className={styles.typeCardIcon}>
                <Icon size={20} aria-hidden="true" />
              </span>
              <span className={styles.typeCardContent}>
                <span className={styles.typeCardName}>
                  {type.name}
                  {taken && <Badge color="gray">{t('notify.capabilities.alreadyConfigured')}</Badge>}
                </span>
                <span className={styles.typeCardAbout}>
                  {aboutParagraphs(type.about).map((paragraph) => (
                    <span key={paragraph}>{paragraph}</span>
                  ))}
                </span>
              </span>
              {!taken && <ChevronRight size={16} aria-hidden="true" className={styles.typeCardArrow} />}
            </button>
          );
        })}
      </div>
    </>
  );
}

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
  const [values, setValues] = useState<CapabilityValues | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [leaving, setLeaving] = useState<string | null>(null);
  useDocumentTitle(t('notify.capabilities.createCrumb'));
  useEffect(() => {
    if (leaving !== null) {
      navigate(leaving);
    }
  }, [leaving, navigate]);
  const configured = new Set((existing.data ?? []).map((capability) => capability.type));

  const choose = (type: CapabilityType) => {
    setSelected(type);
    if (chosen?.id !== type.id) {
      setValues(initialCapabilityValues(type, null));
      setErrors({});
      setFormError(null);
    }
    setChosen(type);
  };

  const create = async () => {
    if (!chosen || !values) {
      return;
    }
    const problems = validateCapability(t, chosen, values);
    setErrors(problems);
    if (Object.values(problems).some(Boolean)) {
      return;
    }
    setBusy(true);
    setFormError(null);
    try {
      const created = await api.post<Capability>(CAPABILITIES, toCapabilityRequest(chosen, values));
      toast.success(t('notify.capabilities.created', { name: created.typeName }));
      await Promise.all([invalidate(CAPABILITIES), client.invalidateQueries({ queryKey: STATE_KEY })]);
      setLeaving(`${LIST_PAGE}/${encodeURIComponent(created.id)}`);
    } catch (error) {
      if (error instanceof ApiError && error.status === 400 && error.fieldErrors.length > 0) {
        const { fields, other } = serverProblems(chosen, error.fieldErrors);
        setErrors(fields);
        setFormError(other.length > 0 ? other.join(' ') : null);
      } else {
        setFormError(describeError(t, error));
      }
    } finally {
      setBusy(false);
    }
  };

  const step = chosen ? 2 : 1;
  const titles = [t('notify.capabilities.selectType'), t('notify.capabilities.configure')];
  const failure = types.error ?? existing.error;

  return (
    <div className={styles.createPage}>
      <Breadcrumbs
        items={[
          { label: t('notify.crumbs.settings'), to: '/settings' },
          { label: t('notify.capabilities.title'), to: LIST_PAGE },
          { label: t('notify.capabilities.createCrumb') },
        ]}
      />
      <SettingsForm
        wide
        label={t('notify.capabilities.createCrumb')}
        onSubmit={() => (chosen ? void create() : selected && choose(selected))}
        progress={{ step, steps: 2, label: titles[step - 1] }}
        start={
          <Button asChild variant="outline">
            <Link to={LIST_PAGE}>{t('notify.capabilities.cancel')}</Link>
          </Button>
        }
        end={
          <>
            {chosen && (
              <Button type="button" variant="soft" disabled={busy} onClick={() => setChosen(null)}>
                <ArrowLeft size={16} aria-hidden="true" />
                {t('notify.capabilities.back')}
              </Button>
            )}
            <Button type="submit" disabled={!selected} loading={busy}>
              {chosen ? t('notify.capabilities.createAction') : t('notify.capabilities.continue')}
            </Button>
          </>
        }
      >
        <UnsavedChangesGuard when={chosen !== null && leaving === null} />
        <h2 className={styles.stepTitle}>{titles[step - 1]}</h2>
        {failure ? (
          <ErrorState error={failure} onRetry={() => void types.refetch()} />
        ) : !types.data || !existing.data ? (
          <Flex justify="center" py="8">
            <Spinner size="3" />
          </Flex>
        ) : chosen && values ? (
          <>
            <CapabilityFields
              lead={
                <SettingsSection title={t('notify.capabilities.capabilityType')}>
                  <div className={styles.chosenType}>
                    <span className={styles.chosenTypeName}>{chosen.name}</span>
                    <About type={chosen} />
                  </div>
                </SettingsSection>
              }
              type={chosen}
              values={values}
              errors={errors}
              readOnly={false}
              onChange={setValues}
            />
            <FormError message={formError} />
          </>
        ) : (
          <TypePicker
            types={types.data}
            configured={configured}
            selected={selected?.id ?? null}
            onSelect={setSelected}
            onChoose={choose}
          />
        )}
      </SettingsForm>
    </div>
  );
}
