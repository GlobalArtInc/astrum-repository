import { Badge, Button, Card, Dialog, Flex, TextField } from '@radix-ui/themes';
import { type FormEvent, useId, useRef, useState } from 'react';
import { createPasskey, isPasskeySupported } from '../../auth/webauthn';
import { useT } from '../../i18n';
import { DateTime, describeError, useToast } from '../../ui';
import { RecoveryCodesContent } from './RecoveryCodesContent';
import styles from './Security.module.css';
import {
  beginPasskey,
  fieldMessage,
  type Passkey,
  type PasskeyAdded,
  registerPasskey,
  removePasskey,
  renamePasskey,
  type TwoFactorStatus,
} from './twoFactor';

type PasskeyAction = 'rename' | 'remove';

interface PasskeyEdit {
  passkey: Passkey;
  action: PasskeyAction;
}

interface PasskeysCardProps {
  status: TwoFactorStatus;
  onChanged(): Promise<void>;
  onClosed(): void;
}

interface PasskeyItemProps {
  passkey: Passkey;
  onEdit(action: PasskeyAction): void;
}

interface DialogProps {
  onClose(): void;
  onChanged(): Promise<void>;
}

interface PasskeyDialogProps extends DialogProps {
  passkey: Passkey;
}

function PasskeyItem({ passkey, onEdit }: PasskeyItemProps) {
  const t = useT();
  return (
    <li className={styles.passkey}>
      <div className={styles.passkeyHead}>
        <span className={styles.passkeyName}>{passkey.name}</span>
        {passkey.backedUp && (
          <Badge size="1" color="gray">
            {t('user.security.backedUp')}
          </Badge>
        )}
        {!passkey.usable && (
          <Badge size="1" color="amber">
            {t('user.security.passkeyNotHere')}
          </Badge>
        )}
      </div>
      <p className={styles.hint}>
        {t.rich('user.security.passkeyCreated', { date: <DateTime value={passkey.createdAt} dateOnly /> })}
      </p>
      <p className={styles.hint}>
        {passkey.lastUsedAt
          ? t.rich('user.security.passkeyLastUsed', {
              date: <DateTime value={passkey.lastUsedAt} dateOnly />,
            })
          : t('user.security.passkeyNeverUsed')}
      </p>
      {!passkey.usable && <p className={styles.hint}>{t('user.security.passkeyOtherRp')}</p>}
      <Flex gap="2" wrap="wrap" mt="2">
        <Button
          size="1"
          variant="soft"
          aria-label={t('user.security.passkeyRenameOne', { name: passkey.name })}
          onClick={() => onEdit('rename')}
        >
          {t('user.security.passkeyRename')}
        </Button>
        <Button
          size="1"
          variant="soft"
          color="red"
          aria-label={t('user.security.passkeyRemoveOne', { name: passkey.name })}
          onClick={() => onEdit('remove')}
        >
          {t('user.security.passkeyRemove')}
        </Button>
      </Flex>
    </li>
  );
}

export function PasskeysCard({ status, onChanged, onClosed }: PasskeysCardProps) {
  const t = useT();
  const [adding, setAdding] = useState(false);
  const [editing, setEditing] = useState<PasskeyEdit | null>(null);
  if (!isPasskeySupported()) {
    return null;
  }
  const close = () => {
    setAdding(false);
    setEditing(null);
    onClosed();
  };
  const addable = status.passkeysAvailable && status.methods.includes('passkey');
  return (
    <Card size="2" className={styles.section}>
      <h2 className={styles.cardTitle}>{t('user.security.passkeys')}</h2>
      <p className={styles.hint}>{t('user.security.passkeysIntro')}</p>
      {status.passkeys.length === 0 ? (
        <p className={styles.line}>{t('user.security.passkeysEmpty')}</p>
      ) : (
        <ul className={styles.passkeys}>
          {status.passkeys.map((passkey) => (
            <PasskeyItem
              key={passkey.id}
              passkey={passkey}
              onEdit={(action) => setEditing({ passkey, action })}
            />
          ))}
        </ul>
      )}
      {!status.passkeysAvailable && <p className={styles.line}>{t('user.security.passkeysUnavailable')}</p>}
      {status.passkeysAvailable && !status.methods.includes('passkey') && (
        <p className={styles.line}>{t('user.security.passkeysOff')}</p>
      )}
      {addable && (
        <Flex gap="2" wrap="wrap" mt="4">
          <Button variant="soft" onClick={() => setAdding(true)}>
            {t('user.security.passkeyAdd')}
          </Button>
        </Flex>
      )}
      {adding && <AddPasskeyDialog onClose={close} onChanged={onChanged} />}
      {editing?.action === 'rename' && (
        <RenamePasskeyDialog
          key={editing.passkey.id}
          passkey={editing.passkey}
          onClose={close}
          onChanged={onChanged}
        />
      )}
      {editing?.action === 'remove' && (
        <RemovePasskeyDialog
          key={editing.passkey.id}
          passkey={editing.passkey}
          onClose={close}
          onChanged={onChanged}
        />
      )}
    </Card>
  );
}

function AddPasskeyDialog({ onClose, onChanged }: DialogProps) {
  const t = useT();
  const toast = useToast();
  const nameId = useId();
  const passwordId = useId();
  const pending = useRef(false);
  const [busy, setBusy] = useState(false);
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [nameError, setNameError] = useState<string | null>(null);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [codes, setCodes] = useState<string[] | null>(null);

  const report = (error: unknown) => {
    const nameMessage = fieldMessage(error, 'name');
    const passwordMessage = fieldMessage(error, 'currentPassword');
    setNameError(nameMessage);
    setPasswordError(passwordMessage);
    setFormError(nameMessage === null && passwordMessage === null ? describeError(t, error) : null);
  };

  const add = async (event: FormEvent) => {
    event.preventDefault();
    if (pending.current || name.trim() === '' || password === '') {
      return;
    }
    pending.current = true;
    setBusy(true);
    setNameError(null);
    setPasswordError(null);
    setFormError(null);
    setNotice(null);
    try {
      const setup = await beginPasskey({ currentPassword: password, name: name.trim() });
      const result = await createPasskey(setup.publicKey);
      if (!result.ok) {
        setNotice(
          result.failure === 'cancelled'
            ? t('user.security.passkeyCancelled')
            : t('user.security.passkeyFailed'),
        );
        return;
      }
      const added: PasskeyAdded = await registerPasskey({
        ceremony: setup.ceremony,
        credential: result.value,
        currentPassword: password,
      });
      setPassword('');
      await onChanged();
      if (added.recoveryCodes) {
        setCodes(added.recoveryCodes);
      } else {
        toast.success(t('user.security.passkeyAdded'));
        onClose();
      }
    } catch (error) {
      report(error);
    } finally {
      pending.current = false;
      setBusy(false);
    }
  };

  return (
    <Dialog.Root
      open
      onOpenChange={(open) => {
        if (!open && !busy) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="480px" className={styles.dialog}>
        {codes ? (
          <RecoveryCodesContent codes={codes} onDone={onClose} />
        ) : (
          <form onSubmit={(event) => void add(event)} noValidate>
            <Dialog.Title className={styles.dialogTitle}>{t('user.security.passkeyAddTitle')}</Dialog.Title>
            <Dialog.Description size="2" color="gray">
              {t('user.security.passkeyAddText')}
            </Dialog.Description>
            <label htmlFor={nameId} className={styles.label}>
              {t('user.security.passkeyName')}
            </label>
            <TextField.Root
              id={nameId}
              name="name"
              value={name}
              maxLength={64}
              placeholder={t('user.security.passkeyNamePlaceholder')}
              autoFocus
              aria-invalid={nameError ? true : undefined}
              onChange={(event) => setName(event.target.value)}
            />
            <p className={styles.hint}>{t('user.security.passkeyNameHint')}</p>
            {nameError && (
              <p className={styles.error} role="alert">
                {nameError}
              </p>
            )}
            <label htmlFor={passwordId} className={styles.label}>
              {t('user.security.password')}
            </label>
            <TextField.Root
              id={passwordId}
              name="password"
              type="password"
              value={password}
              autoComplete="current-password"
              aria-invalid={passwordError ? true : undefined}
              onChange={(event) => setPassword(event.target.value)}
            />
            {passwordError && (
              <p className={styles.error} role="alert">
                {passwordError}
              </p>
            )}
            {notice && <p className={styles.hint}>{notice}</p>}
            {formError && (
              <p className={styles.error} role="alert">
                {formError}
              </p>
            )}
            <Flex gap="3" justify="end" mt="4" wrap="wrap">
              <Dialog.Close>
                <Button type="button" variant="soft" color="gray" disabled={busy}>
                  {t('user.security.cancel')}
                </Button>
              </Dialog.Close>
              <Button type="submit" loading={busy} disabled={name.trim() === '' || password === '' || busy}>
                {t('user.security.continue')}
              </Button>
            </Flex>
          </form>
        )}
      </Dialog.Content>
    </Dialog.Root>
  );
}

function RenamePasskeyDialog({ passkey, onClose, onChanged }: PasskeyDialogProps) {
  const t = useT();
  const toast = useToast();
  const nameId = useId();
  const pending = useRef(false);
  const [busy, setBusy] = useState(false);
  const [name, setName] = useState(passkey.name);
  const [nameError, setNameError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const rename = async (event: FormEvent) => {
    event.preventDefault();
    if (pending.current || name.trim() === '') {
      return;
    }
    pending.current = true;
    setBusy(true);
    setNameError(null);
    setFormError(null);
    try {
      await renamePasskey(passkey.id, name.trim());
      await onChanged();
      toast.success(t('user.security.passkeyRenamed'));
      onClose();
    } catch (error) {
      const message = fieldMessage(error, 'name');
      setNameError(message);
      setFormError(message === null ? describeError(t, error) : null);
    } finally {
      pending.current = false;
      setBusy(false);
    }
  };

  return (
    <Dialog.Root
      open
      onOpenChange={(open) => {
        if (!open && !busy) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="480px" className={styles.dialog}>
        <form onSubmit={(event) => void rename(event)} noValidate>
          <Dialog.Title className={styles.dialogTitle}>{t('user.security.passkeyRenameTitle')}</Dialog.Title>
          <Dialog.Description size="2" color="gray">
            {t('user.security.passkeyRenameText')}
          </Dialog.Description>
          <label htmlFor={nameId} className={styles.label}>
            {t('user.security.passkeyName')}
          </label>
          <TextField.Root
            id={nameId}
            name="name"
            value={name}
            maxLength={64}
            autoFocus
            aria-invalid={nameError ? true : undefined}
            onChange={(event) => setName(event.target.value)}
          />
          {nameError && (
            <p className={styles.error} role="alert">
              {nameError}
            </p>
          )}
          {formError && (
            <p className={styles.error} role="alert">
              {formError}
            </p>
          )}
          <Flex gap="3" justify="end" mt="4" wrap="wrap">
            <Dialog.Close>
              <Button type="button" variant="soft" color="gray" disabled={busy}>
                {t('user.security.cancel')}
              </Button>
            </Dialog.Close>
            <Button type="submit" loading={busy} disabled={name.trim() === '' || busy}>
              {t('user.security.passkeyRename')}
            </Button>
          </Flex>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function RemovePasskeyDialog({ passkey, onClose, onChanged }: PasskeyDialogProps) {
  const t = useT();
  const toast = useToast();
  const passwordId = useId();
  const pending = useRef(false);
  const [busy, setBusy] = useState(false);
  const [password, setPassword] = useState('');
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const remove = async (event: FormEvent) => {
    event.preventDefault();
    if (pending.current || password === '') {
      return;
    }
    pending.current = true;
    setBusy(true);
    setPasswordError(null);
    setFormError(null);
    try {
      await removePasskey(passkey.id, password);
      await onChanged();
      toast.success(t('user.security.passkeyRemoved'));
      onClose();
    } catch (error) {
      const message = fieldMessage(error, 'currentPassword');
      setPasswordError(message);
      setFormError(message === null ? describeError(t, error) : null);
    } finally {
      pending.current = false;
      setBusy(false);
    }
  };

  return (
    <Dialog.Root
      open
      onOpenChange={(open) => {
        if (!open && !busy) {
          onClose();
        }
      }}
    >
      <Dialog.Content maxWidth="480px" className={styles.dialog}>
        <form onSubmit={(event) => void remove(event)} noValidate>
          <Dialog.Title className={styles.dialogTitle}>{t('user.security.passkeyRemoveTitle')}</Dialog.Title>
          <Dialog.Description size="2" color="gray">
            {t('user.security.passkeyRemoveText', { name: passkey.name })}
          </Dialog.Description>
          <label htmlFor={passwordId} className={styles.label}>
            {t('user.security.password')}
          </label>
          <TextField.Root
            id={passwordId}
            name="password"
            type="password"
            value={password}
            autoComplete="current-password"
            autoFocus
            aria-invalid={passwordError ? true : undefined}
            onChange={(event) => setPassword(event.target.value)}
          />
          {passwordError && (
            <p className={styles.error} role="alert">
              {passwordError}
            </p>
          )}
          {formError && (
            <p className={styles.error} role="alert">
              {formError}
            </p>
          )}
          <Flex gap="3" justify="end" mt="4" wrap="wrap">
            <Dialog.Close>
              <Button type="button" variant="soft" color="gray" disabled={busy}>
                {t('user.security.cancel')}
              </Button>
            </Dialog.Close>
            <Button type="submit" color="red" loading={busy} disabled={password === '' || busy}>
              {t('user.security.passkeyRemove')}
            </Button>
          </Flex>
        </form>
      </Dialog.Content>
    </Dialog.Root>
  );
}
