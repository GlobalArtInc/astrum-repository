import { Callout } from '@radix-ui/themes';
import { Lock } from 'lucide-react';

export function BuiltInNote({ text }: { text: string }) {
  return (
    <Callout.Root color="gray" variant="surface">
      <Callout.Icon>
        <Lock size={16} />
      </Callout.Icon>
      <Callout.Text>{text}</Callout.Text>
    </Callout.Root>
  );
}
