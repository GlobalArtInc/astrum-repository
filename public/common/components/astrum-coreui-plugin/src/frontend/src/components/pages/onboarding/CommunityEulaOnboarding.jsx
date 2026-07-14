
import React from 'react';

import {NxInfoAlert} from '@globalart/react-shared-components';
import './CommunityOnboarding.scss';

export default function CommunityEulaOnboarding() {
  return (
    <div className="ce-eula-onboarding-screen">
        <NxInfoAlert>By clicking 'Agree', you acknowledge that you have read and agree to the End User License Agreement.</NxInfoAlert>
    </div>
  );
}
