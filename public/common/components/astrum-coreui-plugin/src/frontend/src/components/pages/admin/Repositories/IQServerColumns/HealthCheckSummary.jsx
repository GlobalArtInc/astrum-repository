
import React, {useRef, useEffect, useState} from 'react';
import {NxH2, NxButtonBar, NxButton} from '@globalart/react-shared-components';
import UIStrings from '../../../../../constants/UIStrings';
import {canReadHealthCheckDetail} from './IQServerHelpers';

import './IQServerColumns.scss';

const {CAPTION, HELP_BUTTON, DETAILS_BUTTON} = UIStrings.REPOSITORIES.LIST.HEALTH_CHECK.SUMMARY;

const HELP_URL = 'http://links.globalart.dev/products/astrum/rhc/manual-remediation-with-rhc';
export default function HealthCheckSummary({healthCheckData, baseTopPosition, close}) {
  const {summaryUrl, detailUrl, iframeHeight, iframeWidth} = healthCheckData;

  const summaryRef = useRef();

  const [topPosition, setTopPosition] = useState("-100%");

  const canReadDetails = canReadHealthCheckDetail();

  useEffect(() => {
    const repoListTopPosition = document
      .querySelector('.as-content-body')
      .getBoundingClientRect().y;
    const initialTopPosition = baseTopPosition - repoListTopPosition;
    const maxSummaryY =
      window.innerHeight - (repoListTopPosition + summaryRef.current.offsetHeight) - 28;
    const position = initialTopPosition < maxSummaryY ? initialTopPosition : maxSummaryY;
    setTopPosition(position);
  }, []);

  const openInNewTab = (e, url) => {
    e.stopPropagation();
    window.open(url, '_blank');
  };

  return (
    <div
      className="as-health-check-summary"
      style={{
        top: topPosition
      }}
      onMouseLeave={close}
      ref={summaryRef}
      data-testid="as-health-check-summary"
    >
      <NxH2>{CAPTION}</NxH2>
      <iframe
        title={CAPTION}
        src={summaryUrl}
        style={{
          height: iframeHeight,
          width: iframeWidth
        }}
      />
      <NxButtonBar>
        <NxButton variant="tertiary" onClick={(e) => openInNewTab(e, HELP_URL)}>
          {HELP_BUTTON}
        </NxButton>
        <NxButton onClick={(e) => openInNewTab(e, detailUrl)} disabled={!canReadDetails}>
          {DETAILS_BUTTON}
        </NxButton>
      </NxButtonBar>
    </div>
  );
}
