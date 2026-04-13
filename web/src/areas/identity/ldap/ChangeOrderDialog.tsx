import { Button, Dialog, Flex, IconButton, Text } from '@radix-ui/themes';
import { ArrowUpDown, ChevronDown, ChevronUp, GripVertical } from 'lucide-react';
import { useState } from 'react';
import { api, useInvalidateApi } from '../../../api';
import { useT } from '../../../i18n';
import { describeError, useToast } from '../../../ui';
import { LDAP, LDAP_ORDER, type LdapServer } from '../api';
import layout from '../shared/Layout.module.css';
import styles from './LdapList.module.css';
import { move } from './model';

const DRAG_TYPE = 'application/x-astrum-ldap-order';

export function ChangeOrderDialog({ servers, onClose }: { servers: LdapServer[]; onClose(): void }) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const [names, setNames] = useState(() => servers.map((server) => server.name));
  const [dragging, setDragging] = useState<number | null>(null);
  const [over, setOver] = useState<number | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const save = async () => {
    setSaving(true);
    setError(null);
    try {
      await api.post(LDAP_ORDER, names);
      toast.success(t('identity.ldap.orderSaved'));
      await invalidate(LDAP);
      onClose();
    } catch (failure) {
      setError(describeError(t, failure));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog.Root
      open
      onOpenChange={(open) => {
        if (!open && !saving) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="550px">
        <Dialog.Title>
          <Flex align="center" gap="1">
            <ArrowUpDown size={20} aria-hidden="true" />
            {t('identity.ldap.orderTitle')}
          </Flex>
        </Dialog.Title>
        <Dialog.Description size="2" color="gray">
          {t('identity.ldap.orderText')}
        </Dialog.Description>
        <ol className={styles.orderList}>
          {names.map((name, index) => (
            <li
              key={name}
              className={styles.orderItem}
              draggable
              data-over={over === index || undefined}
              data-dragging={dragging === index || undefined}
              onDragStart={(event) => {
                event.dataTransfer.setData(DRAG_TYPE, name);
                event.dataTransfer.effectAllowed = 'move';
                setDragging(index);
              }}
              onDragEnd={() => {
                setDragging(null);
                setOver(null);
              }}
              onDragOver={(event) => {
                if (dragging !== null) {
                  event.preventDefault();
                  setOver(index);
                }
              }}
              onDrop={(event) => {
                event.preventDefault();
                if (dragging !== null) {
                  setNames((current) => move(current, dragging, index));
                }
                setDragging(null);
                setOver(null);
              }}
            >
              <span className={styles.grip} aria-hidden="true">
                <GripVertical size={14} />
              </span>
              <span className={styles.orderLabel}>
                {index + 1}.{name}
              </span>
              <span className={styles.orderButtons}>
                <IconButton
                  type="button"
                  size="1"
                  variant="soft"
                  color="gray"
                  disabled={index === 0}
                  aria-label={t('identity.ldap.moveUp', { name })}
                  onClick={() => setNames((current) => move(current, index, index - 1))}
                >
                  <ChevronUp size={14} aria-hidden="true" />
                </IconButton>
                <IconButton
                  type="button"
                  size="1"
                  variant="soft"
                  color="gray"
                  disabled={index === names.length - 1}
                  aria-label={t('identity.ldap.moveDown', { name })}
                  onClick={() => setNames((current) => move(current, index, index + 1))}
                >
                  <ChevronDown size={14} aria-hidden="true" />
                </IconButton>
              </span>
            </li>
          ))}
        </ol>
        {error && (
          <Text as="p" size="2" color="red" mt="3" role="alert">
            {error}
          </Text>
        )}
        <Flex gap="3" mt="4" justify="end" wrap="wrap">
          <Dialog.Close>
            <Button
              size="3"
              variant="outline"
              color="gray"
              className={layout.neutralButton}
              disabled={saving}
            >
              {t('identity.ldap.cancel')}
            </Button>
          </Dialog.Close>
          <Button size="3" className={layout.bigButton} loading={saving} onClick={save}>
            {t('identity.ldap.saveOrder')}
          </Button>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}
