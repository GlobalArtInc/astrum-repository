

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Shield, Eye, Filter, Asterisk, AppWindow, Code } from 'lucide-react';
import { PRIVILEGE_TYPES, PRIVILEGE_TYPE_LABELS } from './types';

import './PrivilegeTypeSelector.scss';

interface PrivilegeTypeSelectorProps {
  onSelect: (typeId: string) => void;
  selectedTypeId?: string | null;
}

const PRIVILEGE_TYPE_DESCRIPTIONS: Record<string, string> = {
  [PRIVILEGE_TYPES.REPOSITORY_ADMIN]: 'Full control of a repository',
  [PRIVILEGE_TYPES.REPOSITORY_VIEW]: 'Browse/read/edit access to repository content',
  [PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR]: 'Access filtered by content selector expression',
  [PRIVILEGE_TYPES.WILDCARD]: 'Pattern-based privilege',
  [PRIVILEGE_TYPES.APPLICATION]: 'Application-level permission',
  [PRIVILEGE_TYPES.SCRIPT]: 'Script execution permission',
};

const PRIVILEGE_TYPE_ICONS: Record<string, React.ElementType> = {
  [PRIVILEGE_TYPES.REPOSITORY_ADMIN]: Shield,
  [PRIVILEGE_TYPES.REPOSITORY_VIEW]: Eye,
  [PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR]: Filter,
  [PRIVILEGE_TYPES.WILDCARD]: Asterisk,
  [PRIVILEGE_TYPES.APPLICATION]: AppWindow,
  [PRIVILEGE_TYPES.SCRIPT]: Code,
};

export function PrivilegeTypeSelector({
  onSelect,
  selectedTypeId,
}: PrivilegeTypeSelectorProps) {
  const types = Object.values(PRIVILEGE_TYPES);

  return (
    <Box className="privilege-type-selector">
      <Box className="privilege-type-selector__grid">
        {types.map((typeId) => {
          const Icon = PRIVILEGE_TYPE_ICONS[typeId] || Asterisk;
          const isSelected = selectedTypeId === typeId;

          return (
            <button
              type="button"
              key={typeId}
              className={`privilege-type-selector__card ${
                isSelected ? 'privilege-type-selector__card--selected' : ''
              }`}
              onClick={() => onSelect(typeId)}
              data-testid={`privilege-type-card-${typeId}`}
            >
              <Flex align="start" gap="4">
                <div className={`privilege-type-selector__icon-wrapper privilege-type-selector__icon-wrapper--${typeId}`}>
                  <Icon size={24} />
                </div>
                <Box style={{ flex: 1 }}>
                  <Text weight="bold" size="3" mb="1" style={{ display: 'block' }}>
                    {PRIVILEGE_TYPE_LABELS[typeId] || typeId}
                  </Text>
                  <Text size="2" color="gray" style={{ lineHeight: '1.4', display: 'block' }}>
                    {PRIVILEGE_TYPE_DESCRIPTIONS[typeId]}
                  </Text>
                </Box>
              </Flex>
            </button>
          );
        })}
      </Box>
    </Box>
  );
}

export default PrivilegeTypeSelector;
