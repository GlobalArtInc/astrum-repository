

import React, { Suspense } from 'react';
import { UIView } from '@uirouter/react';
import { SecurityEntityModalProvider } from '../pages/settings/security/SecurityEntityModalContext';
import { Box, Flex } from '@radix-ui/themes';

import './SettingsPageLayoutRadix.scss';
import SharedStrings from '../constants/SharedStrings';
import CommonStrings from '../constants/CommonStrings';

export default function SettingsPageLayoutRadix() {
  return (
    <SecurityEntityModalProvider>
      <Flex direction="column" className="settings-layout-radix">
        <Box px="5" pt="3" pb="2">
          <a href="#settings" className="settings-layout-radix__back-link">
            {SharedStrings.SETTINGS_LAYOUT.BACK_SETTINGS}
          </a>
        </Box>
        <Box className="settings-layout-radix__content">
          <Suspense fallback={<Box className="settings-layout-radix__loading">{CommonStrings.MESSAGES.LOADING}</Box>}>
            <UIView />
          </Suspense>
        </Box>
      </Flex>
    </SecurityEntityModalProvider>
  );
}
