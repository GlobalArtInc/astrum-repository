import { Box, Tabs } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import { useSearchParams } from 'react-router';

export interface TabDefinition {
  value: string;
  label: ReactNode;
  content: ReactNode;
}

export const TAB_PARAM = 'tab';

export function QueryTabs({ tabs, defaultValue }: { tabs: TabDefinition[]; defaultValue?: string }) {
  const [params, setParams] = useSearchParams();
  const fallback = defaultValue ?? tabs[0]?.value ?? '';
  const asked = params.get(TAB_PARAM);
  const current = tabs.some((tab) => tab.value === asked) ? (asked ?? fallback) : fallback;
  const select = (value: string) => {
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value === fallback) {
          next.delete(TAB_PARAM);
        } else {
          next.set(TAB_PARAM, value);
        }
        return next;
      },
      { replace: true },
    );
  };
  return (
    <Tabs.Root value={current} onValueChange={select}>
      <Box style={{ overflowX: 'auto' }}>
        <Tabs.List>
          {tabs.map((tab) => (
            <Tabs.Trigger key={tab.value} value={tab.value}>
              {tab.label}
            </Tabs.Trigger>
          ))}
        </Tabs.List>
      </Box>
      {tabs.map((tab) => (
        <Tabs.Content key={tab.value} value={tab.value}>
          <Box pt="4">{tab.content}</Box>
        </Tabs.Content>
      ))}
    </Tabs.Root>
  );
}
