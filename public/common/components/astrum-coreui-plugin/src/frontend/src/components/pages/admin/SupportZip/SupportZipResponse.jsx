
import React from 'react';
import UIStrings from '../../../../constants/UIStrings';
import {FieldWrapper, Textfield} from '@globalart/astrum-ui-plugin';
import {AsButton, AsFontAwesomeIcon} from '@globalart/react-shared-components';
import {faDownload} from '@fortawesome/free-solid-svg-icons';

import {SectionFooter} from '@globalart/astrum-ui-plugin';

export default function SupportZipResponse({response, download}) {
  return <div className='as-support-zip-response'>
    <h2>{UIStrings.SUPPORT_ZIP.CREATED_TITLE}</h2>
    <p>Support ZIP has been created.<br/>
    You can reference this file on the filesystem or download the file from your browser.</p>
    <FieldWrapper labelText={UIStrings.SUPPORT_ZIP.CREATED_NAME_LABEL}>
      <Textfield
        name='name'
        disabled={true}
        value={response.name}
      />
    </FieldWrapper>
    <FieldWrapper labelText={UIStrings.SUPPORT_ZIP.CREATED_SIZE_LABEL}>
      <Textfield
        name='size'
        disabled={true}
        value={response.size}
      />
    </FieldWrapper>
    <FieldWrapper labelText={UIStrings.SUPPORT_ZIP.CREATED_PATH_LABEL}>
      <Textfield
        name='file'
        disabled={true}
        value={response.file}
      />
    </FieldWrapper>
    <SectionFooter>
      <AsButton variant="primary" onClick={(event) => download(event, response.name)} type="submit">
        <AsFontAwesomeIcon icon={faDownload}/>
        <span>{UIStrings.SUPPORT_ZIP.CREATED_DOWNLOAD_BUTTON}</span>
      </AsButton>
    </SectionFooter>
  </div>;
}
