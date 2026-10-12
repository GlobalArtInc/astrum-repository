import { TextField } from '@radix-ui/themes';
import { Search } from 'lucide-react';

export function ListSearch({
  value,
  onChange,
  label,
}: {
  value: string;
  onChange(value: string): void;
  label: string;
}) {
  return (
    <TextField.Root
      type="search"
      value={value}
      placeholder={label}
      aria-label={label}
      onChange={(event) => onChange(event.target.value)}
      style={{ flex: '1 1 240px', minWidth: 0 }}
    >
      <TextField.Slot>
        <Search size={16} aria-hidden="true" />
      </TextField.Slot>
    </TextField.Root>
  );
}
