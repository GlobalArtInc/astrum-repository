import React from 'react';
import { Button } from '@radix-ui/themes';
import { ChevronRight } from 'lucide-react';
import CommonStrings from '../../../../../constants/CommonStrings';

interface SettingsLinkProps {
  previewPath: string;
  label?: string;
}

/**
 * Opens the Astrum One settings page behind a section of the repository profile.
 *
 * @param previewPath hash path of the page without '#', e.g. `admin/security/roles`
 */
export function SettingsLink({ previewPath, label = CommonStrings.ACTIONS.CONFIGURE }: SettingsLinkProps) {
  return (
    <Button variant="ghost" size="1" asChild>
      <a href={`#${previewPath}`}>
        {label} <ChevronRight size={12} aria-hidden />
      </a>
    </Button>
  );
}

export default SettingsLink;
