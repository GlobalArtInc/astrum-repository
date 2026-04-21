import { Checkbox, Dialog, IconButton } from '@radix-ui/themes';
import { ChevronDown, ChevronRight, Funnel, X } from 'lucide-react';
import { useId, useState } from 'react';
import { useT } from '../../../i18n';
import styles from './Capabilities.module.css';

export interface FilterOption {
  value: string;
  label: string;
  count: number;
}

export interface FilterGroup {
  id: string;
  title: string;
  options: FilterOption[];
  selected: string[];
  onChange(selected: string[]): void;
}

function Section({ group }: { group: FilterGroup }) {
  const [open, setOpen] = useState(true);
  const id = useId();
  const Chevron = open ? ChevronDown : ChevronRight;
  return (
    <div className={styles.filterSection}>
      <button
        type="button"
        className={styles.filterSectionHead}
        aria-expanded={open}
        aria-controls={id}
        onClick={() => setOpen(!open)}
      >
        <Chevron size={14} aria-hidden="true" />
        <span>{group.title}</span>
      </button>
      {open && (
        <div id={id} className={styles.filterItems} role="group" aria-label={group.title}>
          {group.options.map((option) => (
            // biome-ignore lint/a11y/noLabelWithoutControl: the checkbox inside is the control
            <label key={option.value} className={styles.filterItem}>
              <Checkbox
                checked={group.selected.includes(option.value)}
                onCheckedChange={(checked) =>
                  group.onChange(
                    checked === true
                      ? [...group.selected, option.value]
                      : group.selected.filter((value) => value !== option.value),
                  )
                }
              />
              <span className={styles.filterLabel}>{option.label}</span>
              <span className={styles.filterCount}>{option.count}</span>
            </label>
          ))}
        </div>
      )}
    </div>
  );
}

export function FilterSidebar({ groups, label }: { groups: FilterGroup[]; label: string }) {
  const t = useT();
  return (
    <aside className={styles.sidebar} aria-label={label}>
      <div className={styles.sidebarHead}>
        <Funnel size={16} aria-hidden="true" />
        <span>{t('notify.capabilities.filters')}</span>
      </div>
      <div className={styles.sidebarBody}>
        {groups.map((group) => (
          <Section key={group.id} group={group} />
        ))}
      </div>
      <p className={styles.sidebarFoot}>{t('notify.capabilities.filtersNote')}</p>
    </aside>
  );
}

export function FilterSheet({
  groups,
  open,
  onOpenChange,
}: {
  groups: FilterGroup[];
  open: boolean;
  onOpenChange(open: boolean): void;
}) {
  const t = useT();
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content className={styles.sheet} aria-describedby={undefined}>
        <div className={styles.sheetHead}>
          <Dialog.Title size="5" mb="0">
            {t('notify.capabilities.filters')}
          </Dialog.Title>
          <Dialog.Close>
            <IconButton variant="ghost" color="gray" aria-label={t('notify.capabilities.closeFilters')}>
              <X size={18} aria-hidden="true" />
            </IconButton>
          </Dialog.Close>
        </div>
        <div className={styles.sheetBody}>
          {groups.map((group) => (
            <Section key={group.id} group={group} />
          ))}
        </div>
      </Dialog.Content>
    </Dialog.Root>
  );
}
