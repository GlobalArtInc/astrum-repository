import { Box, Tabs } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import { useSearchParams } from 'react-router';

export interface ProfileTab {
  value: string;
  label: ReactNode;
  content: ReactNode;
  keepMounted?: boolean;
}

export function ProfileTabs({ tabs, defaultTab }: { tabs: ProfileTab[]; defaultTab?: string }) {
  const [params, setParams] = useSearchParams();
  const fallback = defaultTab ?? tabs[0]?.value ?? '';
  const asked = params.get('tab');
  const current = tabs.some((tab) => tab.value === asked) && asked ? asked : fallback;
  const select = (value: string) => {
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value === fallback) {
          next.delete('tab');
        } else {
          next.set('tab', value);
        }
        return next;
      },
      { replace: true },
    );
  };
  return (
    <Tabs.Root value={current} onValueChange={select} style={{ marginTop: -14 }}>
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
        <Tabs.Content key={tab.value} value={tab.value} forceMount={tab.keepMounted ? true : undefined}>
          <div hidden={tab.value !== current}>
            <Box pt="5">{tab.content}</Box>
          </div>
        </Tabs.Content>
      ))}
    </Tabs.Root>
  );
}
