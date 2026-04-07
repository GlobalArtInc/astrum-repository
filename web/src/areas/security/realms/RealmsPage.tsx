import { Button, Flex, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import {
  ArrowDown,
  ArrowUp,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  GripVertical,
  Search,
} from 'lucide-react';
import { type DragEvent, type FormEvent, type ReactNode, useState } from 'react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../../ui';
import { REALMS_ACTIVE, REALMS_AVAILABLE, type Realm } from '../api';
import { HelpCard } from '../shared/AdminList';
import { EditBar, EditForm } from '../shared/Edit';
import edit from '../shared/Edit.module.css';
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
const HELP = 'https://help.globalart.dev/en/realms.html';

type Side = 'available' | 'active';

interface Selection {
  side: Side;
  id: string;
}

function MoveButton({
  label,
  title,
  disabled,
  onClick,
  children,
}: {
  label: string;
  title: string;
  disabled: boolean;
  onClick(): void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      className={styles.move}
      aria-label={label}
      title={title}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

function RealmPane({
  side,
  title,
  label,
  count,
  realms,
  selected,
  readOnly,
  onSelect,
  onMove,
  drag,
}: {
  side: Side;
  title: string;
  label: string;
  count: string;
  realms: Realm[];
  selected: Selection | null;
  readOnly: boolean;
  onSelect(id: string): void;
  onMove(id: string): void;
  drag: {
    start(event: DragEvent, id: string): void;
    end(): void;
    over: number | null;
    dragging: string | null;
    hover(index: number | null): void;
    drop(event: DragEvent, index: number | null): void;
  };
}) {
  const t = useT();
  const [filter, setFilter] = useState('');
  const words = filter.trim().toLowerCase();
  const ordered = side === 'active';
  const shown = realms
    .map((realm, index) => ({ realm, index }))
    .filter(({ realm }) => !words || realmLabel(realm).toLowerCase().includes(words));
  return (
    <div className={ordered ? `${styles.pane} ${styles.activePane}` : styles.pane}>
      <div className={styles.head}>
        <span className={styles.title}>{title}</span>
        <span className={styles.count}>{count}</span>
      </div>
      <div className={styles.filter}>
        <Search size={14} aria-hidden="true" className={styles.searchIcon} />
        <input
          type="text"
          className={styles.filterInput}
          value={filter}
          placeholder={t('security.transfer.filter')}
          aria-label={t('security.transfer.filterOf', { list: label })}
          onChange={(event) => setFilter(event.target.value)}
        />
      </div>
      <div
        className={styles.list}
        role="listbox"
        tabIndex={-1}
        aria-label={label}
        onDragOver={(event) => {
          if (!readOnly) {
            event.preventDefault();
          }
        }}
        onDrop={(event) => !readOnly && drag.drop(event, null)}
      >
        {shown.map(({ realm, index }) => {
          const on = selected?.side === side && selected.id === realm.id;
          const classes = [styles.option];
          if (on) {
            classes.push(styles.selected);
          }
          if (ordered && drag.over === index) {
            classes.push(styles.over);
          }
          return (
            <div
              key={realm.id}
              role="option"
              tabIndex={0}
              aria-selected={on}
              className={classes.join(' ')}
              data-ordered={ordered || undefined}
              data-dragging={drag.dragging === realm.id || undefined}
              draggable={!readOnly}
              onClick={() => !readOnly && onSelect(realm.id)}
              onDoubleClick={() => !readOnly && onMove(realm.id)}
              onKeyDown={(event) => {
                if (readOnly) {
                  return;
                }
                if (event.key === ' ') {
                  event.preventDefault();
                  onSelect(realm.id);
                } else if (event.key === 'Enter') {
                  event.preventDefault();
                  onMove(realm.id);
                }
              }}
              onDragStart={(event) => drag.start(event, realm.id)}
              onDragEnd={drag.end}
              onDragOver={(event) => {
                if (!readOnly && ordered) {
                  event.preventDefault();
                  event.stopPropagation();
                  drag.hover(index);
                }
              }}
              onDrop={(event) => {
                if (!readOnly && ordered) {
                  event.stopPropagation();
                  drag.drop(event, index);
                }
              }}
            >
              {ordered && <GripVertical size={14} aria-hidden="true" className={styles.grip} />}
              <span>{realmLabel(realm)}</span>
              {ordered && <span className={styles.position}>{index + 1}</span>}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function RealmsEditor({ available, saved }: { available: Realm[]; saved: string[] }) {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const canWrite = useCan('astrum:settings:update');
  const readOnly = !canWrite;
  const [active, setActive] = useState(saved);
  const [baseline, setBaseline] = useState(saved);
  const [selected, setSelected] = useState<Selection | null>(null);
  const [dragging, setDragging] = useState<string | null>(null);
  const [over, setOver] = useState<number | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmLocal, setConfirmLocal] = useState(false);
  const dirty = !sameOrder(active, baseline);
  const shownActive = activeRealms(available, active);
  const shownInactive = inactiveRealms(available, active);
  const selectedIndex = selected?.side === 'active' ? active.indexOf(selected.id) : -1;

  const change = (next: string[]) => {
    setActive(next);
    setError(null);
  };

  const toggle = (side: Side, id: string) =>
    setSelected(selected?.side === side && selected.id === id ? null : { side, id });

  const move = (side: Side, id: string) => {
    change(side === 'available' ? activate(active, id) : deactivate(active, id));
    setSelected(null);
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

  const save = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!dirty || readOnly) {
      return;
    }
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

  const droppedId = (event: DragEvent) =>
    event.dataTransfer.getData(DRAG_TYPE) || event.dataTransfer.getData('text/plain') || dragging;

  const drag = (side: Side) => ({
    over: side === 'active' ? over : null,
    dragging,
    start(event: DragEvent, id: string) {
      event.dataTransfer.setData(DRAG_TYPE, id);
      event.dataTransfer.setData('text/plain', id);
      event.dataTransfer.effectAllowed = 'move';
      setDragging(id);
    },
    end() {
      setDragging(null);
      setOver(null);
    },
    hover(index: number | null) {
      setOver(index);
    },
    drop(event: DragEvent, index: number | null) {
      event.preventDefault();
      const id = droppedId(event);
      if (id) {
        if (side === 'available') {
          change(deactivate(active, id));
        } else {
          const target = index ?? active.length;
          const current = active.indexOf(id);
          change(
            current >= 0
              ? moveTo(active, current, target)
              : moveTo(activate(active, id), active.length, target),
          );
        }
      }
      setDragging(null);
      setOver(null);
    },
  });

  return (
    <EditForm
      id="realms-form"
      onSubmit={save}
      bar={
        <EditBar
          end={
            !readOnly && (
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    change(baseline);
                    setSelected(null);
                  }}
                >
                  {t('security.form.discard')}
                </Button>
                <Button type="submit" disabled={!dirty} loading={saving}>
                  {t('security.form.save')}
                </Button>
              </>
            )
          }
        />
      }
    >
      <h2 className={styles.heading}>{t('security.realms.activeRealms')}</h2>
      <FormError message={error} />
      <div className={styles.transfer}>
        <RealmPane
          side="available"
          title={t('security.realms.available')}
          label={t('security.realms.availableList')}
          count={t('security.realms.count', { count: shownInactive.length })}
          realms={shownInactive}
          selected={selected}
          readOnly={readOnly}
          onSelect={(id) => toggle('available', id)}
          onMove={(id) => move('available', id)}
          drag={drag('available')}
        />
        <div className={styles.moves}>
          <MoveButton
            label={t('security.realms.addAll')}
            title={t('security.realms.addAllShort')}
            disabled={readOnly || shownInactive.length === 0}
            onClick={() => {
              change([...active, ...shownInactive.map((realm) => realm.id)]);
              setSelected(null);
            }}
          >
            <ChevronsRight size={16} aria-hidden="true" />
          </MoveButton>
          <MoveButton
            label={t('security.realms.add')}
            title={t('security.realms.addShort')}
            disabled={readOnly || selected?.side !== 'available'}
            onClick={() => selected && move('available', selected.id)}
          >
            <ChevronRight size={16} aria-hidden="true" />
          </MoveButton>
          <MoveButton
            label={t('security.realms.remove')}
            title={t('security.realms.removeShort')}
            disabled={readOnly || selected?.side !== 'active'}
            onClick={() => selected && move('active', selected.id)}
          >
            <ChevronLeft size={16} aria-hidden="true" />
          </MoveButton>
          <MoveButton
            label={t('security.realms.removeAll')}
            title={t('security.realms.removeAllShort')}
            disabled={readOnly || active.length === 0}
            onClick={() => {
              change([]);
              setSelected(null);
            }}
          >
            <ChevronsLeft size={16} aria-hidden="true" />
          </MoveButton>
        </div>
        <RealmPane
          side="active"
          title={t('security.realms.active')}
          label={t('security.realms.activeList')}
          count={t('security.realms.count', { count: shownActive.length })}
          realms={shownActive}
          selected={selected}
          readOnly={readOnly}
          onSelect={(id) => toggle('active', id)}
          onMove={(id) => move('active', id)}
          drag={drag('active')}
        />
        <div className={styles.moves}>
          <MoveButton
            label={t('security.realms.up')}
            title={t('security.realms.upShort')}
            disabled={readOnly || selectedIndex <= 0}
            onClick={() => selected && change(moveBy(active, selected.id, -1))}
          >
            <ArrowUp size={16} aria-hidden="true" />
          </MoveButton>
          <MoveButton
            label={t('security.realms.down')}
            title={t('security.realms.downShort')}
            disabled={readOnly || selectedIndex < 0 || selectedIndex === active.length - 1}
            onClick={() => selected && change(moveBy(active, selected.id, 1))}
          >
            <ArrowDown size={16} aria-hidden="true" />
          </MoveButton>
        </div>
      </div>
      <HelpCard title={t('security.realms.helpTitle')} link={HELP}>
        {t('security.realms.about')}
      </HelpCard>
      <UnsavedChangesGuard when={dirty && !readOnly} />
      <ConfirmDialog
        open={confirmLocal}
        onOpenChange={setConfirmLocal}
        onConfirm={store}
        title={t('security.realms.localTitle')}
        description={t('security.realms.localText')}
        confirmLabel={t('security.realms.localConfirm')}
        danger
      />
    </EditForm>
  );
}

export default function RealmsPage() {
  const t = useT();
  const available = useApi<Realm[]>(REALMS_AVAILABLE);
  const active = useApi<string[]>(REALMS_ACTIVE);
  const error = available.error ?? active.error;
  return (
    <Page
      className={edit.settingsPage}
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
