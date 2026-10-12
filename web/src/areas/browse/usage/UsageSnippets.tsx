import { Box, Dialog, Flex, IconButton, Tabs, Text } from '@radix-ui/themes';
import { X } from 'lucide-react';
import { type ReactNode, useMemo, useState } from 'react';
import { useT } from '../../../i18n';
import { CodeBlock, formatName } from '../../../ui';
import { type UsageContext, usageSnippets } from './snippets';
import styles from './Usage.module.css';

export interface UsageSnippetsProps {
  context: UsageContext;
}

export function UsageSnippets({ context }: UsageSnippetsProps) {
  const t = useT();
  const snippets = useMemo(() => usageSnippets(context), [context]);
  const [chosen, setChosen] = useState<string | undefined>(undefined);
  if (snippets.length === 0) {
    return (
      <Text as="p" size="2" color="gray">
        {t('browse.usage.none', { format: formatName(context.format) })}
      </Text>
    );
  }
  const current = snippets.some((snippet) => snippet.id === chosen) ? chosen : snippets[0]?.id;
  return (
    <Tabs.Root value={current} onValueChange={setChosen} className={styles.root}>
      <Box className={styles.tabs}>
        <Tabs.List size="1" aria-label={t('browse.usage.clients')}>
          {snippets.map((snippet) => (
            <Tabs.Trigger key={snippet.id} value={snippet.id}>
              {snippet.labelKey ? t(snippet.labelKey) : snippet.label}
            </Tabs.Trigger>
          ))}
        </Tabs.List>
      </Box>
      {snippets.map((snippet) => {
        const label = snippet.labelKey ? t(snippet.labelKey) : snippet.label;
        return (
          <Tabs.Content key={snippet.id} value={snippet.id}>
            <Flex direction="column" gap="2" pt="3">
              {snippet.hint && (
                <Text as="p" size="2" color="gray">
                  {t(snippet.hint, snippet.hintVars)}
                </Text>
              )}
              <CodeBlock code={snippet.code} copyLabel={t('browse.usage.copy', { label })} />
            </Flex>
          </Tabs.Content>
        );
      })}
    </Tabs.Root>
  );
}

export interface UsageDialogProps {
  context: UsageContext;
  title: string;
  description?: ReactNode;
  trigger: ReactNode;
}

export function UsageDialog({ context, title, description, trigger }: UsageDialogProps) {
  const t = useT();
  return (
    <Dialog.Root>
      <Dialog.Trigger>{trigger}</Dialog.Trigger>
      <Dialog.Content maxWidth="760px" className={styles.dialog}>
        <Flex justify="between" align="start" gap="3">
          <Dialog.Title mb="1">{title}</Dialog.Title>
          <Dialog.Close>
            <IconButton variant="ghost" color="gray" aria-label={t('browse.usage.close')}>
              <X size={18} aria-hidden="true" />
            </IconButton>
          </Dialog.Close>
        </Flex>
        <Dialog.Description size="2" color="gray" mb="3">
          {description ?? t('browse.usage.description')}
        </Dialog.Description>
        <UsageSnippets context={context} />
      </Dialog.Content>
    </Dialog.Root>
  );
}
