
import React, {useState, useRef} from 'react';
import {AsFontAwesomeIcon, AsTooltip, AsP} from '@globalart/astrum-react-shared-components';
import {faBan, faShieldAlt, faAward, faExclamationCircle} from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../../constants/UIStrings';
import './IQServerColumns.scss';
import {useRepositoriesService} from '../RepositoriesContextProvider';
import {canUpdateHealthCheck, canReadHealthCheckSummary} from './IQServerHelpers';
import HealthCheckSummary from './HealthCheckSummary';
import {Utils} from '@globalart/astrum-ui-plugin';

const {
  LOADING,
  LOADING_ERROR,
  NOT_AVAILABLE_TOOLTIP_HC,
  ANALYZE_BUTTON,
  ANALYZING,
  SUMMARY: {NO_PERMISSION}
} = UIStrings.REPOSITORIES.LIST.HEALTH_CHECK;

export default function HealthCheckCell({name, openModal}) {
  const [state] = useRepositoriesService();

  const {enablingHealthCheckRepoName, enableHealthCheckError, readHealthCheckError, healthCheck} =
    state.context;

  const health = healthCheck ? healthCheck[name] : null;

  const canReadSummary = canReadHealthCheckSummary();

  const isLoading =
    (state.matches('enablingHealthCheckSingleRepo') && enablingHealthCheckRepoName === name) ||
    (state.matches('enablingHealthCheckAllRepos') && !health?.enabled) ||
    state.matches('readingHealthCheck');

  const isLoadingError =
    (!!enableHealthCheckError && enablingHealthCheckRepoName === name) ||
    (!!enableHealthCheckError && !enablingHealthCheckRepoName) ||
    !!readHealthCheckError;

  const isDisabled =
    state.matches('enablingHealthCheckSingleRepo') && enablingHealthCheckRepoName !== name;

  const showModal = (e) => {
    if (isDisabled) {
      return;
    }
    e.stopPropagation();
    openModal();
  };

  const indicatorsRef = useRef();
  const getSummaryBaseTopPosition = () => indicatorsRef.current.getBoundingClientRect().y;

  const [openSummary, setOpenSummary] = useState(false);
  const [showSummary, cancelShowSummary] = Utils.useDebounce(setOpenSummary, true, 500);
  const closeSummary = () => setOpenSummary(false); 

  if (isLoading) {
    return <AsP className="as-rhc-loading-lbl">{LOADING}</AsP>;
  }

  if (isLoadingError) {
    return (
      <AsP className="as-rhc-loading-error">
        <AsFontAwesomeIcon icon={faExclamationCircle} />
        {LOADING_ERROR}
      </AsP>
    );
  }

  if (!health || (!health.enabled && !canUpdateHealthCheck())) {
    return (
      <AsTooltip title={NOT_AVAILABLE_TOOLTIP_HC}>
        <AsFontAwesomeIcon icon={faBan} className="as-unavailable-icon" />
      </AsTooltip>
    );
  }

  if (!health.enabled) {
    return (
      <AsP
        className={isDisabled ? 'as-rhc-analyze-btn-disabled' : 'as-rhc-analyze-btn-enabled'}
        onClick={showModal}
      >
        {ANALYZE_BUTTON}
      </AsP>
    );
  }

  if (health.analyzing) {
    return <AsP className="as-rhc-analyzing-lbl">{ANALYZING}</AsP>;
  }

  if (health.enabled) {
    return (
      <>
        <AsTooltip title={!canReadSummary && NO_PERMISSION}>
          <AsP
            className="as-health-check-indicators"
            onMouseEnter={showSummary}
            onMouseLeave={cancelShowSummary}
            ref={indicatorsRef}
          >
            <AsFontAwesomeIcon icon={faShieldAlt} />
            {health.securityIssueCount}
            <AsFontAwesomeIcon icon={faAward} />
            {health.licenseIssueCount}
          </AsP>
        </AsTooltip>
        {openSummary && canReadSummary && (
          <HealthCheckSummary
            healthCheckData={health}
            baseTopPosition={getSummaryBaseTopPosition()}
            close={closeSummary}
          />
        )}
      </>
    );
  }
}
