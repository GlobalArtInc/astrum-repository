
import React from 'react';

import {AsInfoAlert} from '@globalart/astrum-react-shared-components';
import './CommunityOnboarding.scss';

export default function CommunityEulaOnboarding() {
  return (
    <div className="ce-eula-onboarding-screen">
        <AsInfoAlert>By clicking 'Agree', you acknowledge that you have read and agree to the End User License Agreement.</AsInfoAlert>
    </div>
  );
}
