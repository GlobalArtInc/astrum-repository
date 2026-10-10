
import React from 'react';
import {
  AsFontAwesomeIcon,
  AsTooltip,
  AsSmallThreatCounter,
  AsTextLink,
  AsP
} from '@globalart/astrum-react-shared-components';
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
    return <AsP className="as-rhc-loading-lbl">{HEALTH_CHECK.LOADING}</AsP>;
  }

  if (isLoadingError) {
    return (
      <AsP className="as-rhc-loading-error">
        <AsFontAwesomeIcon icon={faExclamationCircle} />
        {HEALTH_CHECK.LOADING_ERROR}
      </AsP>
    );
  }

  if (!status) {
    return (
      <AsTooltip title={HEALTH_CHECK.NOT_AVAILABLE_TOOLTIP_FS}>
        <AsFontAwesomeIcon icon={faBan} className="as-unavailable-icon"/>
      </AsTooltip>
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
      <AsP className="as-rhc-loading-error">
        <AsFontAwesomeIcon icon={faExclamationCircle} />
        {errorMessage}
      </AsP>
    );
  }

  // ASTRUM-54323: 'message' is the non-admin badge ("Audit enabled" / "Quarantine enabled").
  // Backend guarantees message == null on the admin path so this branch is skipped and the
  // chiclet counters below render instead. See FirewallRepositoryStatusComponent#getRepositoryStatusXO.
  if (message) {
    return <AsP>{message}</AsP>;
  }

  return (
    <div className="as-p as-iq-policy-violations-counters">
      <AsSmallThreatCounter
        criticalCount={criticalComponentCount}
        severeCount={severeComponentCount}
        moderateCount={moderateComponentCount}
      />
      <AsTooltip title={HEALTH_CHECK.QUARANTINED_TOOLTIP}>
        <div>
          <AsFontAwesomeIcon icon={faBan} />
          {quarantinedComponentCount}
        </div>
      </AsTooltip>
      <AsTextLink href={reportUrl} external onClick={(e) => e.stopPropagation()} />
    </div>
  );
}
