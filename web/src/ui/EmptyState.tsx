import { Flex, Heading, Text } from '@radix-ui/themes';
import type { LucideIcon } from 'lucide-react';
import { Inbox } from 'lucide-react';
import type { ReactNode } from 'react';

export interface EmptyStateProps {
  title: string;
  description?: ReactNode;
  icon?: LucideIcon;
  action?: ReactNode;
}

export function EmptyState({ title, description, icon: Icon = Inbox, action }: EmptyStateProps) {
  return (
    <Flex
      direction="column"
      align="center"
      justify="center"
      gap="2"
      py="8"
      px="4"
      style={{ textAlign: 'center' }}
    >
      <Icon size={32} strokeWidth={1.5} color="var(--gray-9)" aria-hidden="true" />
      <Heading as="h2" size="3" weight="medium">
        {title}
      </Heading>
      {description && (
        <Text as="p" size="2" color="gray" style={{ maxWidth: '32rem' }}>
          {description}
        </Text>
      )}
      {action && <Flex mt="2">{action}</Flex>}
    </Flex>
  );
}
