import { Flex } from '@radix-ui/themes';
import { useState } from 'react';
import { useT } from '../../../i18n';
import { amountIn, bestUnit, bytesIn, QUOTA_UNITS, type QuotaUnit } from '../model/quota';
import { NumberInput, SelectInput } from './fields';

interface QuotaLimitInputProps {
  label: string;
  hint: string;
  error?: string;
  disabled?: boolean;
  value: number | null;
  onChange(value: number | null): void;
}

export function QuotaLimitInput({ label, hint, error, disabled, value, onChange }: QuotaLimitInputProps) {
  const t = useT();
  const [unit, setUnit] = useState<QuotaUnit>(() => bestUnit(value));
  return (
    <Flex gap="3" align="end" wrap="wrap">
      <NumberInput
        label={label}
        hint={hint}
        error={error}
        disabled={disabled}
        min={0}
        value={value === null ? null : amountIn(value, unit)}
        onChange={(amount) => onChange(amount === null ? null : bytesIn(amount, unit))}
      />
      <SelectInput
        label={t('repositories.form.quotaUnit')}
        disabled={disabled}
        value={unit}
        options={QUOTA_UNITS.map((item) => ({ value: item, label: item }))}
        onChange={(next) => setUnit(next as QuotaUnit)}
      />
    </Flex>
  );
}
