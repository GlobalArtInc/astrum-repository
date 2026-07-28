

import React from 'react';
import { Callout, Text } from '@radix-ui/themes';
import { Info } from 'lucide-react';
import LoginPageStrings from '../../../constants/LoginPageStrings';

const { INITIAL_PASSWORD_MESSAGE } = LoginPageStrings;

/**
 * Displays initial admin password file path information using a Radix Callout.
 * Shown during first-time setup when admin password needs to be retrieved from file.
 */
export default function InitialPasswordInfo({ passwordFilePath }) {
  return (
    <Callout.Root color="blue" size="2">
      <Callout.Icon>
        <Info size={16} />
      </Callout.Icon>
      <Callout.Text asChild>
        <span>
          <Text size="2">{INITIAL_PASSWORD_MESSAGE}</Text>
          <br />
          <Text as="span" weight="medium" size="2" style={{ fontFamily: 'monospace', overflowWrap: 'break-word' }}>
            {passwordFilePath}
          </Text>
        </span>
      </Callout.Text>
    </Callout.Root>
  );
}
