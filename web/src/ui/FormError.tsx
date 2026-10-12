import { Callout } from '@radix-ui/themes';
import { CircleAlert } from 'lucide-react';

export function FormError({ message }: { message: string | null | undefined }) {
  if (!message) {
    return null;
  }
  return (
    <Callout.Root color="red" role="alert">
      <Callout.Icon>
        <CircleAlert size={16} />
      </Callout.Icon>
      <Callout.Text>{message}</Callout.Text>
    </Callout.Root>
  );
}
