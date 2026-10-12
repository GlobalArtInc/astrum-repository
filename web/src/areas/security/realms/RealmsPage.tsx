import { Badge, Button, Callout, Flex, Heading, IconButton, Spinner, Text } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { ArrowDown, ArrowUp, GripVertical, Info, Plus, X } from 'lucide-react';
import { type DragEvent, useState } from 'react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../../ui';
import { REALMS_ACTIVE, REALMS_AVAILABLE, type Realm } from '../api';
import formStyles from '../shared/Form.module.css';
import { refusedBy } from '../shared/messages';
import {
  activate,
  activeRealms,
  deactivate,
  inactiveRealms,
  LOCAL_REALM,
  moveBy,
  moveTo,
  realmLabel,
  sameOrder,
} from './model';
import styles from './Realms.module.css';

const DRAG_TYPE = 'application/x-astrum-realm';

function RealmsEditor({ available, saved }: { available: Realm[]; saved: string[] }) {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const canWrite = useCan('astrum:settings:update');
  const [active, setActive] = useState(saved);
  const [baseline, setBaseline] = useState(saved);
  const [dragging, setDragging] = useState<string | null>(null);
  const [over, setOver] = useState<number | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmLocal, setConfirmLocal] = useState(false);
  const dirty = !sameOrder(active, baseline);
  const shownActive = activeRealms(available, active);
  const shownInactive = inactiveRealms(available, active);

  const change = (next: string[]) => {
    setActive(next);
    setError(null);
  };

  const store = async () => {
    setSaving(true);
    setError(null);
    try {
      await api.put(REALMS_ACTIVE, active);
      setBaseline(active);
      toast.success(t('security.realms.saved'));
      await Promise.all([invalidate(REALMS_ACTIVE), client.invalidateQueries({ queryKey: STATE_KEY })]);
    } catch (failure) {
      const explained = refusedBy(t, failure);
      setError(explained instanceof Error ? explained.message : String(explained));
    } finally {
      setSaving(false);
    }
  };

  const save = () => {
    if (active.length === 0) {
      setError(t('security.realms.noneActive'));
      return;
    }
    if (!active.includes(LOCAL_REALM) && baseline.includes(LOCAL_REALM)) {
      setConfirmLocal(true);
      return;
    }
    void store();
  };

  const dragStart = (event: DragEvent, id: string) => {
    event.dataTransfer.setData(DRAG_TYPE, id);
    event.dataTransfer.setData('text/plain', id);
    event.dataTransfer.effectAllowed = 'move';
    setDragging(id);
  };

  const dragEnd = () => {
    setDragging(null);
    setOver(null);
  };

  const droppedId = (event: DragEvent) =>
    event.dataTransfer.getData(DRAG_TYPE) || event.dataTransfer.getData('text/plain') || dragging;

  const dropOnActive = (event: DragEvent, index: number) => {
    event.preventDefault();
    const id = droppedId(event);
    if (id) {
      const current = active.indexOf(id);
      change(
        current >= 0 ? moveTo(active, current, index) : moveTo(activate(active, id), active.length, index),
      );
    }
    dragEnd();
  };

  const dropOnAvailable = (event: DragEvent) => {
    event.preventDefault();
    const id = droppedId(event);
    if (id) {
      change(deactivate(active, id));
    }
    dragEnd();
  };

  return (
    <Flex direction="column" gap="4">
      <div className={styles.columns}>
        <section className={styles.panel} aria-labelledby="realms-active">
          <header className={styles.head}>
            <Heading as="h2" size="3" id="realms-active">
              {t('security.realms.active')}
            </Heading>
            <Badge variant="soft" color="gray">
              {t('security.realms.count', { count: active.length })}
            </Badge>
          </header>
          <ol
            className={styles.list}
            aria-labelledby="realms-active"
            onDragOver={(event) => {
              if (canWrite) {
                event.preventDefault();
              }
            }}
            onDrop={(event) => canWrite && dropOnActive(event, active.length)}
          >
            {shownActive.length === 0 && (
              <li className={styles.empty}>
                <Text size="2" color="gray">
                  {t('security.realms.emptyActive')}
                </Text>
              </li>
            )}
            {shownActive.map((realm, index) => {
              const label = realmLabel(t, realm);
              return (
                <li
                  key={realm.id}
                  className={styles.item}
                  data-dragging={dragging === realm.id || undefined}
                  data-over={over === index || undefined}
                  draggable={canWrite}
                  onDragStart={(event) => dragStart(event, realm.id)}
                  onDragEnd={dragEnd}
                  onDragOver={(event) => {
                    if (canWrite) {
                      event.preventDefault();
                      event.stopPropagation();
                      setOver(index);
                    }
                  }}
                  onDrop={(event) => {
                    event.stopPropagation();
                    if (canWrite) {
                      dropOnActive(event, index);
                    }
                  }}
                >
                  {canWrite && <GripVertical size={16} aria-hidden="true" className={styles.grip} />}
                  <span className={styles.position} aria-hidden="true">
                    {index + 1}
                  </span>
                  <span className={styles.name}>
                    <Text size="2" weight="medium">
                      {label}
                    </Text>
                    <Text size="1" color="gray" className={formStyles.mono}>
                      {realm.id}
                    </Text>
                  </span>
                  {canWrite && (
                    <span className={styles.buttons}>
                      <IconButton
                        type="button"
                        size="1"
                        variant="ghost"
                        color="gray"
                        disabled={index === 0}
                        aria-label={t('security.realms.up', { name: label })}
                        onClick={() => change(moveBy(active, realm.id, -1))}
                      >
                        <ArrowUp size={14} aria-hidden="true" />
                      </IconButton>
                      <IconButton
                        type="button"
                        size="1"
                        variant="ghost"
                        color="gray"
                        disabled={index === active.length - 1}
                        aria-label={t('security.realms.down', { name: label })}
                        onClick={() => change(moveBy(active, realm.id, 1))}
                      >
                        <ArrowDown size={14} aria-hidden="true" />
                      </IconButton>
                      <IconButton
                        type="button"
                        size="1"
                        variant="ghost"
                        color="red"
                        aria-label={t('security.realms.deactivate', { name: label })}
                        onClick={() => change(deactivate(active, realm.id))}
                      >
                        <X size={14} aria-hidden="true" />
                      </IconButton>
                    </span>
                  )}
                </li>
              );
            })}
          </ol>
        </section>
        <section
          className={styles.panel}
          aria-labelledby="realms-available"
          onDragOver={(event) => {
            if (canWrite) {
              event.preventDefault();
            }
          }}
          onDrop={(event) => canWrite && dropOnAvailable(event)}
        >
          <header className={styles.head}>
            <Heading as="h2" size="3" id="realms-available">
              {t('security.realms.available')}
            </Heading>
            <Badge variant="soft" color="gray">
              {t('security.realms.count', { count: shownInactive.length })}
            </Badge>
          </header>
          <ul className={styles.list} aria-labelledby="realms-available">
            {shownInactive.length === 0 && (
              <li className={styles.empty}>
                <Text size="2" color="gray">
                  {t('security.realms.emptyAvailable')}
                </Text>
              </li>
            )}
            {shownInactive.map((realm) => {
              const label = realmLabel(t, realm);
              return (
                <li
                  key={realm.id}
                  className={styles.item}
                  draggable={canWrite}
                  onDragStart={(event) => dragStart(event, realm.id)}
                  onDragEnd={dragEnd}
                >
                  <span className={styles.name}>
                    <Text size="2" weight="medium">
                      {label}
                    </Text>
                    <Text size="1" color="gray" className={formStyles.mono}>
                      {realm.id}
                    </Text>
                  </span>
                  {canWrite && (
                    <span className={styles.buttons}>
                      <IconButton
                        type="button"
                        size="1"
                        variant="soft"
                        aria-label={t('security.realms.activate', { name: label })}
                        onClick={() => change(activate(active, realm.id))}
                      >
                        <Plus size={14} aria-hidden="true" />
                      </IconButton>
                    </span>
                  )}
                </li>
              );
            })}
          </ul>
        </section>
      </div>
      <Callout.Root color="gray" variant="surface" className={styles.about}>
        <Callout.Icon>
          <Info size={16} />
        </Callout.Icon>
        <Callout.Text>{t('security.realms.about')}</Callout.Text>
      </Callout.Root>
      <FormError message={error} />
      {canWrite && (
        <div className={formStyles.actions}>
          <Button type="button" loading={saving} disabled={!dirty} onClick={save}>
            {t('security.form.save')}
          </Button>
          <Button
            type="button"
            variant="soft"
            color="gray"
            disabled={!dirty}
            onClick={() => change(baseline)}
          >
            {t('security.form.discard')}
          </Button>
        </div>
      )}
      <UnsavedChangesGuard when={dirty} />
      <ConfirmDialog
        open={confirmLocal}
        onOpenChange={setConfirmLocal}
        onConfirm={store}
        title={t('security.realms.localTitle')}
        description={t('security.realms.localText')}
        confirmLabel={t('security.realms.localConfirm')}
        danger
      />
    </Flex>
  );
}

export default function RealmsPage() {
  const t = useT();
  const available = useApi<Realm[]>(REALMS_AVAILABLE);
  const active = useApi<string[]>(REALMS_ACTIVE);
  const error = available.error ?? active.error;
  return (
    <Page
      title={t('security.realms.title')}
      description={t('security.realms.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.realms.title') },
      ]}
    >
      {available.isPending || active.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : error || !available.data || !active.data ? (
        <ErrorState error={error} onRetry={() => void Promise.all([available.refetch(), active.refetch()])} />
      ) : (
        <RealmsEditor available={available.data} saved={active.data} />
      )}
    </Page>
  );
}
