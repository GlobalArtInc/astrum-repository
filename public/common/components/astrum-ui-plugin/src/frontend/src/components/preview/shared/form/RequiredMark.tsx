

import React from 'react';
import { Text } from '@radix-ui/themes';

/** Red asterisk that follows a required-field label. Purely decorative — aria-hidden. */
export function RequiredMark() {
  return (
    <Text as="span" color="red" weight="bold" aria-hidden="true"> *</Text>
  );
}

export default RequiredMark;
