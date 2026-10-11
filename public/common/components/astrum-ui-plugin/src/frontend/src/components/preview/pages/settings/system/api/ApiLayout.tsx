

import React from 'react';
import { Box, Flex, IconButton } from '@radix-ui/themes';
import { PanelLeftClose, PanelLeft } from 'lucide-react';
import ApiStrings from '../../../../constants/pages/settings/system/ApiStrings';

export interface ApiLayoutProps {
  leftPanel: React.ReactNode;
  rightPanel: React.ReactNode;
  leftCollapsed: boolean;
  onToggleLeft: () => void;
}

export function ApiLayout({ leftPanel, rightPanel, leftCollapsed, onToggleLeft }: ApiLayoutProps) {
  return (
    <Flex gap="3" align="start" className="api-layout">
      {!leftCollapsed && (
        <Box className="api-layout__left" data-testid="api-layout-left">
          {leftPanel}
        </Box>
      )}
      <Flex direction="column" gap="2" className="api-layout__toggle-col">
        <IconButton
          type="button"
          variant="soft"
          size="1"
          onClick={onToggleLeft}
          aria-label={leftCollapsed ? ApiStrings.LAYOUT.EXPAND_ENDPOINT_LIST : ApiStrings.LAYOUT.COLLAPSE_ENDPOINT_LIST}
        >
          {leftCollapsed ? <PanelLeft size={16} /> : <PanelLeftClose size={16} />}
        </IconButton>
      </Flex>
      <Box className="api-layout__right" style={{ flex: 1, minWidth: 0 }} data-testid="api-layout-right">
        {rightPanel}
      </Box>
    </Flex>
  );
}
