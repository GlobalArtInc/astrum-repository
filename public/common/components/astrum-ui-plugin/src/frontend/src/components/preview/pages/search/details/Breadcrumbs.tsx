

import React from 'react';
import { Box, Flex, Button, Text } from '@radix-ui/themes';
import { ChevronRight } from 'lucide-react';

export interface BreadcrumbItem {
  label: string;
  href?: string;
  onClick?: () => void;
}

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
}

export function Breadcrumbs({ items }: BreadcrumbsProps) {
  return (
    <Box p="4" pt="4">
      <Flex align="center" gap="2">
        {items.map((item, index) => (
          <Flex key={index} align="center" gap="2">
            {index > 0 && <ChevronRight size={14} color="var(--gray-9)" />}
            {item.onClick || item.href ? (
              <Button
                variant="ghost"
                size="2"
                color="blue"
                onClick={item.onClick}
              >
                <Text size="2" color="blue">
                  {item.label}
                </Text>
              </Button>
            ) : (
              <Text size="2" weight="medium" color="gray">
                {item.label}
              </Text>
            )}
          </Flex>
        ))}
      </Flex>
    </Box>
  );
}

