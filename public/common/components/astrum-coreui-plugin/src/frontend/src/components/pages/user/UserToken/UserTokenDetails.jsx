 
import React from 'react';
import {faKey} from '@fortawesome/free-solid-svg-icons';

import {NxButton, NxFontAwesomeIcon, NxModal, NxCopyToClipboard} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import './UserToken.scss';

export default function UserTokenDetails({userToken, onCloseClick}) {
  return <NxModal className="nx-modal--wide" onClose={onCloseClick}>
    <header className="nx-modal-header">
      <h2 className="nx-h2">
        <NxFontAwesomeIcon icon={faKey}/>
        <span>{UIStrings.USER_TOKEN.MENU.text}</span>
      </h2>
    </header>
    <div className="nx-modal-content">
      <p>{UIStrings.USER_TOKEN.LABELS.USER_TOKEN_NOTE} <strong>{UIStrings.USER_TOKEN.LABELS.KEEP_SECRET_NOTE}</strong>
      </p>
      <NxCopyToClipboard
          label={UIStrings.USER_TOKEN.LABELS.USER_TOKEN_NAME_CODE}
          content={userToken.nameCode}
          inputProps={{inputAttributes: {rows: 1, name: 'nameCode'}}}
      />
      <NxCopyToClipboard
          label={UIStrings.USER_TOKEN.LABELS.USER_TOKEN_PASS_CODE}
          content={userToken.passCode}
          inputProps={{inputAttributes: {rows: 1, name: 'passCode'}}}
      />
      <NxCopyToClipboard
          label={UIStrings.USER_TOKEN.LABELS.MAVEN_USAGE}
          content={`<server>\n\t<id>\${server}</id>\n\t<username>${userToken.nameCode}</username>\n\t<password>${userToken.passCode}</password>\n</server>`}
          inputProps={{inputAttributes: {rows: 5, name: 'mavenUsage'}}}
      />
      <NxCopyToClipboard
          label={UIStrings.USER_TOKEN.LABELS.BASE64_USER_TOKEN}
          content={window.btoa(`${userToken.nameCode}:${userToken.passCode}`)}
          inputProps={{inputAttributes: {rows: 1, name: 'base64'}}}
      />
      <p>{UIStrings.USER_TOKEN.LABELS.AUTO_HIDE}</p>
    </div>
    <footer className="nx-footer">
      <div className="nx-btn-bar">
        <NxButton onClick={onCloseClick}>{UIStrings.USER_TOKEN.BUTTONS.CLOSE}</NxButton>
      </div>
    </footer>
  </NxModal>
}
