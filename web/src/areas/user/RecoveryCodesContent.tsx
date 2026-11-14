import { Button, Callout, Code, Dialog, Flex } from '@radix-ui/themes';
import { TriangleAlert } from 'lucide-react';
import { useId, useState } from 'react';
import { useT } from '../../i18n';
import { CheckboxField, copyText, useToast } from '../../ui';
import styles from './Security.module.css';

interface RecoveryCodesContentProps {
  codes: string[];
  onDone(): void;
}

export function RecoveryCodesContent({ codes, onDone }: RecoveryCodesContentProps) {
  const t = useT();
  const toast = useToast();
  const savedId = useId();
  const [saved, setSaved] = useState(false);

  const copyAll = async () => {
    try {
      await copyText(codes.join('\n'));
      toast.success(t('user.security.copied'));
    } catch {
      toast.error(t('user.security.copyFailed'));
    }
  };

  const download = () => {
    const url = URL.createObjectURL(
      new Blob([`${codes.join('\n')}\n`], { type: 'text/plain;charset=utf-8' }),
    );
    const link = document.createElement('a');
    link.href = url;
    link.download = 'astrum-recovery-codes.txt';
    document.body.append(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };

  return (
    <>
      <Dialog.Title className={styles.dialogTitle}>{t('user.security.codesTitle')}</Dialog.Title>
      <Callout.Root color="amber" role="alert" className={styles.notice}>
        <Callout.Icon>
          <TriangleAlert size={16} aria-hidden="true" />
        </Callout.Icon>
        <Callout.Text>{t('user.security.codesOnce')}</Callout.Text>
      </Callout.Root>
      <Dialog.Description size="2" color="gray">
        {t('user.security.codesText')}
      </Dialog.Description>
      <ul className={styles.codes}>
        {codes.map((item) => (
          <li key={item}>
            <Code size="3" variant="soft">
              {item}
            </Code>
          </li>
        ))}
      </ul>
      <Flex gap="2" wrap="wrap">
        <Button type="button" variant="soft" color="gray" onClick={() => void copyAll()}>
          {t('user.security.copyAll')}
        </Button>
        <Button type="button" variant="soft" color="gray" onClick={download}>
          {t('user.security.download')}
        </Button>
      </Flex>
      <div className={styles.saved}>
        <CheckboxField id={savedId} label={t('user.security.saved')} value={saved} onChange={setSaved} />
      </div>
      <Flex justify="end" mt="4">
        <Button type="button" disabled={!saved} onClick={onDone}>
          {t('user.security.done')}
        </Button>
      </Flex>
    </>
  );
}
