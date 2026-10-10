
import React from 'react';

import {AsInfoAlert} from '@globalart/astrum-react-shared-components';
import './CommunityOnboarding.scss';
import UIStrings from '../../../constants/UIStrings';

export default function CommunityEulaOnboarding() {
  return (
    <div className="ce-eula-onboarding-screen">
        <AsInfoAlert>{UIStrings.PAGES.EULA_ONBOARDING}</AsInfoAlert>
    </div>
  );
}
