

import React from 'react';
import { Button, Tooltip } from '@radix-ui/themes';
import { ExternalLink } from 'lucide-react';
import { getHeritageEquivalent } from '../../../../../../../utils/previewHeritageNavigation';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';

const CLASSIC_UI_TOOLTIP = RepositoryProfileStrings.CLASSIC_LINK.OPENS_DEFAULT_UI_ASTRUM_ONE;

interface ClassicSettingsLinkProps {
  previewPath: string;
  label?: string;
}

/**
 * Opens the Classic UI equivalent of a Preview admin settings page in a new tab.
 * Uses the existing heritage route mapping to compute the Classic URL.
 * ASTRUM-51915: Prevents Repository Profile from bypassing Coming Soon gates.
 */
export function ClassicSettingsLink({ previewPath, label = CommonStrings.ACTIONS.CONFIGURE }: ClassicSettingsLinkProps) {
  const classicHash = getHeritageEquivalent(previewPath);
  if (!classicHash) return null;

  const handleClick = () => {
    const baseUrl = `${window.location.origin}${window.location.pathname}`;
    window.open(`${baseUrl}#${classicHash}`, '_blank', 'noopener,noreferrer');
  };

  return (
    <Tooltip content={CLASSIC_UI_TOOLTIP}>
      <Button
        variant="ghost"
        size="1"
        onClick={handleClick}
      >
        {label} <ExternalLink size={12} />
      </Button>
    </Tooltip>
  );
}

export default ClassicSettingsLink;
