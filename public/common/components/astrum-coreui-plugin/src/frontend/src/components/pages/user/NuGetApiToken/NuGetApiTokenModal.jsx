
import React from 'react';
import {faKey} from "@fortawesome/free-solid-svg-icons";
import {ExtJS} from '@globalart/astrum-ui-plugin';

import {
  AsButton,
  AsFontAwesomeIcon,
  AsModal,
  AsCopyToClipboard,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import './NugetApiToken.scss';

export default function NuGetApiTokenModal({apiKey, onCloseClick}) {
  const exampleCode = UIStrings.NUGET_API_KEY.DETAILS.REGISTER_COMMAND
      .replace('{0}', apiKey)
      .replace('{1}', ExtJS.absolutePath('repository/{repository name}/'));

  return <AsModal id="nuget-api-key-modal" className="as-modal--wide" onClose={onCloseClick}>
    <header className="as-modal-header">
      <h2 className="as-h2">
        <AsFontAwesomeIcon icon={faKey} fixedWidth/>
        <span> {UIStrings.NUGET_API_KEY.MENU.text} </span>
      </h2>
    </header>

    <div className="as-modal-content">
      <p>
        <span> {UIStrings.NUGET_API_KEY.DETAILS.MAIN} <strong> {UIStrings.NUGET_API_KEY.DETAILS.WARNING} </strong> </span> <br/>
      </p>
      <AsCopyToClipboard
          label={UIStrings.NUGET_API_KEY.DETAILS.API_KEY_TEXT}
          content={apiKey}
          inputProps={{inputAttributes: {rows: 1}}}
          id="nuget-api-key"
      />
      <AsCopyToClipboard
          label={UIStrings.NUGET_API_KEY.DETAILS.REGISTER_TEXT}
          content={exampleCode}
          inputProps={{inputAttributes: {rows: 2}}}
          id="nuget-example-command"
      />
      <p>
        <span> {UIStrings.NUGET_API_KEY.DETAILS.AUTO_CLOSE}</span> <br/>
      </p>
    </div>
    <footer className="as-footer">
      <div className="as-btn-bar">
        <AsButton onClick={onCloseClick}>{UIStrings.NUGET_API_KEY.CLOSE}</AsButton>
      </div>
    </footer>
  </AsModal>
}
