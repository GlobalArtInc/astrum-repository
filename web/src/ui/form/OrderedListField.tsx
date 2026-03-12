import { Flex, IconButton, Select, Text } from '@radix-ui/themes';
import { ArrowDown, ArrowUp, GripVertical, X } from 'lucide-react';
import { type DragEvent, type ReactNode, useState } from 'react';
import { useT } from '../../i18n';
import styles from './Field.module.css';
import { describedBy, type FieldProps, FieldShell, type Option, useFieldId } from './fields';

export interface OrderedListFieldProps extends FieldProps {
  value: string[];
  onChange(value: string[]): void;
  options: Option[];
  addPlaceholder: string;
  empty?: ReactNode;
  noOptions?: ReactNode;
  describe?: (value: string) => ReactNode;
}

export function moveItem<T>(items: readonly T[], from: number, to: number): T[] {
  if (from === to || from < 0 || from >= items.length || to < 0 || to >= items.length) {
    return [...items];
  }
  const next = [...items];
  const [moved] = next.splice(from, 1);
  next.splice(to, 0, moved as T);
  return next;
}

export function addItem(items: readonly string[], item: string): string[] {
  return items.includes(item) ? [...items] : [...items, item];
}

export function removeItem(items: readonly string[], item: string): string[] {
  return items.filter((existing) => existing !== item);
}

export function OrderedListField(props: OrderedListFieldProps) {
  const t = useT();
  const id = useFieldId(props.id);
  const [dragged, setDragged] = useState<number | null>(null);
  const [over, setOver] = useState<number | null>(null);
  const labels = new Map(props.options.map((option) => [option.value, option.label]));
  const labelOf = (value: string) => labels.get(value) ?? value;
  const available = props.options.filter((option) => !props.value.includes(option.value));
  const disabled = props.disabled === true;

  const drop = (event: DragEvent, index: number) => {
    event.preventDefault();
    if (dragged !== null) {
      props.onChange(moveItem(props.value, dragged, index));
    }
    setDragged(null);
    setOver(null);
  };

  return (
    <FieldShell {...props} id={id}>
      {props.value.length === 0 ? (
        <Text size="2" color="gray" className={styles.listEmpty}>
          {props.empty ?? t('ui.form.noneChosen')}
        </Text>
      ) : (
        <ol
          className={styles.ordered}
          aria-label={props.label}
          aria-describedby={describedBy(id, props.hint, props.error)}
        >
          {props.value.map((value, index) => {
            const name = labelOf(value);
            return (
              <li
                key={value}
                className={styles.orderedItem}
                data-over={over === index && dragged !== index ? 'true' : undefined}
                data-dragging={dragged === index ? 'true' : undefined}
                draggable={!disabled}
                onDragStart={(event) => {
                  event.dataTransfer.effectAllowed = 'move';
                  event.dataTransfer.setData('text/plain', value);
                  setDragged(index);
                }}
                onDragOver={(event) => {
                  if (dragged !== null) {
                    event.preventDefault();
                    setOver(index);
                  }
                }}
                onDragEnd={() => {
                  setDragged(null);
                  setOver(null);
                }}
                onDrop={(event) => drop(event, index)}
              >
                {!disabled && (
                  <span className={styles.grip} title={t('ui.form.drag')} aria-hidden="true">
                    <GripVertical size={14} />
                  </span>
                )}
                <Text size="1" color="gray" className={styles.position} aria-hidden="true">
                  {index + 1}
                </Text>
                <Flex direction="column" className={styles.orderedLabel}>
                  <Text size="2" weight="medium">
                    {name}
                  </Text>
                  {props.describe?.(value)}
                </Flex>
                {!disabled && (
                  <Flex gap="1" className={styles.orderedButtons}>
                    <IconButton
                      type="button"
                      size="1"
                      variant="ghost"
                      color="gray"
                      disabled={index === 0}
                      aria-label={t('ui.form.moveUp', { name })}
                      onClick={() => props.onChange(moveItem(props.value, index, index - 1))}
                    >
                      <ArrowUp size={14} aria-hidden="true" />
                    </IconButton>
                    <IconButton
                      type="button"
                      size="1"
                      variant="ghost"
                      color="gray"
                      disabled={index === props.value.length - 1}
                      aria-label={t('ui.form.moveDown', { name })}
                      onClick={() => props.onChange(moveItem(props.value, index, index + 1))}
                    >
                      <ArrowDown size={14} aria-hidden="true" />
                    </IconButton>
                    <IconButton
                      type="button"
                      size="1"
                      variant="ghost"
                      color="red"
                      aria-label={t('ui.form.remove', { name })}
                      onClick={() => props.onChange(removeItem(props.value, value))}
                    >
                      <X size={14} aria-hidden="true" />
                    </IconButton>
                  </Flex>
                )}
              </li>
            );
          })}
        </ol>
      )}
      {!disabled &&
        (available.length > 0 ? (
          <Select.Root
            value=""
            onValueChange={(value) => {
              if (value) {
                props.onChange(addItem(props.value, value));
              }
            }}
          >
            <Select.Trigger
              id={id}
              placeholder={props.addPlaceholder}
              aria-label={props.addPlaceholder}
              aria-invalid={props.error ? true : undefined}
            />
            <Select.Content position="popper">
              {available.map((option) => (
                <Select.Item key={option.value} value={option.value}>
                  {option.label}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        ) : (
          props.noOptions && (
            <Text id={id} size="1" color="gray">
              {props.noOptions}
            </Text>
          )
        ))}
    </FieldShell>
  );
}
