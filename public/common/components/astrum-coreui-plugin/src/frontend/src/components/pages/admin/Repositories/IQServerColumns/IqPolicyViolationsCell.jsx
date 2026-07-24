
import React from 'react';
import {
  NxFontAwesomeIcon,
  NxTooltip,
  NxSmallThreatCounter,
  NxTextLink,
  NxP
} from '@globalart/react-shared-components';
import {faBan, faExclamationCircle} from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../../constants/UIStrings';
import './IQServerColumns.scss';
import {useRepositoriesService} from '../RepositoriesContextProvider';

const {HEALTH_CHECK} = UIStrings.REPOSITORIES.LIST;

export default function IqPolicyViolationsCell({name}) {
  const [state] = useRepositoriesService();

  const {readFirewallRepositoryStatusError, firewallStatus} = state.context;

  const status = firewallStatus ? firewallStatus[name] : null;

  const isLoading = state.matches('readingFirewallStatus');

  const isLoadingError = !!readFirewallRepositoryStatusError;

  if (isLoading) {
    return <NxP className="as-rhc-loading-lbl">{HEALTH_CHECK.LOADING}</NxP>;
  }

  if (isLoadingError) {
    return (
      <NxP className="as-rhc-loading-error">
        <NxFontAwesomeIcon icon={faExclamationCircle} />
        {HEALTH_CHECK.LOADING_ERROR}
      </NxP>
    );
  }

  if (!status) {
    return (
      <NxTooltip title={HEALTH_CHECK.NOT_AVAILABLE_TOOLTIP_FS}>
        <NxFontAwesomeIcon icon={faBan} className="as-unavailable-icon"/>
      </NxTooltip>
    );
  }

  const {
    criticalComponentCount,
    severeComponentCount,
    moderateComponentCount,
    quarantinedComponentCount,
    reportUrl,
    message,
    errorMessage
  } = status;

  if (errorMessage) {
    return (
      <NxP className="as-rhc-loading-error">
        <NxFontAwesomeIcon icon={faExclamationCircle} />
        {errorMessage}
      </NxP>
    );
  }

  // ASTRUM-54323: 'message' is the non-admin badge ("Audit enabled" / "Quarantine enabled").
  // Backend guarantees message == null on the admin path so this branch is skipped and the
  // chiclet counters below render instead. See FirewallRepositoryStatusComponent#getRepositoryStatusXO.
  if (message) {
    return <NxP>{message}</NxP>;
  }

  return (
    <div className="nx-p as-iq-policy-violations-counters">
      <NxSmallThreatCounter
        criticalCount={criticalComponentCount}
        severeCount={severeComponentCount}
        moderateCount={moderateComponentCount}
      />
      <NxTooltip title={HEALTH_CHECK.QUARANTINED_TOOLTIP}>
        <div>
          <NxFontAwesomeIcon icon={faBan} />
          {quarantinedComponentCount}
        </div>
      </NxTooltip>
      <NxTextLink href={reportUrl} external onClick={(e) => e.stopPropagation()} />
    </div>
  );
}
