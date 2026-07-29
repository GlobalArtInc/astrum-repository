
import React, {useState} from 'react';
import {
  NxFieldset,
  NxCheckbox,
  NxButton,
  NxFontAwesomeIcon,
  NxTooltip,
} from '@globalart/react-shared-components';
import classNames from 'classnames';

import ExtJS from '../../../interface/ExtJS';
import Permissions from '../../../constants/Permissions';

import {faCertificate} from '@fortawesome/free-solid-svg-icons';

import ValidationUtils from '../../../interface/ValidationUtils';
import SslCertificateDetailsModal from '../SslCertificateDetailsModal/SslCertificateDetailsModal';

import UIStrings from '../../../constants/UIStrings';
import './UseAstrumTruststore.scss';

const {
  USE_TRUST_STORE: {LABEL, DESCRIPTION, VIEW_CERTIFICATE, NOT_SECURE_URL},
} = UIStrings;

export default function UseAstrumTruststore({remoteUrl, ...checkboxProps}) {
  const [showModal, setShowModal] = useState(false);
  const canRead = ExtJS.checkPermission(Permissions.SSL_TRUSTSTORE.READ);
  const hasSecureRemoteUrl = ValidationUtils.isSecureUrl(remoteUrl);
  const disabledCheckbox = !hasSecureRemoteUrl;
  const disabledViewCertificate = !canRead || !hasSecureRemoteUrl;
  const viewCertificateClasses = classNames({
    disabled: disabledViewCertificate,
  });
  let message;

  if (!hasSecureRemoteUrl) {
    message = NOT_SECURE_URL;
  }

  const openModal = () => {
    if (!disabledViewCertificate) {
      setShowModal(true);
    }
  };
  const closeModal = () => setShowModal(false);

  return (
    <NxFieldset label={LABEL}>
      <div className="as-use-astrum-trust-store">
        <NxTooltip title={disabledCheckbox && message}>
          <span>
            <NxCheckbox {...checkboxProps} disabled={disabledCheckbox}>
              {DESCRIPTION}
            </NxCheckbox>
          </span>
        </NxTooltip>
        <NxTooltip
          title={disabledViewCertificate && UIStrings.PERMISSION_ERROR}
        >
          <NxButton
            variant="tertiary"
            onClick={openModal}
            className={viewCertificateClasses}
            type="button"
          >
            <NxFontAwesomeIcon icon={faCertificate} />
            <span>{VIEW_CERTIFICATE}</span>
          </NxButton>
        </NxTooltip>
      </div>
      {showModal && (
        <SslCertificateDetailsModal
          remoteUrl={remoteUrl}
          onCancel={closeModal}
        />
      )}
    </NxFieldset>
  );
}
