
import React from 'react';
import {useMachine} from '@xstate/react';
import classNames from 'classnames';
import ExtJS from '../../../interface/ExtJS';
import {
  AsButton,
  AsLoadWrapper,
  AsModal,
  AsWarningAlert,
  AsLoadError,
  AsTooltip
} from '@globalart/astrum-react-shared-components';

import ReadOnlyField from '../ReadOnlyField/ReadOnlyField';
import SslCertificateDetailsModalMachine from './SslCertificateDetailsModalMachine';
import DateUtils from '../../../interface/DateUtils';
import UIStrings from '../../../constants/UIStrings';
import Permissions from '../../../constants/Permissions';

/**
 *
 * @since 3.36
 * @param {string} remoteUrl the host to retrieve the certificate from
 * @param {function} onCancel a function to fire when the window is closed
 * @returns {JSX.Element}
 */
export default function SslCertificateDetailsModal({remoteUrl, onCancel}) {
  const url = new URL(remoteUrl);
  const [current, send] = useMachine(SslCertificateDetailsModalMachine, {
    context: {
      host: url.hostname,
      port: url.port ? url.port : '443',
      onCancel
    },
    devTools: true
  });
  const {certificateDetails = {}, error = null, isInTruststore = null} = current.context;
  const isLoading = current.matches('loading') || current.matches('adding') || current.matches('removing');
  const isViewing = current.matches('viewing');
  const hasLoadError = current.matches('loadError');
  const hasAddCertificateError = current.matches('addCertificateError');
  const hasRemoveCertificateError = current.matches('removeCertificateError');

  const SSL_CERTIFICATE_DETAILS = UIStrings.SSL_CERTIFICATE_DETAILS;
  const issuedDate = DateUtils.timestampToString(certificateDetails?.issuedOn);
  const expiredDate = DateUtils.timestampToString(certificateDetails?.expiresOn);

  const canCreate = ExtJS.checkPermission(Permissions.SSL_TRUSTSTORE.CREATE);
  const canDelete = ExtJS.checkPermission(Permissions.SSL_TRUSTSTORE.DELETE);
  const addCertificateClasses = classNames({
    disabled: !canCreate,
  });
  const removeCertificateClasses = classNames({
    disabled: !canDelete,
  });

  function retryHandler() {
    send({type: 'RETRY'});
  }

  function addToTruststore() {
    if(canCreate){
      send({type: 'ADD_CERTIFICATE'});
    }
  }

  function removeFromTruststore() {
    if(canDelete){
      send({type: 'REMOVE_CERTIFICATE'});
    }
  }

  return (
      <AsModal onCancel={onCancel}>
        <header className="as-modal-header">
          <h2 className="as-h2" id="modal-header-text">
            {SSL_CERTIFICATE_DETAILS.TITLE}
          </h2>
        </header>
        <div className="as-modal-content">
          <AsLoadWrapper retryHandler={retryHandler} loading={isLoading} error={hasLoadError && error}>
            {() => <>
              {hasAddCertificateError &&
              <AsLoadError titleMessage={SSL_CERTIFICATE_DETAILS.ADD_ERROR}
                           error={error}
                           retryHandler={retryHandler}/>}

              {hasRemoveCertificateError &&
              <AsLoadError titleMessage={SSL_CERTIFICATE_DETAILS.REMOVE_ERROR}
                           error={error}
                           retryHandler={retryHandler}/>}

              {isViewing && <>
                <AsWarningAlert>{SSL_CERTIFICATE_DETAILS.WARNING}</AsWarningAlert>
                <dl className="as-read-only">
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.NAME}
                                        value={certificateDetails.subjectCommonName}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.ORG}
                                        value={certificateDetails.subjectOrganization}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.UNIT}
                                        value={certificateDetails.subjectOrganizationalUnit}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.ISSUER_NAME}
                                        value={certificateDetails.issuerCommonName}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.ISSUER_ORG}
                                        value={certificateDetails.issuerOrganization}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.ISSUER_UNIT}
                                        value={certificateDetails.issuerOrganizationalUnit}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.ISSUE_DATE}
                                        value={issuedDate}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.EXPIRE_DATE}
                                        value={expiredDate}/>
                  <ReadOnlyField label={SSL_CERTIFICATE_DETAILS.FINGERPRINT}
                                        value={certificateDetails.fingerprint}/>
                </dl>
              </>}
            </>}
          </AsLoadWrapper>
        </div>
        <footer className="as-footer">
          <div className="as-btn-bar">
            {isInTruststore &&
            <AsTooltip title={!canDelete && UIStrings.PERMISSION_ERROR}>
              <AsButton variant="primary"
                        type="button"
                        disabled={isLoading}
                        className={removeCertificateClasses}
                        onClick={removeFromTruststore}>
                {SSL_CERTIFICATE_DETAILS.REMOVE_CERTIFICATE}
              </AsButton>
            </AsTooltip>}

            {isInTruststore === false && // isInTruststore will be null during loading, don't show this button then
            <AsTooltip title={!canCreate && UIStrings.PERMISSION_ERROR}>
              <AsButton variant="primary"
                        type="button"
                        disabled={isLoading}
                        className={addCertificateClasses}
                        onClick={addToTruststore}>
                {SSL_CERTIFICATE_DETAILS.ADD_CERTIFICATE}
              </AsButton>
            </AsTooltip>}

            <AsButton type="button" onClick={onCancel}>{UIStrings.CLOSE}</AsButton>
          </div>
        </footer>
      </AsModal>
  );
}
